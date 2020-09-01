use super::super::board_representation::game_state::*;
use super::quiescence::q_search;
use super::*;
use super::{MATE_SCORE, MAX_SEARCH_DEPTH, STANDARD_SCORE};
use crate::evaluation::eval_game_state;
use crate::move_generation::makemove::{make_move, make_nullmove};
use crate::search::cache::{CacheEntry, INVALID_STATIC_EVALUATION};
use crate::search::moveordering::{MoveOrderer, NORMAL_STAGES};
use crate::search::quiescence::{piece_value, see};
use crate::search::searcher::Thread;

pub const LMP_DEPTH: usize = 4;
pub const FUTILITY_MARGIN: i16 = 90;
pub const FUTILITY_DEPTH: i16 = 6;
pub const STATIC_NULL_MOVE_MARGIN: i16 = 120;
pub const STATIC_NULL_MOVE_DEPTH: i16 = 5;
pub const NULL_MOVE_PRUNING_DEPTH: i16 = 3;
pub const HISTORY_PRUNING_DEPTH: i16 = 2;
pub const HISTORY_PRUNING_THRESHOLD: isize = 0;
pub const SEE_PRUNING_DEPTH: i16 = 6;
pub const SEE_PRUNING_CAPTURE_MULT: f64 = -23.;
pub const SEE_PRUNING_QUIET_MULT: f64 = -23.;

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
    if let SearchInstruction::StopSearching(res) = max_depth(&p) {
        return res;
    }

    //Step 3. Check for draw or mate distance pruning if not root (need best move at root)
    if !root {
        if let SearchInstruction::StopSearching(r) = check_for_draw(p.game_state, &thread.history) {
            return r;
        }
        //Mate distance pruning
        if let SearchInstruction::StopSearching(res) = mate_distance_pruning(&mut p) {
            return res;
        }
    }
    let original_alpha = p.alpha;
    //Reset killer moves for granchildren
    if p.current_depth + 2 < thread.killer_moves.len() {
        thread.killer_moves[p.current_depth + 2][0] = None;
        thread.killer_moves[p.current_depth + 2][1] = None;
    }

    //Step 4. Attacks and in check  flag
    let incheck = p.game_state.in_check();

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
    //TODO Correctly insert and retrieve mates into the TT
    let mut tt_entry: Option<CacheEntry> = None;
    if let SearchInstruction::StopSearching(res) = thread.itcs.cache().lookup(&p, &mut tt_entry) {
        #[cfg(feature = "search-statistics")]
        {
            thread.search_statistics.add_cache_hit_aj_replace_ns();
        }
        return res;
    }
    #[cfg(feature = "search-statistics")]
    {
        if tt_move.is_some() {
            thread.search_statistics.add_cache_hit_ns();
        }
    }
    let mut tt_move = if let Some(ce) = tt_entry {
        Some(CacheEntry::u16_to_mv(ce.mv, p.game_state))
    } else {
        None
    };
    let mut static_evaluation = if let Some(ce) = tt_entry {
        if ce.static_evaluation != INVALID_STATIC_EVALUATION {
            Some(ce.static_evaluation)
        } else {
            None
        }
    } else {
        None
    };
    thread
        .history
        .push(p.game_state.get_hash(), p.game_state.get_half_moves() == 0);

    //Step 9. Static Eval if needed
    let prunable = !is_pv_node && !incheck;
    make_eval(&p, &mut static_evaluation, prunable);

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
            null_move_pruning(&p, thread, static_evaluation, &tt_entry)
        {
            return res;
        }
    }

    //Step 11. Internal Iterative Deepening
    if is_pv_node && !incheck && pv_table_move.is_none() && tt_move.is_none() && p.depth_left > 6 {
        if let SearchInstruction::StopSearching(res) =
            internal_iterative_deepening(&p, thread, &mut tt_move)
        {
            return res;
        }
    }

    //Step 12. Futil Pruning and margin preparation
    let futil_margin = prepare_futility_pruning(&p, static_evaluation);

    //Step 14. Iterate through all moves
    let mut current_max_score = STANDARD_SCORE;
    let mut index: usize = 0;
    let mut quiets_tried: usize = 0;
    let mut search_quiets = true;
    let mut move_orderer = MoveOrderer {
        stage: 0,
        stages: &NORMAL_STAGES,
        gen_only_captures: false,
    };
    loop {
        let mv = move_orderer.next(thread, &p, pv_table_move, tt_move, search_quiets);
        if mv.is_none() {
            break;
        }
        let (mv, move_score) = mv.unwrap(); //Move score is only set for bad_capture

        //Step 14.4. UCI Reporting at root
        //uci_report_move(&p, su, &mv, index);

        let isc = mv.is_capture();
        let isp = if let GameMoveType::Promotion(_, _) = mv.move_type {
            true
        } else {
            false
        };
        let is_quiet_move = !isc && !isp;
        let gives_check = p.game_state.gives_check(mv);

        if !root
            && is_quiet_move
            && current_max_score > MATED_IN_MAX
            && p.game_state.has_non_pawns(p.game_state.get_color_to_move())
            && !gives_check
        {
            //Step 14.5. Futility Pruning. Skip quiet moves if futil_margin can't raise alpha
            if futil_margin <= p.alpha {
                #[cfg(feature = "search-statistics")]
                {
                    thread.search_statistics.add_futil_pruning();
                }
                index += 1;
                search_quiets = false;
                continue;
            }
            //Step 14.6. History Pruning. Skip quiet moves in low depths if they are below threshold
            if p.depth_left <= HISTORY_PRUNING_DEPTH
                && thread.history_score[p.game_state.get_color_to_move()][mv.from as usize]
                    [mv.to as usize]
                    < HISTORY_PRUNING_THRESHOLD
            {
                #[cfg(feature = "search-statistics")]
                {
                    thread.search_statistics.add_history_pruned();
                }
                index += 1;
                continue;
            }

            if !incheck
                && p.depth_left <= LMP_DEPTH as i16
                && quiets_tried > (3 * 2u32.pow((p.depth_left - 1) as u32)) as usize
            {
                index += 1;
                search_quiets = false;
                continue;
            }
            //Step 14.7 SEE Pruning. Skip quiet moves which have negative SEE Score on low depths
            let margin =
                (SEE_PRUNING_QUIET_MULT * (p.depth_left as f64 * p.depth_left as f64)) as i16;
            if p.depth_left <= SEE_PRUNING_DEPTH && -piece_value(mv.piece_type) < margin {
                let see_value = see(p.game_state, mv, true, &mut thread.see_buffer);
                if see_value < margin {
                    index += 1;
                    continue;
                }
            }
        } else if !root
            && isc
            && current_max_score > MATED_IN_MAX
            && p.depth_left <= SEE_PRUNING_DEPTH
            && move_score < SEE_PRUNING_CAPTURE_MULT * p.depth_left as f64 * p.depth_left as f64
            && p.game_state.has_non_pawns(p.game_state.get_color_to_move())
            && !gives_check
        {
            index += 1;
            continue;
        }

        //Step 14.7. Late move reductions. Compute reduction based on move type, node type and depth
        let reduction =
            if p.depth_left > 2 && (!isc || move_score < 0.) && index >= 2 && (!root || index >= 5)
            {
                compute_lmr_reduction(&p, thread, mv, index, isc || isp, gives_check, incheck)
            } else {
                0
            };

        let next_state = make_move(p.game_state, mv);
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
            uci_report_pv(
                &p,
                thread,
                following_score,
                following_score > original_alpha,
            );
        }

        //Step 14.10. Update alpha if score raises alpha
        if following_score > p.alpha {
            p.alpha = following_score;
        }

        //Step 14.11. Beta cutoff: update several history statistics, and killer moves, then break
        if p.alpha >= p.beta {
            #[cfg(feature = "search-statistics")]
            {
                thread.search_statistics.add_normal_node_beta_cutoff(index);
            }
            if !isc {
                update_quiet_cutoff(&p, thread, mv, quiets_tried);
            }
            break;
        } else if !isc {
            //Step 14.12 Move does not cause beta cutoff, add to quiet moves tried and update butterfly heuristic
            thread.quiets_tried[p.current_depth][quiets_tried] = Some(mv);
            quiets_tried += 1;
            thread.bf_score[p.game_state.get_color_to_move()][mv.from as usize][mv.to as usize] +=
                p.depth_left as usize * p.depth_left as usize;
            //TODO: Update bf should maybe also be done in decrement history quiets
        }

        index += 1;
    }

    thread.history.pop();

    //Step 15. Evaluate leafs correctly
    let game_status =
        check_end_condition(p.game_state, current_max_score > STANDARD_SCORE, incheck);
    if game_status != GameResult::Ingame {
        clear_pv(p.current_depth, thread);
        return leaf_score(game_status, p.color, p.current_depth as i16);
    }
    #[cfg(feature = "search-statistics")]
    {
        if p.alpha < p.beta {
            thread.search_statistics.add_normal_node_non_beta_cutoff();
        }
    }

    //Step 16. Make TT Entry
    if !thread.self_stop {
        thread.itcs.cache().insert(
            &p,
            thread.pv_table[p.current_depth].pv[0].expect("Can't unwrap move for TT"),
            current_max_score,
            original_alpha,
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
    mv: GameMove,
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
pub fn max_depth(p: &CombinedSearchParameters) -> SearchInstruction {
    if p.current_depth >= (MAX_SEARCH_DEPTH - 1) {
        SearchInstruction::StopSearching(eval_game_state(p.game_state).final_eval * p.color)
    } else {
        SearchInstruction::ContinueSearching
    }
}

#[inline(always)]
pub fn get_pvtable_move(p: &CombinedSearchParameters, thread: &Thread) -> Option<GameMove> {
    //PV-Table lookup
    if thread.pv_applicable.len() > (p.current_depth + 1)
        && thread.pv_applicable[p.current_depth] == p.game_state.get_hash()
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
    static_evaluation: &mut Option<i16>,
    prunable: bool,
) {
    if static_evaluation.is_none()
        && (prunable
            && (p.depth_left <= STATIC_NULL_MOVE_DEPTH || p.depth_left >= NULL_MOVE_PRUNING_DEPTH)
            || p.depth_left <= FUTILITY_DEPTH)
    {
        let eval_res = eval_game_state(p.game_state);
        *static_evaluation = Some(eval_res.final_eval);
        #[cfg(feature = "search-statistics")]
        {
            thread.search_statistics.add_static_eval_node();
        }
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
        #[cfg(feature = "search-statistics")]
        {
            thread.search_statistics.add_static_null_move_node();
        }
        SearchInstruction::StopSearching(static_evaluation.expect("Static null move 2") * p.color)
    } else {
        SearchInstruction::ContinueSearching
    }
}

#[inline(always)]
pub fn null_move_pruning(
    p: &CombinedSearchParameters,
    thread: &mut Thread,
    static_evaluation: Option<i16>,
    tt_entry: &Option<CacheEntry>,
) -> SearchInstruction {
    if p.depth_left >= NULL_MOVE_PRUNING_DEPTH
        && p.game_state.has_non_pawns(p.game_state.get_color_to_move())
        && static_evaluation.expect("null move static") * p.color >= p.beta
        && (tt_entry.is_none()
            || !tt_entry.unwrap().is_lower_bound()
            || tt_entry.unwrap().score >= p.beta)
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
            #[cfg(feature = "search-statistics")]
            {
                thread.search_statistics.add_nm_pruning();
            }
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
    #[cfg(feature = "search-statistics")]
    {
        thread.search_statistics.add_iid_node();
    }
    if thread.self_stop {
        return SearchInstruction::StopSearching(STANDARD_SCORE);
    }
    thread
        .history
        .push(p.game_state.get_hash(), p.game_state.get_half_moves() == 0);
    *tt_move = thread.pv_table[p.current_depth].pv[0];
    SearchInstruction::ContinueSearching
}

#[inline(always)]
pub fn prepare_futility_pruning(
    p: &CombinedSearchParameters,
    static_evaluation: Option<i16>,
) -> i16 {
    let futil_pruning = p.depth_left <= FUTILITY_DEPTH && p.current_depth > 0;
    if futil_pruning {
        static_evaluation.expect("Futil pruning") * p.color + p.depth_left * FUTILITY_MARGIN
    } else {
        MATE_SCORE
    }
}

#[inline(always)]
pub fn compute_lmr_reduction(
    p: &CombinedSearchParameters,
    thread: &Thread,
    mv: GameMove,
    index: usize,
    iscp: bool,
    gives_check: bool,
    in_check: bool,
) -> i16 {
    let mut reduction = ((f64::from(p.depth_left) / 2. - 1.).max(0.).sqrt()
        + (index as f64 / 2.0 - 1.).max(0.).sqrt()) as i16;
    if iscp {
        reduction /= 2;
    }
    if p.beta - p.alpha > 1 {
        reduction = (f64::from(reduction) * 0.66) as i16;
    }
    if gives_check {
        reduction -= 1;
    }
    if in_check {
        reduction -= 2;
    }
    if thread.history_score[p.game_state.get_color_to_move()][mv.from as usize][mv.to as usize] > 0
    {
        reduction -= 1;
    }
    reduction = reduction.min(p.depth_left - 1);
    reduction.max(1)
}

#[inline(always)]
pub fn uci_report_pv(
    p: &CombinedSearchParameters,
    thread: &mut Thread,
    following_score: i16,
    no_fail: bool,
) {
    if p.current_depth == 0 {
        thread.replace_current_pv(
            p.game_state,
            ScoredPrincipalVariation {
                pv: thread.pv_table[0].clone(),
                score: following_score,
                depth: p.depth_left as usize,
            },
            no_fail,
        );
    }
}

#[inline(always)]
pub fn update_quiet_cutoff(
    p: &CombinedSearchParameters,
    thread: &mut Thread,
    mv: GameMove,
    quiets_tried: usize,
) {
    thread.hh_score[p.game_state.get_color_to_move()][mv.from as usize][mv.to as usize] +=
        p.depth_left as usize * p.depth_left as usize;
    thread.history_score[p.game_state.get_color_to_move()][mv.from as usize][mv.to as usize] +=
        p.depth_left as isize * p.depth_left as isize;
    decrement_history_quiets(
        thread,
        p.current_depth,
        quiets_tried,
        p.depth_left as isize,
        p.game_state.get_color_to_move(),
    );
    if let Some(s) = thread.killer_moves[p.current_depth][0] {
        if mv == s {
            return;
        }
    }
    if let Some(s) = thread.killer_moves[p.current_depth][1] {
        if mv == s {
            return;
        }
    }
    if let Some(s) = thread.killer_moves[p.current_depth][0] {
        thread.killer_moves[p.current_depth][1] = Some(s);
    }
    thread.killer_moves[p.current_depth][0] = Some(mv);
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
