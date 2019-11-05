use super::super::board_representation::game_state::*;
use super::super::movegen;
use super::super::GameState;
use super::quiescence::{is_capture, q_search, see};
use super::GradedMove;
use super::*;
use super::{MATED_IN_MAX, MATE_SCORE, MAX_SEARCH_DEPTH, STANDARD_SCORE};
use crate::evaluation::eval_game_state;
use crate::move_generation::makemove::{make_move, make_nullmove};
use crate::search::searcher::Thread;

pub const FUTILITY_MARGIN: i16 = 90;
pub const FUTILITY_DEPTH: i16 = 6;
pub const STATIC_NULL_MOVE_MARGIN: i16 = 120;
pub const STATIC_NULL_MOVE_DEPTH: i16 = 5;
pub const NULL_MOVE_PRUNING_DEPTH: i16 = 3;

pub fn principal_variation_search(mut p: CombinedSearchParameters, thread: &mut Thread) -> i16 {
    //Step 0. Prepare variables
    thread.search_statistics.add_normal_node(p.current_depth);
    clear_pv(p.current_depth, thread);
    let root = p.current_depth == 0;
    let is_pv_node = p.beta - p.alpha > 1;
    //Step 1. Check timeout and if stop flag is set, if we are main thread
    if thread.search_statistics.nodes_searched % 4096 == 0 {
        checkup(thread)
    }
    if thread.search_statistics.nodes_searched % 8192 == 0 {
        thread.itcs.update(
            thread.id,
            thread.search_statistics.nodes_searched,
            thread.search_statistics.seldepth,
        );
    }
    if thread.self_stop {
        return STANDARD_SCORE;
    }

    //Step 2. Max Search depth reached
    if let SearchInstruction::StopSearching(res) = max_depth(&p, thread) {
        return res;
    }

    //Step 3. Check for draw or mate distance pruning if not root (need best move at root)
    if !root {
        if let SearchInstruction::StopSearching(res) = check_for_draw(p.game_state, &thread.history)
        {
            return res;
        }
        //Mate distance pruning
        if let SearchInstruction::StopSearching(res) = mate_distance_pruning(&mut p) {
            return res;
        }
    }
    let original_alpha = p.alpha;

    //Step 4. Attacks and in check  flag
    thread.attack_container.attack_containers[p.current_depth].write_state(p.game_state);
    let incheck = in_check(
        p.game_state,
        &thread.attack_container.attack_containers[p.current_depth],
    );

    //Step 5. Check extensions if not at root
    if incheck && !root {
        p.depth_left += 1;
    }

    //Step 6. Drop into quiescence search if depth == 0
    if p.depth_left <= 0 {
        debug_assert_eq!(p.depth_left, 0);
        thread.search_statistics.add_q_root();
        return q_search(p, thread);
    }

    //Step 7. PV-Table Lookup
    let pv_table_move = get_pvtable_move(&p, thread);

    //Step 8. TT Lookup
    let mut static_evaluation = None;
    let mut tt_move: Option<GameMove> = None;
    if let SearchInstruction::StopSearching(res) = thread.itcs.cache.lookup(
        &p,
        &mut static_evaluation,
        &mut tt_move,
        thread.root_plies_played,
    ) {
        thread.search_statistics.add_cache_hit_aj_replace_ns();
        thread.pv_table[p.current_depth].pv[0] = tt_move;
        return res;
    }
    if tt_move.is_some() {
        thread.search_statistics.add_cache_hit_ns();
    }
    thread
        .history
        .push(p.game_state.hash, p.game_state.half_moves == 0);

    //Step 9. Static Eval if needed
    let prunable = !is_pv_node && !incheck;
    make_eval(&p, thread, &mut static_evaluation, prunable);

    //Step 10. Prunings
    if prunable {
        //Step 10.1 Static Null Move Pruning
        if let SearchInstruction::StopSearching(res) =
            static_null_move_pruning(&p, thread, static_evaluation)
        {
            return res;
        }
        //Step 10.2 Null Move Forward Pruning
        if let SearchInstruction::StopSearching(res) =
            null_move_pruning(&p, thread, static_evaluation)
        {
            return res;
        }
    }

    //Step 11. Internal Iterative Deepening
    let mut has_generated_moves = if is_pv_node
        && !incheck
        && pv_table_move.is_none()
        && tt_move.is_none()
        && p.depth_left > 6
    {
        if let SearchInstruction::StopSearching(res) =
            internal_iterative_deepening(&p, thread, &mut tt_move)
        {
            return res;
        }
        true
    } else {
        false
    };

    //Step 12. Futil Pruning and margin preparation
    let futil_margin = prepare_futility_pruning(&p, static_evaluation);
    //Step 13. Prepare staged movegen
    let hash_and_pv_move_counter =
        prepare_staged_movegen(&p, thread, has_generated_moves, &pv_table_move, &tt_move);

    //Step 14. Iterate through all moves
    let mut current_max_score = STANDARD_SCORE;
    let mut index: usize = 0;
    let mut moves_tried: usize = 0;
    let mut moves_from_movelist_tried: usize = 0;
    let mut quiets_tried: usize = 0;
    while moves_tried
        < thread.movelist.move_lists[p.current_depth].counter + hash_and_pv_move_counter
        || !has_generated_moves
    {
        //Step 14.1. If tt move and pv move have been tried, generate all moves
        if moves_tried == hash_and_pv_move_counter && !has_generated_moves {
            has_generated_moves = true;
            make_and_evaluate_moves(p.game_state, thread, p.current_depth);
            continue;
        }

        //Step 14.2. Select the next move
        let (mv, move_score) = select_next_move(
            &p,
            thread,
            moves_tried,
            moves_from_movelist_tried,
            hash_and_pv_move_counter,
            &tt_move,
            &pv_table_move,
        );

        //Step 14.3. If the move is from the movelist, make sure we haven't searched it already as tt move or pv table move
        if moves_tried >= hash_and_pv_move_counter {
            moves_from_movelist_tried += 1;
            if let SearchInstruction::SkipMove = is_duplicate(&mv, &pv_table_move, &tt_move) {
                moves_tried += 1;
                continue;
            }
        }
        moves_tried += 1;

        //Step 14.4. UCI Reporting at root
        //uci_report_move(&p, su, &mv, index);

        let isc = is_capture(&mv);
        let isp = if let GameMoveType::Promotion(_, _) = mv.move_type {
            true
        } else {
            false
        };
        let is_quiet_move = !isc && !isp;
        let next_state = make_move(p.game_state, &mv);

        if !root
            && is_quiet_move
            && current_max_score > MATED_IN_MAX
            && p.game_state.has_non_pawns(p.game_state.color_to_move)
            && !in_check_slow(&next_state)
        {
            //Step 14.5. Futility Pruning. Skip quiet moves if futil_margin can't raise alpha
            if futil_margin <= p.alpha {
                thread.search_statistics.add_futil_pruning();
                index += 1;
                continue;
            }
            //Step 14.6. History Pruning. Skip quiet moves in low depths if they are below threshold
            if p.depth_left <= 2
                && thread.history_score[p.game_state.color_to_move][mv.from as usize]
                    [mv.to as usize]
                    < 0
            {
                thread.search_statistics.add_history_pruned();
                index += 1;
                continue;
            }
        }

        //Step 14.7. Late move reductions. Compute reduction based on move type, node type and depth
        let reduction = if p.depth_left > 2
            && !incheck
            && (!isc || move_score < 0.)
            && index >= 2
            && (!root || index >= 5)
        {
            compute_lmr_reduction(&p, thread, &mv, index, isc || isp, &next_state)
        } else {
            0
        };

        //Step 14.8. Search the moves
        let mut following_score: i16;
        if p.depth_left <= 2 || !is_pv_node || index == 0 {
            //Step 14.8.1 Full move window. This is done in pv nodes when index == 0 or depth left <= 2, e.g. the first move. If we are in a pv node,
            // reduction is 0 and we really search the full window (without research). Else we are in a zero window, and the full window search is just
            // zero window again (with reduction). If the reduced zero window search raises alpha, research without reduction
            debug_assert!(!is_pv_node || reduction == 0);
            following_score = -principal_variation_search(
                CombinedSearchParameters::from(
                    -p.beta,
                    -p.alpha,
                    p.depth_left - 1 - reduction,
                    &next_state,
                    -p.color,
                    p.current_depth + 1,
                ),
                thread,
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
                    thread,
                );
            }
        } else {
            //We are in a pv node and search with zero window all moves except the first (and with reduction). If
            // the reduced zero window search raises alpha, research
            following_score = -principal_variation_search(
                CombinedSearchParameters::from(
                    -p.alpha - 1,
                    -p.alpha,
                    p.depth_left - 1 - reduction,
                    &next_state,
                    -p.color,
                    p.current_depth + 1,
                ),
                thread,
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
                    thread,
                );
            }
        }

        //Step 14.9. Update principal variation if move raised current best moves score (does not have to raise alpha)
        // Also update UCI pv
        if following_score > current_max_score && !thread.self_stop {
            thread.pv_table[p.current_depth].pv[0] = Some(mv);
            current_max_score = following_score;
            concatenate_pv(p.current_depth, thread);
            uci_report_pv(&p, thread, following_score);
        }

        //Step 14.10. Update alpha if score raises alpha
        if following_score > p.alpha {
            p.alpha = following_score;
        }

        //Step 14.11. Beta cutoff: update several history statistics, and killer moves, then break
        if p.alpha >= p.beta {
            thread.search_statistics.add_normal_node_beta_cutoff(index);
            if !isc {
                update_quiet_cutoff(&p, thread, &mv, quiets_tried);
            }
            break;
        } else if !isc {
            //Step 14.12 Move does not cause beta cutoff, add to quiet moves tried and update butterfly heuristic
            thread.quiets_tried[p.current_depth][quiets_tried] = Some(mv);
            quiets_tried += 1;
            thread.bf_score[p.game_state.color_to_move][mv.from as usize][mv.to as usize] +=
                p.depth_left as usize * p.depth_left as usize;
            //TODO: Update bf should maybe also be done in decrement history quiets
        }

        index += 1;
    }

    thread.history.pop();

    //Step 15. Evaluate leafs correctly
    let game_status = check_end_condition(p.game_state, moves_tried > 0, incheck);
    if game_status != GameResult::Ingame {
        clear_pv(p.current_depth, thread);
        return leaf_score(game_status, p.color, p.current_depth as i16);
    }

    if p.alpha < p.beta {
        thread.search_statistics.add_normal_node_non_beta_cutoff();
    }

    //Step 16. Make TT Entry
    if !thread.self_stop {
        thread.itcs.cache.insert(
            &p,
            &thread.pv_table[p.current_depth].pv[0].expect("Can't unwrap move for TT"),
            current_max_score,
            original_alpha,
            thread.root_plies_played,
            static_evaluation,
        );
    }

    //Step 17. Return
    current_max_score
}

