use super::super::board_representation::game_state::{GameMove, GameMoveType, GameResult};
use super::super::movegen;
use super::super::movegen::{AdditionalGameStateInformation, MoveList};
use super::super::GameState;
use super::cache::{Cache, CacheEntry};
use super::quiescence::{is_capture, q_search, see};
use super::search::Search;
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
pub fn principal_variation_search(
    mut alpha: i16,
    mut beta: i16,
    mut depth_left: i16,
    game_state: &GameState,
    color: i16,
    current_depth: usize,
    search: &mut Search,
    root_pliesplayed: usize,
    history: &mut Vec<u64>,
    stop: &Arc<AtomicBool>,
    cache: &mut Cache,
    move_list: &mut MoveList,
    calculated_moves: bool,
    agsi_pre: Option<AdditionalGameStateInformation>,
) -> PrincipalVariation {
    search.search_statistics.add_normal_node(current_depth);
    if search.search_statistics.nodes_searched % 1024 == 0 {
        checkup(search, stop);
    }

    let root = root_pliesplayed == ((game_state.full_moves - 1) * 2 + game_state.color_to_move);
    let mut pv: PrincipalVariation = PrincipalVariation::new(depth_left as usize);

    if search.stop {
        return pv;
    }
    let agsi = if calculated_moves {
        agsi_pre.expect("Couldn't unwrap agsi_pre")
    } else {
        movegen::generate_moves2(&game_state, false, move_list, current_depth)
    };
    if !root {
        let game_status = check_end_condition(
            &game_state,
            agsi.stm_haslegalmove,
            agsi.stm_incheck,
            history,
        );
        if game_status != GameResult::Ingame {
            pv.score = leaf_score(game_status, color, depth_left);
            return pv;
        }
    }

    //Check Extensions
    if agsi.stm_incheck && !root && !calculated_moves {
        depth_left += 1;
    }
    //Max search-depth reached
    if current_depth >= (MAX_SEARCH_DEPTH - 1) {
        pv.score = eval_game_state(&game_state, false).final_eval * color;
        return pv;
    }

    //Drop into quiescence search
    if depth_left <= 0 {
        search.search_statistics.add_q_root();
        pv = q_search(
            alpha,
            beta,
            &game_state,
            color,
            0,
            current_depth,
            search,
            history,
            cache,
            root_pliesplayed,
            move_list,
            agsi,
            true,
        );
        return pv;
    }

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
            move_list.graded_moves[current_depth][mv_index] = Some(GradedMove::new(mv_index, 0.0));
        }
        mv_index += 1;
    }

    let mut in_pv = false;
    {
        if let Some(ce) = search.principal_variation[current_depth] {
            if ce.hash == game_state.hash {
                in_pv = true;
                let mv = CacheEntry::u16_to_mv(ce.mv, &game_state);
                let mv_index = find_move(&mv, move_list, current_depth, true);
                move_list.graded_moves[current_depth][mv_index]
                    .as_mut()
                    .unwrap()
                    .score = 40000.0;
            }
        }
    }

    //Probe TT
    let mut static_evaluation = None;
    let mut cache_hit = false;
    {
        let ce = &cache.cache[game_state.hash as usize & super::cache::CACHE_MASK];
        if let Some(s) = ce {
            let ce: &CacheEntry = s;
            if ce.hash == game_state.hash {
                search.search_statistics.add_cache_hit_ns();
                if ce.depth >= depth_left as i8 {
                    if beta - alpha == 1 {
                        if !ce.alpha && !ce.beta {
                            search.search_statistics.add_cache_hit_replace_ns();
                            pv.pv.push(CacheEntry::u16_to_mv(ce.mv, &game_state));
                            pv.score = ce.score;
                            return pv;
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
                                search.search_statistics.add_cache_hit_aj_replace_ns();
                                pv.score = ce.score;
                                pv.pv.push(CacheEntry::u16_to_mv(ce.mv, &game_state));
                                return pv;
                            }
                        }
                    }
                }
                static_evaluation = ce.static_evaluation;
                let mv = CacheEntry::u16_to_mv(ce.mv, &game_state);
                let mv_index = find_move(&mv, move_list, current_depth, true);
                move_list.graded_moves[current_depth][mv_index]
                    .as_mut()
                    .unwrap()
                    .score = 29900.0;
                cache_hit = true;
            }
        }
    }

    let mut my_history: Vec<u64> = Vec::with_capacity(10);
    let next_history: &mut Vec<u64> = if game_state.half_moves == 0 {
        &mut my_history
    } else {
        history
    };
    next_history.push(game_state.hash);

    if beta - alpha == 1
        && !agsi.stm_incheck
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
                search,
                root_pliesplayed,
                next_history,
                stop,
                cache,
                move_list,
                false,
                None,
            )
            .score;
            if rat >= beta {
                search.search_statistics.add_nm_pruning();
                pv.score = rat;
                next_history.pop();
                return pv;
            }
        }
    }

    if beta - alpha > 1 && !in_pv && !cache_hit && depth_left > 6 {
        next_history.pop();
        let iid = principal_variation_search(
            alpha,
            beta,
            depth_left - 2,
            &game_state,
            color,
            current_depth,
            search,
            root_pliesplayed,
            next_history,
            stop,
            cache,
            move_list,
            true,
            Some(agsi.clone()),
        );
        next_history.push(game_state.hash);
        if search.stop {
            return pv;
        }
        if iid.pv.len() == 0 {
            panic!("IID PV is 0");
        }
        let mv_index = find_move(&iid.pv[0], move_list, current_depth, true);
        move_list.graded_moves[current_depth][mv_index]
            .as_mut()
            .unwrap()
            .score = 29900.0;
    }
    //History Heuristic
    let mut mv_index = 0;
    while mv_index < move_list.counter[current_depth] {
        let mv: &GameMove = move_list.move_list[current_depth][mv_index]
            .as_ref()
            .unwrap();
        let score = search.hh_score[mv.from][mv.to] as f64
            / search.bf_score[mv.from][mv.to] as f64
            / 1000.0;
        move_list.graded_moves[current_depth][mv_index]
            .as_mut()
            .unwrap()
            .score += score;
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

    let mut futil_pruning = depth_left <= 8 && !agsi.stm_incheck;
    let mut futil_margin = 0;
    if futil_pruning {
        if let None = static_evaluation {
            static_evaluation = Some(eval_game_state(&game_state, false).final_eval);
        }
        futil_margin = static_evaluation.unwrap() * color + depth_left * 120;
    }
    let mut index: usize = 0;
    let mut mv_index: usize = 0;
    while mv_index < move_list.counter[current_depth] {
        let gmvindex = get_next_gm(
            move_list,
            current_depth,
            mv_index,
            move_list.counter[current_depth],
        );
        let mv = move_list.move_list[current_depth][gmvindex].unwrap();
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
            && pv.score > MATED_IN_MAX
            && !gives_check(&mv, &game_state, &next_state)
        {
            if futil_margin <= alpha {
                mv_index += 1;
                continue;
            } else {
                futil_pruning = false;
            }
        }
        let mut following_pv: PrincipalVariation;
        let mut reduction = 0;
        if depth_left > 2
            && !in_pv
            && !agsi.stm_incheck
            && !isc
            && index >= 2
            && !isp
            && !gives_check(&mv, &game_state, &next_state)
        {
            //let mut reduction = 1;
            reduction = (((depth_left - 1) as f64).sqrt() + ((index - 1) as f64).sqrt()) as i16;
            if beta - alpha > 1 {
                reduction = (reduction as f64 * 0.66) as i16;
            }
            if reduction > depth_left - 2 {
                reduction = depth_left - 2
            }
        }
        if depth_left <= 2 || !in_pv || index == 0 {
            following_pv = principal_variation_search(
                -beta,
                -alpha,
                depth_left - 1 - reduction,
                &next_state,
                -color,
                current_depth + 1,
                search,
                root_pliesplayed,
                next_history,
                stop,
                cache,
                move_list,
                false,
                None,
            );
            if reduction > 0 && -following_pv.score > alpha {
                following_pv = principal_variation_search(
                    -beta,
                    -alpha,
                    depth_left - 1,
                    &next_state,
                    -color,
                    current_depth + 1,
                    search,
                    root_pliesplayed,
                    next_history,
                    stop,
                    cache,
                    move_list,
                    false,
                    None,
                );
            }
        } else {
            following_pv = principal_variation_search(
                -alpha - 1,
                -alpha,
                depth_left - 1,
                &next_state,
                -color,
                current_depth + 1,
                search,
                root_pliesplayed,
                next_history,
                stop,
                cache,
                move_list,
                false,
                None,
            );
            let rating = -following_pv.score;
            if rating > alpha {
                following_pv = principal_variation_search(
                    -beta,
                    -alpha,
                    depth_left - 1,
                    &next_state,
                    -color,
                    current_depth + 1,
                    search,
                    root_pliesplayed,
                    next_history,
                    stop,
                    cache,
                    move_list,
                    false,
                    None,
                );
            }
        }
        let rating = -following_pv.score;

        if rating > pv.score {
            pv.pv.clear();
            pv.pv.push(mv);
            pv.pv.append(&mut following_pv.pv);
            pv.score = rating;
        }
        if rating > alpha {
            alpha = rating;
        }
        if alpha >= beta {
            search.search_statistics.add_normal_node_beta_cutoff(index);
            if !isc {
                search.hh_score[mv.from][mv.to] += HH_INCREMENT;
                //Replace killers
                //Dont replace if already in table
                if let Some(s) = search.killer_moves[current_depth][0] {
                    if s.from == mv.from && s.to == mv.to && s.move_type == mv.move_type {
                        break;
                    }
                }
                if let Some(s) = search.killer_moves[current_depth][1] {
                    if s.from == mv.from && s.to == mv.to && s.move_type == mv.move_type {
                        break;
                    }
                }
                if let Some(s) = search.killer_moves[current_depth][0] {
                    search.killer_moves[current_depth][1] = Some(s);
                }
                search.killer_moves[current_depth][0] = Some(mv);
            }
            break;
        } else {
            if !isc {
                search.bf_score[mv.from][mv.to] += BF_INCREMENT;
            }
        }
        mv_index += 1;
        index += 1;
    }
    next_history.pop();
    if alpha < beta {
        search.search_statistics.add_normal_node_non_beta_cutoff();
    }
    //Make cache
    if !search.stop {
        make_cache(
            cache,
            &pv,
            &game_state,
            alpha,
            beta,
            depth_left,
            root_pliesplayed,
            static_evaluation,
        );
    }
    return pv;
}

