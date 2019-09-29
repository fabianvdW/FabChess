use super::super::board_representation::game_state::*;
use super::super::movegen;
use super::super::movegen::MoveList;
use super::super::GameState;
use super::cache::CacheEntry;
use super::quiescence::{is_capture, q_search, see};
use super::searcher::Search;
use super::searcher::SearchUtils;
use super::GradedMove;
use super::*;
use super::{MATED_IN_MAX, MATE_SCORE, MAX_SEARCH_DEPTH, STANDARD_SCORE};
use crate::bitboards;
use crate::board_representation::game_state_attack_container::GameStateAttackContainer;
use crate::evaluation::eval_game_state;
use crate::move_generation::makemove::{make_move, make_nullmove};

pub const FUTILITY_MARGIN: i16 = 90;
pub const FUTILITY_DEPTH: i16 = 8;
pub const STATIC_NULL_MOVE_MARGIN: i16 = 120;
pub const STATIC_NULL_MOVE_DEPTH: i16 = 5;
pub const NULL_MOVE_PRUNING_DEPTH: i16 = 3;

#[inline(always)]
pub fn mate_distance_pruning(p: &mut CombinedSearchParameters) -> SearchInstruction {
    //My score can at maximum be mate with this move
    p.beta = p.beta.min(MATE_SCORE - (p.current_depth as i16 + 1));
    //My score is atleast mate in this move
    p.alpha = p.alpha.max(-(MATE_SCORE - p.current_depth as i16));
    //The bounds will never be changed both at once
    if p.alpha >= p.beta {
        return SearchInstruction::StopSearching(p.alpha);
    }
    SearchInstruction::ContinueSearching
}
#[inline(always)]
pub fn max_depth(p: &CombinedSearchParameters, su: &mut SearchUtils) -> SearchInstruction {
    if p.current_depth >= (MAX_SEARCH_DEPTH - 1) {
        su.thread_memory.reserved_attack_container.attack_containers[p.current_depth]
            .write_state(p.game_state);
        SearchInstruction::StopSearching(
            eval_game_state(
                p.game_state,
                &su.thread_memory.reserved_attack_container.attack_containers[p.current_depth],
            )
            .final_eval
                * p.color,
        )
    } else {
        SearchInstruction::ContinueSearching
    }
}

#[inline(always)]
pub fn get_pvtable_move(p: &CombinedSearchParameters, search: &Search) -> Option<GameMove> {
    //PV-Table lookup
    if let Some(ce) = search.principal_variation[p.current_depth] {
        if ce.hash == p.game_state.hash {
            return Some(CacheEntry::u16_to_mv(ce.mv, p.game_state));
        }
    }
    None
}