#[inline(always)]
pub fn uci_report_move(
    p: &CombinedSearchParameters,
    thread: &mut Thread,
    mv: &GameMove,
    index: usize,
) {
    if p.current_depth == 0 && thread.itcs.get_time_elapsed() > 1000 {
        println!(
            "info depth {} currmove {:?} currmovenumber {}",
            p.depth_left,
            mv,
            (index + 1)
        );
    }
}

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
pub fn max_depth(p: &CombinedSearchParameters, thread: &mut Thread) -> SearchInstruction {
    if p.current_depth >= (MAX_SEARCH_DEPTH - 1) {
        thread.attack_container.attack_containers[p.current_depth].write_state(p.game_state);
        SearchInstruction::StopSearching(
            eval_game_state(
                p.game_state,
                &thread.attack_container.attack_containers[p.current_depth],
                p.alpha * p.color,
                p.beta * p.color,
            )
            .final_eval
                * p.color,
        )
    } else {
        SearchInstruction::ContinueSearching
    }
}

#[inline(always)]
pub fn get_pvtable_move(p: &CombinedSearchParameters, thread: &Thread) -> Option<GameMove> {
    //PV-Table lookup
    if thread.pv_applicable.len() > (p.current_depth + 1)
        && thread.pv_applicable[p.current_depth] == p.game_state.hash
    {
        if thread.current_pv.pv.pv[p.current_depth].is_none() {
            println!("Error will occur in thread {}", thread.id);
            println!("We are in depth {}", p.current_depth);
        }
        return Some(
            thread.current_pv.pv.pv[p.current_depth]
                .expect("Unable to unwrap pv! get_pvtable_move"),
        );
    }
    None
}

