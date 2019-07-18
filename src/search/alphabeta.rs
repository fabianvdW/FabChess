use super::super::board_representation::game_state::{GameMove, GameMoveType, GameResult};
use super::super::movegen;
use super::super::movegen::MoveList;
use super::super::GameState;
use super::cache::{Cache, CacheEntry};
use super::history::History;
use super::quiescence::{is_capture, q_search, see};
use super::search::Search;
use super::search::SearchUtils;
use super::GradedMove;
use crate::evaluation::eval_game_state;
use crate::move_generation::makemove::{make_move, make_nullmove};
use std::fmt::{Display, Formatter, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const MATE_SCORE: i16 = 15000;
pub const MATED_IN_MAX: i16 = -14000;
pub const HH_INCREMENT: usize = 1;
pub const BF_INCREMENT: usize = 1;
pub const MAX_SEARCH_DEPTH: usize = 100;
pub const STANDARD_SCORE: i16 = -32767;

//TODO Static Null Move Pruning, Delta Pruning
//Clean up arguments
//Additional Thoughts:
//Do most of the forward pruning before movegen
pub fn principal_variation_search(
    mut alpha: i16,
    mut beta: i16,
    mut depth_left: i16,
    game_state: &GameState,
    color: i16,
    current_depth: usize,
    su: &mut SearchUtils,
) -> i16 {
    su.search.search_statistics.add_normal_node(current_depth);
    clear_pv(current_depth, su.search);
    if su.search.search_statistics.nodes_searched % 1024 == 0 {
        checkup(su.search, su.stop);
    }
    if su.search.stop {
        return STANDARD_SCORE;
    }
    //Max search-depth reached
    if current_depth >= (MAX_SEARCH_DEPTH - 1) {
        return eval_game_state(&game_state, false).final_eval * color;
    }

    let root = current_depth == 0;
    //Check for draw
    if !root && check_for_draw(game_state, su.history) {
        return leaf_score(GameResult::Draw, color, depth_left);
    }
    let is_pv_node = beta - alpha > 1;
    let incheck = in_check(game_state);
    let is_likelystalemate = !incheck && is_likelystalemate(game_state);

    //Check Extensions and extend if you would drop into q search but estimate a stalemate
    if incheck && !root || depth_left == 0 && is_likelystalemate {
        depth_left += 1;
    }

    //Drop into quiescence search
    if depth_left <= 0 {
        su.search.search_statistics.add_q_root();
        return q_search(alpha, beta, &game_state, color, 0, current_depth, su);
    }

    let mut pv_table_move: Option<GameMove> = None;
    let mut has_pvmove = false;
    let mut tt_move: Option<GameMove> = None;
    let mut has_ttmove = false;

    //PV-Table lookup
    {
        if let Some(ce) = su.search.principal_variation[current_depth] {
            if ce.hash == game_state.hash {
                has_pvmove = true;
                let mv = CacheEntry::u16_to_mv(ce.mv, &game_state);
                pv_table_move = Some(mv);
            }
        }
    }

    //Probe TT
    let mut static_evaluation = None;
    {
        let ce = &su.cache.cache[game_state.hash as usize & super::cache::CACHE_MASK];
        if let Some(s) = ce {
            let ce: &CacheEntry = s;
            if ce.hash == game_state.hash {
                su.search.search_statistics.add_cache_hit_ns();
                if ce.depth >= depth_left as i8 {
                    if beta - alpha == 1 {
                        if !ce.alpha && !ce.beta {
                            su.search.search_statistics.add_cache_hit_replace_ns();
                            su.search.pv_table[current_depth].pv[0] =
                                Some(CacheEntry::u16_to_mv(ce.mv, &game_state));
                            return ce.score;
                        } else {
                            if ce.beta {
                                if ce.score > alpha {
                                    alpha = ce.score;
                                }
                            } else if ce.alpha {
                                if ce.score < beta {
                                    beta = ce.score;
                                }
                            }
                            if alpha >= beta {
                                su.search.search_statistics.add_cache_hit_aj_replace_ns();
                                su.search.pv_table[current_depth].pv[0] =
                                    Some(CacheEntry::u16_to_mv(ce.mv, &game_state));
                                return ce.score;
                            }
                        }
                    }
                }
                static_evaluation = ce.static_evaluation;
                let mv = CacheEntry::u16_to_mv(ce.mv, &game_state);
                tt_move = Some(mv);
                has_ttmove = true;
            }
        }
    }

    su.history.push(game_state.hash, game_state.half_moves == 0);

    //Static Null Move Pruning
    //Null Move Forward Pruning
    if !is_pv_node
        && !incheck
        && !is_likelystalemate
        && (game_state.pieces[1][game_state.color_to_move]
            | game_state.pieces[2][game_state.color_to_move]
            | game_state.pieces[3][game_state.color_to_move]
            | game_state.pieces[4][game_state.color_to_move])
            != 0u64
    {
        if let None = static_evaluation {
            static_evaluation = Some(eval_game_state(&game_state, false).final_eval);
        }
        if static_evaluation.unwrap() * color >= beta {
            let nextgs = make_nullmove(&game_state);
            let rat = -principal_variation_search(
                -beta,
                -beta + 1,
                (depth_left - 4 - depth_left / 6).max(0),
                &nextgs,
                -color,
                current_depth + 1,
                su,
            );
            if rat >= beta {
                su.search.search_statistics.add_nm_pruning();
                su.history.pop();
                return rat;
            }
        }
    }

    //Internal Iterative Deepening
    let mut has_generated_moves = false;
    if is_pv_node && !incheck && !is_likelystalemate && !has_pvmove && !has_ttmove && depth_left > 6
    {
        su.history.pop();
        principal_variation_search(
            alpha,
            beta,
            depth_left - 2,
            &game_state,
            color,
            current_depth,
            su,
        );
        su.history.push(game_state.hash, game_state.half_moves == 0);
        if su.search.stop {
            return STANDARD_SCORE;
        }
        tt_move = su.search.pv_table[current_depth].pv[0];
        has_ttmove = if let Some(_) = tt_move.as_ref() {
            true
        } else {
            false
        };
        has_generated_moves = true;
    }

    //Prepare Futility Pruning
    let mut futil_pruning = depth_left <= 8 && !incheck;
    let mut futil_margin = 0;
    if futil_pruning {
        if let None = static_evaluation {
            static_evaluation = Some(eval_game_state(&game_state, false).final_eval);
        }
        futil_margin = static_evaluation.unwrap() * color + depth_left * 90;
    }
    if !has_generated_moves {
        su.move_list.counter[current_depth] = 0;
    }
    let mut hash_and_pv_move_counter = 0;
    {
        if has_pvmove {
            hash_and_pv_move_counter += 1;
        }
        if has_ttmove && !has_pvmove {
            hash_and_pv_move_counter += 1;
        } else if has_ttmove {
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
    while moves_tried < su.move_list.counter[current_depth] + hash_and_pv_move_counter
        || !has_generated_moves
    {
        if moves_tried == hash_and_pv_move_counter && !has_generated_moves {
            has_generated_moves = true;
            make_and_evaluate_moves(game_state, su.search, current_depth, su.move_list);
            continue;
        }
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
            su.move_list.move_list[current_depth][get_next_gm(
                su.move_list,
                current_depth,
                moves_from_movelist_tried,
                su.move_list.counter[current_depth],
            )]
            .unwrap()
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

        let isc = is_capture(&mv);
        let isp = if let GameMoveType::Promotion(_, _) = mv.move_type {
            true
        } else {
            false
        };
        let next_state = make_move(&game_state, &mv);
        //--------------------------------------------------------------
        //Futility Pruning
        if futil_pruning
            && !isc
            && !isp
            && current_max_score > MATED_IN_MAX
            && !in_check(&next_state)
        {
            if futil_margin <= alpha {
                continue;
            } else {
                futil_pruning = false;
            }
        }
        let mut following_score: i16;
        let mut reduction = 0;
        if depth_left > 2
            && !has_pvmove
            && !incheck
            && !isc
            && index >= 2
            && !isp
            && !in_check(&next_state)
        {
            //FRUITED RELOADED REDUCTION! NEXT THREE LINES ARE COPIED:
            reduction = (((depth_left - 1) as f64).sqrt() + ((index - 1) as f64).sqrt()) as i16;
            if is_pv_node {
                reduction = (reduction as f64 * 0.66) as i16;
            }
            if reduction > depth_left - 2 {
                reduction = depth_left - 2
            }
        }
        if depth_left <= 2 || !has_pvmove || index == 0 {
            following_score = -principal_variation_search(
                -beta,
                -alpha,
                depth_left - 1 - reduction,
                &next_state,
                -color,
                current_depth + 1,
                su,
            );
            if reduction > 0 && following_score > alpha {
                following_score = -principal_variation_search(
                    -beta,
                    -alpha,
                    depth_left - 1,
                    &next_state,
                    -color,
                    current_depth + 1,
                    su,
                );
            }
        } else {
            following_score = -principal_variation_search(
                -alpha - 1,
                -alpha,
                depth_left - 1,
                &next_state,
                -color,
                current_depth + 1,
                su,
            );
            if following_score > alpha {
                following_score = -principal_variation_search(
                    -beta,
                    -alpha,
                    depth_left - 1,
                    &next_state,
                    -color,
                    current_depth + 1,
                    su,
                );
            }
        }

        if following_score > current_max_score {
            su.search.pv_table[current_depth].pv[0] = Some(mv);
            current_max_score = following_score;

            concatenate_pv(current_depth, su.search);
        }
        if following_score > alpha {
            alpha = following_score;
        }
        if alpha >= beta {
            su.search
                .search_statistics
                .add_normal_node_beta_cutoff(index);
            if !isc {
                su.search.hh_score[mv.from][mv.to] += HH_INCREMENT;
                //Replace killers
                //Dont replace if already in table
                if let Some(s) = su.search.killer_moves[current_depth][0] {
                    if s.from == mv.from && s.to == mv.to && s.move_type == mv.move_type {
                        break;
                    }
                }
                if let Some(s) = su.search.killer_moves[current_depth][1] {
                    if s.from == mv.from && s.to == mv.to && s.move_type == mv.move_type {
                        break;
                    }
                }
                if let Some(s) = su.search.killer_moves[current_depth][0] {
                    su.search.killer_moves[current_depth][1] = Some(s);
                }
                su.search.killer_moves[current_depth][0] = Some(mv);
            }
            break;
        } else {
            if !isc {
                su.search.bf_score[mv.from][mv.to] += BF_INCREMENT;
            }
        }
        index += 1;
    }

    su.history.pop();
    let game_status = check_end_condition(&game_state, moves_tried > 0, incheck);
    if game_status != GameResult::Ingame {
        clear_pv(current_depth, su.search);
        return leaf_score(game_status, color, depth_left);
    }

    if alpha < beta {
        su.search
            .search_statistics
            .add_normal_node_non_beta_cutoff();
    }
    //Make cache
    if !su.search.stop {
        make_cache(
            su.cache,
            &su.search.pv_table[current_depth],
            current_max_score,
            &game_state,
            alpha,
            beta,
            depth_left,
            su.root_pliesplayed,
            static_evaluation,
        );
    }
    return current_max_score;
}
#[inline(always)]
pub fn make_and_evaluate_moves(
    game_state: &GameState,
    search: &mut Search,
    current_depth: usize,
    move_list: &mut MoveList,
) {
    movegen::generate_moves2(&game_state, false, move_list, current_depth);
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
    while mv_index < move_list.counter[current_depth] {
        let mv: &GameMove = move_list.move_list[current_depth][mv_index]
            .as_ref()
            .unwrap();
        if is_capture(mv) {
            if GameMoveType::EnPassant == mv.move_type {
                move_list.graded_moves[current_depth][mv_index] =
                    Some(GradedMove::new(mv_index, 9999.0));
            } else {
                let mut sval = see(&game_state, &mv, true, &mut search.see_buffer) as f64;
                if sval >= 0.0 {
                    sval += 10000.0;
                }
                move_list.graded_moves[current_depth][mv_index] =
                    Some(GradedMove::new(mv_index, sval));
            }
        } else {
            //Assing history score
            let score = search.hh_score[mv.from][mv.to] as f64
                / search.bf_score[mv.from][mv.to] as f64
                / 1000.0;
            move_list.graded_moves[current_depth][mv_index] =
                Some(GradedMove::new(mv_index, score));
        }
        mv_index += 1;
    }

    {
        //Killer moves
        if let Some(s) = search.killer_moves[current_depth][0] {
            let mv_index = find_move(&s, move_list, current_depth, false);
            if mv_index < move_list.counter[current_depth] {
                move_list.graded_moves[current_depth][mv_index]
                    .as_mut()
                    .unwrap()
                    .score += 5000.0;
            }
        }
        if let Some(s) = search.killer_moves[current_depth][1] {
            let mv_index = find_move(&s, move_list, current_depth, false);
            if mv_index < move_list.counter[current_depth] {
                move_list.graded_moves[current_depth][mv_index]
                    .as_mut()
                    .unwrap()
                    .score += 5000.0;
            }
        }
    }
}
#[inline(always)]
pub fn is_likelystalemate(game_state: &GameState) -> bool {
    if (game_state.pieces[2][0]
        | game_state.pieces[2][1]
        | game_state.pieces[3][0]
        | game_state.pieces[3][1]
        | game_state.pieces[4][0]
        | game_state.pieces[4][1])
        != 0u64
    {
        return false;
    }
    //Else calculate all legal moves
    let my_pieces = game_state.pieces[0][game_state.color_to_move]
        | game_state.pieces[1][game_state.color_to_move]
        | game_state.pieces[2][game_state.color_to_move]
        | game_state.pieces[3][game_state.color_to_move]
        | game_state.pieces[4][game_state.color_to_move]
        | game_state.pieces[5][game_state.color_to_move];
    let enemy_pieces = game_state.pieces[0][1 - game_state.color_to_move]
        | game_state.pieces[1][1 - game_state.color_to_move]
        | game_state.pieces[2][1 - game_state.color_to_move]
        | game_state.pieces[3][1 - game_state.color_to_move]
        | game_state.pieces[4][1 - game_state.color_to_move]
        | game_state.pieces[5][1 - game_state.color_to_move];
    let mut my_knights = game_state.pieces[1][game_state.color_to_move];
    while my_knights != 0u64 {
        let idx = my_knights.trailing_zeros() as usize;
        if movegen::knight_attack(idx) & !my_pieces != 0u64 {
            return false;
        }
        my_knights ^= 1u64 << idx;
    }
    if movegen::king_attack(
        game_state.pieces[5][game_state.color_to_move].trailing_zeros() as usize,
    ) & !my_pieces
        != 0u64
    {
        return false;
    }
    if game_state.color_to_move == 0 {
        if movegen::w_pawn_west_targets(game_state.pieces[0][0])
            | movegen::w_pawn_east_targets(game_state.pieces[0][0])
                & (game_state.en_passant | enemy_pieces)
            != 0u64
        {
            return false;
        }
        if movegen::w_single_push_pawn_targets(game_state.pieces[0][0], !my_pieces & !enemy_pieces)
            != 0u64
        {
            return false;
        }
    } else {
        if movegen::b_pawn_west_targets(game_state.pieces[0][1])
            | movegen::b_pawn_east_targets(game_state.pieces[0][1])
                & (game_state.en_passant | enemy_pieces)
            != 0u64
        {
            return false;
        }
        if movegen::b_single_push_pawn_targets(game_state.pieces[0][1], !my_pieces & !enemy_pieces)
            != 0u64
        {
            return false;
        }
    }
    true
}
#[inline(always)]
pub fn clear_pv(at_depth: usize, search: &mut Search) {
    let mut index = 0;
    while let Some(_) = search.pv_table[at_depth].pv[index].as_ref() {
        search.pv_table[at_depth].pv[index] = None;
        index += 1;
    }
}

#[inline(always)]
pub fn concatenate_pv(at_depth: usize, search: &mut Search) {
    let mut index = 0;
    while let Some(mv) = search.pv_table[at_depth + 1].pv[index].as_ref() {
        search.pv_table[at_depth].pv[index + 1] = Some(mv.clone());
        index += 1;
    }
    while let Some(_) = search.pv_table[at_depth].pv[index + 1].as_ref() {
        search.pv_table[at_depth].pv[index + 1] = None;
        index += 1;
    }
}

#[inline(always)]
pub fn in_check(game_state: &GameState) -> bool {
    let my_king = game_state.pieces[5][game_state.color_to_move];
    if (movegen::knight_attack(my_king.trailing_zeros() as usize)
        & game_state.pieces[1][1 - game_state.color_to_move])
        != 0u64
    {
        return true;
    }
    if game_state.color_to_move == 0 {
        if (movegen::w_pawn_west_targets(my_king) | movegen::w_pawn_east_targets(my_king))
            & game_state.pieces[0][1 - game_state.color_to_move]
            != 0u64
        {
            return true;
        }
    } else {
        if (movegen::b_pawn_west_targets(my_king) | movegen::b_pawn_east_targets(my_king))
            & game_state.pieces[0][1 - game_state.color_to_move]
            != 0u64
        {
            return true;
        }
    }
    let all_pieces = game_state.pieces[0][game_state.color_to_move]
        | game_state.pieces[1][game_state.color_to_move]
        | game_state.pieces[2][game_state.color_to_move]
        | game_state.pieces[3][game_state.color_to_move]
        | game_state.pieces[4][game_state.color_to_move]
        | game_state.pieces[0][1 - game_state.color_to_move]
        | game_state.pieces[1][1 - game_state.color_to_move]
        | game_state.pieces[2][1 - game_state.color_to_move]
        | game_state.pieces[3][1 - game_state.color_to_move]
        | game_state.pieces[4][1 - game_state.color_to_move]
        | game_state.pieces[5][1 - game_state.color_to_move];
    if movegen::bishop_attack(my_king.trailing_zeros() as usize, all_pieces)
        & (game_state.pieces[2][1 - game_state.color_to_move]
            | game_state.pieces[4][1 - game_state.color_to_move])
        != 0u64
    {
        return true;
    }
    if movegen::rook_attack(my_king.trailing_zeros() as usize, all_pieces)
        & (game_state.pieces[3][1 - game_state.color_to_move]
            | game_state.pieces[4][1 - game_state.color_to_move])
        != 0u64
    {
        return true;
    }
    false
}

#[inline(always)]
pub fn checkup(search: &mut Search, stop: &Arc<AtomicBool>) {
    search.search_statistics.refresh_time_elapsed();
    if search.tc.time_over(
        search.search_statistics.time_elapsed,
        &search.tc_information,
    ) || stop.load(Ordering::Relaxed)
    {
        search.stop = true;
        //println!("{}", search.search_statistics);
    }
}

#[inline(always)]
pub fn get_next_gm(
    mv_list: &mut MoveList,
    current_depth: usize,
    mv_index: usize,
    max_moves: usize,
) -> usize {
    if mv_list.counter[current_depth] == 0 {
        panic!("List has to be longer than 1")
    } else {
        let mut index = mv_index;
        for i in (mv_index + 1)..max_moves {
            if mv_list.graded_moves[current_depth][i]
                .as_ref()
                .unwrap()
                .score
                > mv_list.graded_moves[current_depth][index]
                    .as_ref()
                    .unwrap()
                    .score
            {
                index = i;
            }
        }
        let result = mv_list.graded_moves[current_depth][index]
            .as_ref()
            .unwrap()
            .mv_index;
        mv_list.graded_moves[current_depth][index] =
            mv_list.graded_moves[current_depth][mv_index].clone();
        result
    }
}

#[inline(always)]
pub fn find_move(mv: &GameMove, mv_list: &MoveList, current_depth: usize, contains: bool) -> usize {
    let mut mv_index = 0;
    while mv_index < mv_list.counter[current_depth] {
        let mvs = mv_list.move_list[current_depth][mv_index].as_ref().unwrap();
        if mvs.from == mv.from && mvs.to == mv.to && mvs.move_type == mv.move_type {
            break;
        }
        mv_index += 1;
    }
    if mv_index < mv_list.counter[current_depth] {
        return mv_index;
    } else if contains {
        panic!("Type 2 error");
    } else {
        return mv_index;
    }
}

#[inline(always)]
pub fn make_cache(
    cache: &mut Cache,
    pv: &PrincipalVariation,
    score: i16,
    game_state: &GameState,
    original_alpha: i16,
    beta: i16,
    depth_left: i16,
    root_plies_played: usize,
    static_evaluation: Option<i16>,
) {
    let beta_node: bool = score >= beta;
    let alpha_node: bool = score < original_alpha;

    let index = game_state.hash as usize & super::cache::CACHE_MASK;

    let ce = &cache.cache[game_state.hash as usize & super::cache::CACHE_MASK];
    let new_entry_val = depth_left as f64 * if beta_node || alpha_node { 0.7 } else { 1.0 };
    if let None = ce {
        let new_entry = CacheEntry::new(
            &game_state,
            depth_left,
            score,
            alpha_node,
            beta_node,
            match pv.pv[0].as_ref() {
                Some(mv) => &mv,
                _ => panic!("Invalid pv!"),
            },
            static_evaluation,
        );
        cache.cache[index] = Some(new_entry);
    } else {
        let old_entry: &CacheEntry = match ce {
            Some(s) => s,
            _ => panic!("Invalid if let!"),
        };
        //Make replacement scheme better
        let mut old_entry_val = old_entry.depth as f64
            * if old_entry.beta || old_entry.alpha {
                0.7
            } else {
                1.0
            };
        if old_entry.plies_played < root_plies_played as u16 {
            old_entry_val = -1.0;
        }
        if old_entry_val <= new_entry_val {
            let new_entry = CacheEntry::new(
                &game_state,
                depth_left,
                score,
                alpha_node,
                beta_node,
                match pv.pv[0].as_ref() {
                    Some(mv) => &mv,
                    _ => panic!("Invalid pv!"),
                },
                static_evaluation,
            );
            cache.cache[index] = Some(new_entry);
        }
    }
}

#[inline(always)]
pub fn leaf_score(game_status: GameResult, color: i16, depth_left: i16) -> i16 {
    if game_status == GameResult::Draw {
        return 0;
    } else if game_status == GameResult::WhiteWin {
        return (MATE_SCORE + depth_left) * color;
    } else if game_status == GameResult::BlackWin {
        return (MATE_SCORE + depth_left) * -color;
    }
    panic!("Invalid Leaf");
}

//Doesn't actually check for stalemate
#[inline(always)]
pub fn check_for_draw(game_state: &GameState, history: &History) -> bool {
    if game_state.pieces[0][0]
        | game_state.pieces[1][0]
        | game_state.pieces[2][0]
        | game_state.pieces[3][0]
        | game_state.pieces[4][0]
        | game_state.pieces[0][1]
        | game_state.pieces[1][1]
        | game_state.pieces[2][1]
        | game_state.pieces[3][1]
        | game_state.pieces[4][1]
        == 0u64
    {
        return true;
    }
    if game_state.half_moves >= 100 {
        return true;
    }

    if history.get_occurences(game_state) >= 1 {
        return true;
    }
    false
}
#[inline(always)]
pub fn check_end_condition(
    game_state: &GameState,
    has_legal_moves: bool,
    in_check: bool,
) -> GameResult {
    if in_check && !has_legal_moves {
        if game_state.color_to_move == 0 {
            return GameResult::BlackWin;
        } else {
            return GameResult::WhiteWin;
        }
    }
    if !in_check && !has_legal_moves {
        return GameResult::Draw;
    }
    GameResult::Ingame
}

pub struct PrincipalVariation {
    pub pv: Vec<Option<GameMove>>,
}

impl PrincipalVariation {
    pub fn new(depth_left: usize) -> PrincipalVariation {
        PrincipalVariation {
            pv: vec![None; depth_left + 1],
        }
    }
}

impl Display for PrincipalVariation {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!("PV of length {}\n", self.pv.len(),));
        for mv in &self.pv {
            res_str.push_str(&format!("{:?}\n", mv));
        }
        write!(formatter, "{}", res_str)
    }
}