#[inline(always)]
pub fn gives_check(_mv: &GameMove, game_state: &GameState, next_state: &GameState) -> bool {
    //Check if move gives check
    let stm_nextstate_iswhite = next_state.color_to_move == 0;
    let next_state_stm_king = next_state.pieces[5][next_state.color_to_move];
    let next_state_stm_king_sq = next_state_stm_king.trailing_zeros() as usize;
    let enemy_pawns = game_state.pieces[0][1 - next_state.color_to_move];
    let enemy_knights = game_state.pieces[1][1 - next_state.color_to_move];
    let enemy_bishops = game_state.pieces[2][1 - next_state.color_to_move]
        | game_state.pieces[4][1 - next_state.color_to_move];
    let enemy_rooks = game_state.pieces[3][1 - next_state.color_to_move]
        | game_state.pieces[4][1 - next_state.color_to_move];
    let blockers = enemy_pawns
        | enemy_knights
        | enemy_bishops
        | enemy_rooks
        | next_state.pieces[5][1 - next_state.color_to_move]
        | next_state.pieces[0][next_state.color_to_move]
        | next_state.pieces[1][next_state.color_to_move]
        | next_state.pieces[2][next_state.color_to_move]
        | next_state.pieces[3][next_state.color_to_move]
        | next_state.pieces[4][next_state.color_to_move];
    (if stm_nextstate_iswhite {
        movegen::attackers_from_black(
            next_state_stm_king,
            next_state_stm_king_sq,
            enemy_pawns,
            enemy_knights,
            enemy_bishops,
            enemy_rooks,
            blockers,
        )
        .0
    } else {
        movegen::attackers_from_white(
            next_state_stm_king,
            next_state_stm_king_sq,
            enemy_pawns,
            enemy_knights,
            enemy_bishops,
            enemy_rooks,
            blockers,
        )
        .0
    }) != 0u64
}