#[inline(always)]
pub fn make_eval(
    p: &CombinedSearchParameters,
    thread: &mut Thread,
    static_evaluation: &mut Option<i16>,
    prunable: bool,
) {
    if static_evaluation.is_none()
        && (prunable
            && (p.depth_left <= STATIC_NULL_MOVE_DEPTH || p.depth_left >= NULL_MOVE_PRUNING_DEPTH)
            || p.depth_left <= FUTILITY_DEPTH)
    {
        let eval_res = eval_game_state(
            p.game_state,
            &thread.attack_container.attack_containers[p.current_depth],
            p.alpha * p.color,
            p.beta * p.color,
        );
        *static_evaluation = Some(eval_res.final_eval);
        thread.search_statistics.add_static_eval_node();
    }
}

#[inline(always)]
pub fn static_null_move_pruning(
    p: &CombinedSearchParameters,
    thread: &mut Thread,
    static_evaluation: Option<i16>,
) -> SearchInstruction {
    if p.depth_left <= STATIC_NULL_MOVE_DEPTH
        && static_evaluation.expect("Static null move") * p.color
            - STATIC_NULL_MOVE_MARGIN * p.depth_left
            >= p.beta
    {
        thread.history.pop();
        thread.search_statistics.add_static_null_move_node();
        SearchInstruction::StopSearching(
            static_evaluation.expect("Static null move 2") * p.color
                - STATIC_NULL_MOVE_DEPTH * p.depth_left,
        )
    } else {
        SearchInstruction::ContinueSearching
    }
}