pub fn principal_variation_search(mut p: CombinedSearchParameters, su: &mut SearchUtils) -> i16 {
    //Step 0. Prepare variables
    su.search.search_statistics.add_normal_node(p.current_depth);
    clear_pv(p.current_depth, su.search);
    let root = p.current_depth == 0;
    let is_pv_node = p.beta - p.alpha > 1;
    //Step 1. Check timeout and if stop flag is set
    if su.search.search_statistics.nodes_searched % 1024 == 0 {
        checkup(su.search, su.stop)
    }
    if su.search.stop {
        return STANDARD_SCORE;
    }

    //Step 2. Max Search depth reached
    if let SearchInstruction::StopSearching(res) = max_depth(&p, su) {
        return res;
    }

    //Step 3. Check for draw or mate distance pruning if not root (need best move at root)
    if !root {
        if let SearchInstruction::StopSearching(res) = check_for_draw(p.game_state, su.history) {
            return res;
        }
        //Mate distance pruning
        if let SearchInstruction::StopSearching(res) = mate_distance_pruning(&mut p) {
            return res;
        }
    }
    let original_alpha = p.alpha;

    //Step 4. Attacks and in check  flag
    su.thread_memory.reserved_attack_container.attack_containers[p.current_depth]
        .write_state(p.game_state);
    let incheck = in_check(
        p.game_state,
        &su.thread_memory.reserved_attack_container.attack_containers[p.current_depth],
    );

    //Step 5. Check extensions if not at root
    if incheck && !root {
        p.depth_left += 1;
    }

    //Step 6. Drop into quiescence search if depth == 0
    if p.depth_left <= 0 {
        debug_assert_eq!(p.depth_left, 0);
        su.search.search_statistics.add_q_root();
        return q_search(p, su);
    }

    //Step 7. PV-Table Lookup
    let pv_table_move = get_pvtable_move(&p, su.search);

    //Step 8. TT Lookup
    let mut static_evaluation = None;
    let mut tt_move: Option<GameMove> = None;
    if let SearchInstruction::StopSearching(res) =
        su.cache
            .lookup(su.search, &p, &mut static_evaluation, &mut tt_move)
    {
        return res;
    }
    su.history
        .push(p.game_state.hash, p.game_state.half_moves == 0);

    //Make static eval
    let prunable = !is_pv_node && !incheck;
    if static_evaluation.is_none()
        && (prunable
            && (p.depth_left <= STATIC_NULL_MOVE_DEPTH || p.depth_left >= NULL_MOVE_PRUNING_DEPTH)
            || !incheck && p.depth_left <= FUTILITY_DEPTH)
    {
        let eval_res = eval_game_state(
            p.game_state,
            &su.thread_memory.reserved_attack_container.attack_containers[p.current_depth],
        );
        static_evaluation = Some(eval_res.final_eval);
        su.search.search_statistics.add_static_eval_node();
    }
    //Replace static eval by tt score if available
    /*if false
        && static_evaluation.is_some()
        && tt_entry.is_some()
        && tt_entry.as_ref().unwrap().depth > 0
    {
        let content = tt_entry.as_ref().expect("TT entry impossible");
        let score = static_evaluation.expect("Static eval impossible") * color;
        if !content.alpha && !content.beta
            || content.alpha && content.score < score
            || content.beta && content.score > score
        {
            static_evaluation = Some(content.score * color);
            su.search.search_statistics.add_cache_hit_replace_eval();
        }
    }*/
    //Static Null Move Pruning
    if prunable
        && p.depth_left <= STATIC_NULL_MOVE_DEPTH
        && static_evaluation.expect("Static null move") * p.color
            - STATIC_NULL_MOVE_MARGIN * p.depth_left
            >= p.beta
    {
        su.history.pop();
        su.search.search_statistics.add_static_null_move_node();
        return static_evaluation.expect("Static null move 2") * p.color
            - STATIC_NULL_MOVE_DEPTH * p.depth_left;
    }

    //Null Move Forward Pruning
    if prunable
        && p.depth_left >= NULL_MOVE_PRUNING_DEPTH
        && p.game_state.phase.phase > 0.
        && static_evaluation.expect("null move static") * p.color >= p.beta
    {
        let nextgs = make_nullmove(p.game_state);
        let rat = -principal_variation_search(
            CombinedSearchParameters::from(
                -p.beta,
                -p.beta + 1,
                (p.depth_left - 4 - p.depth_left / 6).max(0),
                &nextgs,
                -p.color,
                p.current_depth + 1,
            ),
            su,
        );

        if rat >= p.beta {
            su.search.search_statistics.add_nm_pruning();
            su.history.pop();
            return rat;
        }
    }

    //Internal Iterative Deepening
    let mut has_generated_moves = if is_pv_node
        && !incheck
        && pv_table_move.is_none()
        && tt_move.is_none()
        && p.depth_left > 6
    {
        su.history.pop();
        principal_variation_search(
            CombinedSearchParameters::from(
                p.alpha,
                p.beta,
                p.depth_left - 2,
                &p.game_state,
                p.color,
                p.current_depth,
            ),
            su,
        );
        su.history
            .push(p.game_state.hash, p.game_state.half_moves == 0);
        su.search.search_statistics.add_iid_node();
        if su.search.stop {
            return STANDARD_SCORE;
        }
        tt_move = su.search.pv_table[p.current_depth].pv[0];
        true
    } else {
        false
    };

    //Prepare Futility Pruning
    let mut futil_pruning = p.depth_left <= FUTILITY_DEPTH && !incheck && !root;
    let futil_margin = if futil_pruning {
        static_evaluation.expect("Futil pruning") * p.color + p.depth_left * FUTILITY_MARGIN
    } else {
        0
    };
    if !has_generated_moves {
        su.thread_memory.reserved_movelist.move_lists[p.current_depth].counter = 0;
    }
    let mut hash_and_pv_move_counter = 0;
    {
        if pv_table_move.is_some() {
            hash_and_pv_move_counter += 1;
        }
        if tt_move.is_some() && pv_table_move.is_none() {
            hash_and_pv_move_counter += 1;
        } else if tt_move.is_some() {
            //Make sure that tt_move != pv_table_move
            if *tt_move
                .as_ref()
                .expect("Couldn't unwrap tt move although we have one")
                != *pv_table_move
                    .as_ref()
                    .expect("Couldn't unwrap pv move although we have one")
            {
                hash_and_pv_move_counter += 1;
            }
        }
    }

    let mut current_max_score = STANDARD_SCORE;
    let mut index: usize = 0;
    let mut moves_tried: usize = 0;
    let mut moves_from_movelist_tried: usize = 0;
    let mut quiets_tried: usize = 0;
    while moves_tried
        < su.thread_memory.reserved_movelist.move_lists[p.current_depth].counter
            + hash_and_pv_move_counter
        || !has_generated_moves
    {
        if moves_tried == hash_and_pv_move_counter && !has_generated_moves {
            has_generated_moves = true;
            make_and_evaluate_moves(
                p.game_state,
                su.search,
                p.current_depth,
                &mut su.thread_memory.reserved_movelist.move_lists[p.current_depth],
                &su.thread_memory.reserved_attack_container.attack_containers[p.current_depth],
            );
            continue;
        }
        let mut move_score = 0.;
        let mv: GameMove = if moves_tried < hash_and_pv_move_counter {
            if moves_tried == 0 {
                if let Some(pvmv) = pv_table_move {
                    pvmv
                } else {
                    tt_move.expect("Moves tried ==0 and no pv move, couldn't unwrap even tt move")
                }
            } else {
                tt_move.expect("Moves tried >0 and no tt move")
            }
        } else {
            let available_moves =
                su.thread_memory.reserved_movelist.move_lists[p.current_depth].counter;
            let r = get_next_gm(
                &mut su.thread_memory.reserved_movelist.move_lists[p.current_depth],
                moves_from_movelist_tried,
                available_moves,
            );
            move_score = r.1;
            su.thread_memory.reserved_movelist.move_lists[p.current_depth].move_list[r.0].unwrap()
        };
        //Make sure that our move is not the same as tt or pv move, if we have any
        if moves_tried >= hash_and_pv_move_counter {
            moves_from_movelist_tried += 1;
            if let Some(pv_move) = pv_table_move.as_ref() {
                if mv == *pv_move {
                    moves_tried += 1;
                    continue;
                }
            }
            if let Some(tt_mv) = tt_move.as_ref() {
                if mv == *tt_mv {
                    moves_tried += 1;
                    continue;
                }
            }
        }
        moves_tried += 1;
        if root && su.search.search_statistics.time_elapsed > 1000 {
            println!(
                "info depth {} currmove {:?} currmovenumber {}",
                p.depth_left,
                mv,
                (index + 1)
            );
            //UCI-Reporting
        }
        let isc = is_capture(&mv);
        let isp = if let GameMoveType::Promotion(_, _) = mv.move_type {
            true
        } else {
            false
        };
        let next_state = make_move(p.game_state, &mv);
        //--------------------------------------------------------------
        //Futility Pruning
        if futil_pruning
            && !isc
            && !isp
            && current_max_score > MATED_IN_MAX
            && !in_check_slow(&next_state)
        {
            if futil_margin <= p.alpha {
                su.search.search_statistics.add_futil_pruning();
                continue;
            } else {
                futil_pruning = false;
            }
        }
        if !root
            && p.depth_left <= 2
            && !isc
            && !isp
            && !incheck
            && current_max_score > MATED_IN_MAX
            && su.search.history_score[p.game_state.color_to_move][mv.from][mv.to] < 0
        {
            su.search.search_statistics.add_history_pruned();
            continue;
        }

        let mut following_score: i16;
        let mut reduction = 0;
        if p.depth_left > 2
            && !incheck
            && (!isc || move_score < 0.)
            && index >= 2
            && (!root || index >= 5)
        {
            //FRUITED RELOADED REDUCTION! NEXT THREE LINES ARE COPIED:
            reduction = (f64::from(p.depth_left - 1).sqrt() + ((index - 1) as f64).sqrt()) as i16;
            if isc || isp {
                reduction /= 2;
            }
            if is_pv_node {
                reduction = (f64::from(reduction) * 0.66) as i16;
            }
            if in_check_slow(&next_state) {
                reduction -= 1;
            }
            if su.search.history_score[p.game_state.color_to_move][mv.from][mv.to] > 0 {
                reduction -= 1;
            }
            reduction = reduction.min(p.depth_left - 1);
            reduction = reduction.max(1);
        }
        //Passer extension
        /*if !isc
            && !isp
            && !incheck
            && mv.piece_type == PieceType::Pawn
            && is_passer(game_state, &mv)
        {
            depth_left += 1;
        }*/
        if p.depth_left <= 2 || !is_pv_node || index == 0 {
            following_score = -principal_variation_search(
                CombinedSearchParameters::from(
                    -p.beta,
                    -p.alpha,
                    p.depth_left - 1 - reduction,
                    &next_state,
                    -p.color,
                    p.current_depth + 1,
                ),
                su,
            );
            if reduction > 0 && following_score > p.alpha {
                following_score = -principal_variation_search(
                    CombinedSearchParameters::from(
                        -p.beta,
                        -p.alpha,
                        p.depth_left - 1,
                        &next_state,
                        -p.color,
                        p.current_depth + 1,
                    ),
                    su,
                );
            }
        } else {
            following_score = -principal_variation_search(
                CombinedSearchParameters::from(
                    -p.alpha - 1,
                    -p.alpha,
                    p.depth_left - 1 - reduction,
                    &next_state,
                    -p.color,
                    p.current_depth + 1,
                ),
                su,
            );
            if following_score > p.alpha {
                following_score = -principal_variation_search(
                    CombinedSearchParameters::from(
                        -p.beta,
                        -p.alpha,
                        p.depth_left - 1,
                        &next_state,
                        -p.color,
                        p.current_depth + 1,
                    ),
                    su,
                );
            }
        }

        if following_score > current_max_score && !su.search.stop {
            su.search.pv_table[p.current_depth].pv[0] = Some(mv);
            current_max_score = following_score;
            concatenate_pv(p.current_depth, su.search);
            if root && su.search.search_statistics.time_elapsed > 1000 {
                let nps = su.search.search_statistics.getnps();
                println!(
                    "info depth {} nodes {} nps {} score cp {} lowerbound pv {}",
                    p.depth_left,
                    su.search.search_statistics.nodes_searched,
                    nps,
                    following_score,
                    su.search.pv_table[0]
                );
                //UCI-Reporting
            }
        }
        if following_score > p.alpha {
            p.alpha = following_score;
        }
        if p.alpha >= p.beta {
            su.search
                .search_statistics
                .add_normal_node_beta_cutoff(index);
            if !isc {
                su.search.hh_score[p.game_state.color_to_move][mv.from][mv.to] +=
                    p.depth_left as usize * p.depth_left as usize;
                su.search.history_score[p.game_state.color_to_move][mv.from][mv.to] +=
                    p.depth_left as isize * p.depth_left as isize;
                decrement_history_quiets(
                    su.search,
                    p.current_depth,
                    quiets_tried,
                    p.depth_left as isize,
                    p.game_state.color_to_move,
                );
                //Replace killers
                //Dont replace if already in table
                if let Some(s) = su.search.killer_moves[p.current_depth][0] {
                    if s.from == mv.from && s.to == mv.to && s.move_type == mv.move_type {
                        break;
                    }
                }
                if let Some(s) = su.search.killer_moves[p.current_depth][1] {
                    if s.from == mv.from && s.to == mv.to && s.move_type == mv.move_type {
                        break;
                    }
                }
                if let Some(s) = su.search.killer_moves[p.current_depth][0] {
                    su.search.killer_moves[p.current_depth][1] = Some(s);
                }
                su.search.killer_moves[p.current_depth][0] = Some(mv);
            }
            break;
        } else if !isc {
            su.search.quiets_tried[p.current_depth][quiets_tried] = Some(mv);
            quiets_tried += 1;
            su.search.bf_score[p.game_state.color_to_move][mv.from][mv.to] +=
                p.depth_left as usize * p.depth_left as usize;
        }

        index += 1;
    }

    su.history.pop();
    let game_status = check_end_condition(p.game_state, moves_tried > 0, incheck);
    if game_status != GameResult::Ingame {
        clear_pv(p.current_depth, su.search);
        return leaf_score(game_status, p.color, p.current_depth as i16);
    }

    if p.alpha < p.beta {
        su.search
            .search_statistics
            .add_normal_node_non_beta_cutoff();
    }
    //Make cache
    if !su.search.stop {
        make_cache(
            su.cache,
            &su.search.pv_table[p.current_depth],
            current_max_score,
            p.game_state,
            original_alpha,
            p.beta,
            p.depth_left,
            su.root_pliesplayed,
            static_evaluation,
            root,
        );
    }
    current_max_score
}