#[inline(always)]
pub fn get_occurences(history: &Vec<u64>, game_state: &GameState) -> usize {
    let mut occurences = 0;
    for gs in history.iter().rev() {
        if *gs == game_state.hash {
            occurences += 1;
        }
    }
    occurences
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
    game_state: &GameState,
    original_alpha: i16,
    beta: i16,
    depth_left: i16,
    root_plies_played: usize,
    static_evaluation: Option<i16>,
) {
    let beta_node: bool = pv.score >= beta;
    let alpha_node: bool = pv.score < original_alpha;

    let index = game_state.hash as usize & super::cache::CACHE_MASK;

    let ce = &cache.cache[game_state.hash as usize & super::cache::CACHE_MASK];
    let new_entry_val = depth_left as f64 * if beta_node || alpha_node { 0.7 } else { 1.0 };
    if let None = ce {
        let new_entry = CacheEntry::new(
            &game_state,
            depth_left,
            pv.score,
            alpha_node,
            beta_node,
            match pv.pv.get(0) {
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
                pv.score,
                alpha_node,
                beta_node,
                match pv.pv.get(0) {
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

#[inline(always)]
pub fn check_end_condition(
    game_state: &GameState,
    has_legal_moves: bool,
    in_check: bool,
    history: &Vec<u64>,
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
        return GameResult::Draw;
    }
    if game_state.half_moves >= 100 {
        return GameResult::Draw;
    }

    if get_occurences(history, game_state) >= 1 {
        return GameResult::Draw;
    }
    GameResult::Ingame
}
pub struct PrincipalVariation {
    pub pv: Vec<GameMove>,
    pub score: i16,
}

impl PrincipalVariation {
    pub fn new(depth_left: usize) -> PrincipalVariation {
        PrincipalVariation {
            pv: Vec::with_capacity(depth_left),
            score: -32767,
        }
    }
}

impl Display for PrincipalVariation {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!(
            "PV of length {} with score {}\n",
            self.pv.len(),
            self.score
        ));
        for mv in &self.pv {
            res_str.push_str(&format!("{:?}\n", mv));
        }
        write!(formatter, "{}", res_str)
    }
}