#[inline(always)]
pub fn null_move_pruning(
    p: &CombinedSearchParameters,
    thread: &mut Thread,
    static_evaluation: Option<i16>,
) -> SearchInstruction {
    if p.depth_left >= NULL_MOVE_PRUNING_DEPTH
        && p.game_state.has_non_pawns(p.game_state.color_to_move)
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
            thread,
        );
        if rat >= p.beta {
            thread.search_statistics.add_nm_pruning();
            thread.history.pop();
            return SearchInstruction::StopSearching(rat);
        }
    }
    SearchInstruction::ContinueSearching
}

#[inline(always)]
pub fn internal_iterative_deepening(
    p: &CombinedSearchParameters,
    thread: &mut Thread,
    tt_move: &mut Option<GameMove>,
) -> SearchInstruction {
    thread.history.pop();
    principal_variation_search(
        CombinedSearchParameters::from(
            p.alpha,
            p.beta,
            p.depth_left - 2,
            &p.game_state,
            p.color,
            p.current_depth,
        ),
        thread,
    );
    thread.search_statistics.add_iid_node();
    if thread.self_stop {
        return SearchInstruction::StopSearching(STANDARD_SCORE);
    }
    thread
        .history
        .push(p.game_state.hash, p.game_state.half_moves == 0);
    *tt_move = thread.pv_table[p.current_depth].pv[0];
    SearchInstruction::ContinueSearching
}