pub fn is_passer(g: &GameState, mv: &GameMove) -> bool {
    let pawn_board = 1u64 << mv.to;
    let enemy_pawns = g.pieces[PAWN][1 - g.color_to_move] & !pawn_board;
    let mut enemy_front_span = if g.color_to_move == WHITE {
        bitboards::b_front_span(enemy_pawns)
    } else {
        bitboards::w_front_span(enemy_pawns)
    };
    enemy_front_span |=
        bitboards::west_one(enemy_front_span) | bitboards::east_one(enemy_front_span);
    (pawn_board & !enemy_front_span).count_ones() > 0
}

pub fn decrement_history_quiets(
    search: &mut Search,
    current_depth: usize,
    quiets_tried: usize,
    depth_left: isize,
    side_to_move: usize,
) {
    for i in 0..quiets_tried {
        let mv = search.quiets_tried[current_depth][i].as_ref().unwrap();
        search.history_score[side_to_move][mv.from][mv.to] -= depth_left * depth_left;
    }
}

#[inline(always)]
pub fn make_and_evaluate_moves(
    game_state: &GameState,
    search: &mut Search,
    current_depth: usize,
    move_list: &mut MoveList,
    attack_container: &GameStateAttackContainer,
) {
    movegen::generate_moves(&game_state, false, move_list, attack_container);
    //Move Ordering
    //1. PV-Move +30000
    //2. Hash move + 29999
    //if SEE>0
    //3. Winning captures Sort by SEE + 10000
    //4. Equal captures Sort by SEE+ 10000
    //5. Killer moves + 5000
    //6. Non captures (history heuristic) history heuristic score
    //7. Losing captures (SEE<0) see score
    let mut mv_index = 0;
    while mv_index < move_list.counter {
        let mv: &GameMove = move_list.move_list[mv_index].as_ref().unwrap();
        if is_capture(mv) {
            if GameMoveType::EnPassant == mv.move_type {
                move_list.graded_moves[mv_index] = Some(GradedMove::new(mv_index, 9999.0));
            } else {
                let mut sval = f64::from(see(&game_state, &mv, true, &mut search.see_buffer));
                if sval >= 0.0 {
                    sval += 10000.0;
                }
                move_list.graded_moves[mv_index] = Some(GradedMove::new(mv_index, sval));
            }
        } else {
            //Assing history score
            let score = search.hh_score[game_state.color_to_move][mv.from][mv.to] as f64
                / search.bf_score[game_state.color_to_move][mv.from][mv.to] as f64
                / 1000.0;
            move_list.graded_moves[mv_index] = Some(GradedMove::new(mv_index, score));
        }
        mv_index += 1;
    }

    {
        //Killer moves
        if let Some(s) = search.killer_moves[current_depth][0] {
            let mv_index = find_move(&s, move_list, false);
            if mv_index < move_list.counter {
                move_list.graded_moves[mv_index].as_mut().unwrap().score += 5000.0;
            }
        }
        if let Some(s) = search.killer_moves[current_depth][1] {
            let mv_index = find_move(&s, move_list, false);
            if mv_index < move_list.counter {
                move_list.graded_moves[mv_index].as_mut().unwrap().score += 5000.0;
            }
        }
    }
}