#[inline(always)]
pub fn prepare_futility_pruning(
    p: &CombinedSearchParameters,
    static_evaluation: Option<i16>,
) -> (i16) {
    let futil_pruning = p.depth_left <= FUTILITY_DEPTH && p.current_depth > 0;
    if futil_pruning {
        static_evaluation.expect("Futil pruning") * p.color + p.depth_left * FUTILITY_MARGIN
    } else {
        MATE_SCORE
    }
}

#[inline(always)]
pub fn prepare_staged_movegen(
    p: &CombinedSearchParameters,
    thread: &mut Thread,
    has_generated_moves: bool,
    pv_table_move: &Option<GameMove>,
    tt_move: &Option<GameMove>,
) -> usize {
    if !has_generated_moves {
        thread.movelist.move_lists[p.current_depth].counter = 0;
    }
    let mut hash_and_pv_move_counter = 0;
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
    hash_and_pv_move_counter
}

#[inline(always)]
pub fn is_duplicate(
    mv: &GameMove,
    pv_table_move: &Option<GameMove>,
    tt_move: &Option<GameMove>,
) -> SearchInstruction {
    if let Some(pv_move) = pv_table_move.as_ref() {
        if *mv == *pv_move {
            return SearchInstruction::SkipMove;
        }
    }
    if let Some(tt_mv) = tt_move.as_ref() {
        if *mv == *tt_mv {
            return SearchInstruction::SkipMove;
        }
    }
    SearchInstruction::ContinueSearching
}

#[inline(always)]
pub fn select_next_move(
    p: &CombinedSearchParameters,
    thread: &mut Thread,
    moves_tried: usize,
    moves_from_movelist_tried: usize,
    hash_and_pv_move_counter: usize,
    tt_move: &Option<GameMove>,
    pv_table_move: &Option<GameMove>,
) -> (GameMove, f64) {
    let mut move_score = 0.;
    let mv: GameMove = if moves_tried < hash_and_pv_move_counter {
        if moves_tried == 0 {
            if let Some(pvmv) = pv_table_move {
                *pvmv
            } else {
                tt_move.expect("Moves tried ==0 and no pv move, couldn't unwrap even tt move")
            }
        } else {
            tt_move.expect("Moves tried >0 and no tt move")
        }
    } else {
        let available_moves = thread.movelist.move_lists[p.current_depth].counter;
        let r = get_next_gm(
            &mut thread.movelist.move_lists[p.current_depth],
            moves_from_movelist_tried,
            available_moves,
        );
        move_score = r.1;
        thread.movelist.move_lists[p.current_depth].move_list[r.0].unwrap()
    };
    (mv, move_score)
}

#[inline(always)]
pub fn compute_lmr_reduction(
    p: &CombinedSearchParameters,
    thread: &Thread,
    mv: &GameMove,
    index: usize,
    iscp: bool,
    next_state: &GameState,
) -> i16 {
    let mut reduction = ((f64::from(p.depth_left) / 2. - 1.).max(0.).sqrt()
        + (index as f64 / 2.0 - 1.).max(0.).sqrt()) as i16;
    if iscp {
        reduction /= 2;
    }
    if p.beta - p.alpha > 1 {
        reduction = (f64::from(reduction) * 0.66) as i16;
    }
    if in_check_slow(&next_state) {
        reduction -= 1;
    }
    if thread.history_score[p.game_state.color_to_move][mv.from as usize][mv.to as usize] > 0 {
        reduction -= 1;
    }
    reduction = reduction.min(p.depth_left - 1);
    reduction.max(1)
}

#[inline(always)]
pub fn uci_report_pv(p: &CombinedSearchParameters, thread: &mut Thread, following_score: i16) {
    if p.current_depth == 0 {
        thread.replace_current_pv(
            p.game_state,
            ScoredPrincipalVariation {
                pv: thread.pv_table[0].clone(),
                score: following_score,
                depth: p.depth_left as usize,
            },
        );
    }
}

#[inline(always)]
pub fn update_quiet_cutoff(
    p: &CombinedSearchParameters,
    thread: &mut Thread,
    mv: &GameMove,
    quiets_tried: usize,
) {
    thread.hh_score[p.game_state.color_to_move][mv.from as usize][mv.to as usize] +=
        p.depth_left as usize * p.depth_left as usize;
    thread.history_score[p.game_state.color_to_move][mv.from as usize][mv.to as usize] +=
        p.depth_left as isize * p.depth_left as isize;
    decrement_history_quiets(
        thread,
        p.current_depth,
        quiets_tried,
        p.depth_left as isize,
        p.game_state.color_to_move,
    );
    if let Some(s) = thread.killer_moves[p.current_depth][0] {
        if *mv == s {
            return;
        }
    }
    if let Some(s) = thread.killer_moves[p.current_depth][1] {
        if *mv == s {
            return;
        }
    }
    if let Some(s) = thread.killer_moves[p.current_depth][0] {
        thread.killer_moves[p.current_depth][1] = Some(s);
    }
    thread.killer_moves[p.current_depth][0] = Some(*mv);
}

pub fn decrement_history_quiets(
    thread: &mut Thread,
    current_depth: usize,
    quiets_tried: usize,
    depth_left: isize,
    side_to_move: usize,
) {
    for i in 0..quiets_tried {
        let mv = thread.quiets_tried[current_depth][i].as_ref().unwrap();
        thread.history_score[side_to_move][mv.from as usize][mv.to as usize] -=
            depth_left * depth_left;
    }
}

#[inline(always)]
pub fn make_and_evaluate_moves(game_state: &GameState, thread: &mut Thread, current_depth: usize) {
    movegen::generate_moves(
        &game_state,
        false,
        &mut thread.movelist.move_lists[current_depth],
        &thread.attack_container.attack_containers[current_depth],
    );
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
    let move_list = &mut thread.movelist.move_lists[current_depth];
    while mv_index < move_list.counter {
        let mv: &GameMove = move_list.move_list[mv_index].as_ref().unwrap();
        if is_capture(mv) {
            if GameMoveType::EnPassant == mv.move_type {
                move_list.graded_moves[mv_index] = Some(GradedMove::new(mv_index, 9999.0));
            } else {
                let mut sval = f64::from(see(&game_state, &mv, true, &mut thread.see_buffer));
                if sval >= 0.0 {
                    sval += 10000.0;
                }
                move_list.graded_moves[mv_index] = Some(GradedMove::new(mv_index, sval));
            }
        } else {
            //Assing history score
            let score = thread.hh_score[game_state.color_to_move][mv.from as usize][mv.to as usize]
                as f64
                / thread.bf_score[game_state.color_to_move][mv.from as usize][mv.to as usize]
                    as f64
                / 1000.0;
            move_list.graded_moves[mv_index] = Some(GradedMove::new(mv_index, score));
        }
        mv_index += 1;
    }

    {
        //Killer moves
        if let Some(s) = thread.killer_moves[current_depth][0] {
            let mv_index = find_move(&s, move_list, false);
            if mv_index < move_list.counter {
                move_list.graded_moves[mv_index].as_mut().unwrap().score += 5000.0;
            }
        }
        if let Some(s) = thread.killer_moves[current_depth][1] {
            let mv_index = find_move(&s, move_list, false);
            if mv_index < move_list.counter {
                move_list.graded_moves[mv_index].as_mut().unwrap().score += 5000.0;
            }
        }
    }
}
