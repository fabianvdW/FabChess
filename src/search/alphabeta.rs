use std::fmt::{Display, Formatter, Result};
use super::search::Search;
use super::super::GameState;
use super::super::board_representation::game_state::{GameMove, GameMoveType};
use super::super::movegen;
use super::cache::{CacheEntry, Cache};
use super::quiesence::{q_search, see, is_capture};
use super::GradedMove;
use crate::evaluation::eval_game_state;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const MATE_SCORE: f64 = 3000.0;
pub const HH_INCREMENT: usize = 1;
pub const BF_INCREMENT: usize = 1;


pub fn principal_variation_search(mut alpha: f64, mut beta: f64, mut depth_left: isize, game_state: &GameState, color: isize, current_depth: usize, search: &mut Search, root_pliesplayed: usize, history: &mut Vec<u64>, stop: &Arc<AtomicBool>, cache: &mut Cache) -> PrincipalVariation {
    search.search_statistics.add_normal_node(current_depth);
    if search.search_statistics.nodes_searched % 4095 == 0 {
        checkup(search, stop);
    }

    let root = root_pliesplayed == ((game_state.full_moves - 1) * 2 + game_state.color_to_move);
    let mut pv: PrincipalVariation = PrincipalVariation::new(depth_left as usize);

    if search.stop {
        return pv;
    }
    let (legal_moves, in_check) = movegen::generate_moves(&game_state);
    if !root {
        let game_status = check_end_condition(&game_state, legal_moves.len() > 0, in_check, history);
        if game_status != GameResult::Ingame {
            pv.score = leaf_score(game_status, color, depth_left);
            return pv;
        }
    }

    if in_check && !root {
        depth_left += 1;
    }

    if depth_left <= 0 {
        search.search_statistics.add_q_root();
        pv = q_search(alpha, beta, &game_state, color, 0, legal_moves, in_check, current_depth, search, history, cache, root_pliesplayed);
        return pv;
    }

    //Move Ordering
    //1. PV-Move +30000.0
    //2. Hash move + 29999.0
    //if see>0
    //3. Winning captures Sort by SEE + 25000
    //4. Equal captures Sort by SEE+ 25000
    //5. Killer moves + 20000
    //6. Non captures (history heuristic) history heuristic score
    //7. Losing captures (see<0) see score
    let mut graded_moves = Vec::with_capacity(legal_moves.len());
    for mv in legal_moves {
        if is_capture(&mv) {
            if GameMoveType::EnPassant == mv.move_type {
                graded_moves.push(GradedMove::new(mv, 24999.0));
            } else {
                let mut sval = see(&game_state, &mv, true);
                if sval >= 0.0 {
                    sval += 25000.0;
                }
                graded_moves.push(GradedMove::new(mv, sval));
            }
        } else {
            //History Heuristic
            let score = search.hh_score[mv.from][mv.to] as f64 / search.bf_score[mv.from][mv.to] as f64 / 10000000.0;
            graded_moves.push(GradedMove::new(mv, score));
        }
    }

    {
        //Killer moves
        if let Some(s) = search.killer_moves[current_depth][0] {
            let mv_index = find_move(&s, &graded_moves, false);
            if mv_index < graded_moves.len() {
                graded_moves[mv_index].score += 2000.0;
            }
        }
        if let Some(s) = search.killer_moves[current_depth][1] {
            let mv_index = find_move(&s, &graded_moves, false);
            if mv_index < graded_moves.len() {
                graded_moves[mv_index].score += 2000.0;
            }
        }
    }

    let mut in_pv = false;
    {
        if let Some(ce) = search.principal_variation[current_depth] {
            if ce.hash == game_state.hash {
                in_pv = true;
                let mv = CacheEntry::u16_to_mv(ce.mv, &game_state);
                let mv_index = find_move(&mv, &graded_moves, true);
                graded_moves[mv_index].score = 30000.0;
            }
        }
    }

    //Probe TT
    let mut cache_hit = false;
    if !in_pv {
        let ce = &cache.cache[game_state.hash as usize & super::cache::CACHE_MASK];
        if let Some(s) = ce {
            let ce: &CacheEntry = s;
            if ce.hash == game_state.hash {
                search.search_statistics.add_cache_hit_ns();
                if ce.depth >= depth_left as i8 {
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
                let mv = CacheEntry::u16_to_mv(ce.mv, &game_state);
                let mv_index = find_move(&mv, &graded_moves, true);
                graded_moves[mv_index].score = 29900.0;
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

    if beta - alpha <= 0.002 && depth_left >= 4 && !in_check &&
        (game_state.pieces[1][game_state.color_to_move] | game_state.pieces[2][game_state.color_to_move] |
            game_state.pieces[3][game_state.color_to_move] | game_state.pieces[4][game_state.color_to_move]) != 0u64 {
        if eval_game_state(&game_state).final_eval * color as f64 >= beta {
            let nextgs = movegen::make_nullmove(&game_state);
            let rat = -principal_variation_search(-beta, -beta + 0.001, depth_left - 4, &nextgs, -color, current_depth + 1, search, root_pliesplayed, next_history, stop, cache).score;
            if rat >= beta {
                search.search_statistics.add_nm_pruning();
                pv.score = rat;
                next_history.pop();
                return pv;
            }
        }
    }

    if beta - alpha > 0.002 && !in_pv && !cache_hit && depth_left >= 5 {
        next_history.pop();
        let iid = principal_variation_search(alpha, beta, depth_left / 2, &game_state, color, current_depth, search, root_pliesplayed, next_history, stop, cache);
        next_history.push(game_state.hash);
        let mv_index = find_move(&iid.pv[0], &graded_moves, true);
        graded_moves[mv_index].score = 29900.0;
    }
    let mut index: usize = 0;
    while graded_moves.len() > 0 {
        let gmvindex = get_next_gm(&graded_moves);
        let gmv = graded_moves.remove(gmvindex);
        let mv = gmv.mv;
        let isc = is_capture(&mv);
        let isp = if let GameMoveType::Promotion(_, _) = mv.move_type { true } else { false };
        let next_state = movegen::make_move(&game_state, &mv);
        let mut following_pv: PrincipalVariation;
        if depth_left > 2 && !in_pv && !in_check && (!isc || gmv.score < 0.0) && index >= 2 && !isp {
            //let mut reduction = 1;
            let mut reduction = (((depth_left - 1isize) as f64).sqrt() + ((index - 1) as f64).sqrt()) as isize;
            if beta - alpha > 0.002 {
                reduction = (reduction as f64 * 0.66) as isize;
            }
            if reduction > depth_left - 2 {
                reduction = depth_left - 2
            }
            following_pv = principal_variation_search(-beta, -alpha, depth_left - 1 - reduction, &next_state, -color, current_depth + 1, search, root_pliesplayed, next_history, stop, cache);
            if -following_pv.score > alpha {
                following_pv = principal_variation_search(-beta, -alpha, depth_left - 1, &next_state, -color, current_depth + 1, search, root_pliesplayed, next_history, stop, cache);
            }
        } else if depth_left <= 2 || !in_pv || index == 0 {
            following_pv = principal_variation_search(-beta, -alpha, depth_left - 1, &next_state, -color, current_depth + 1, search, root_pliesplayed, next_history, stop, cache);
        } else {
            following_pv = principal_variation_search(-alpha - 0.001, -alpha, depth_left - 1, &next_state, -color, current_depth + 1, search, root_pliesplayed, next_history, stop, cache);
            let rating = -following_pv.score;
            if rating > alpha {
                following_pv = principal_variation_search(-beta, -alpha, depth_left - 1, &next_state, -color, current_depth + 1, search, root_pliesplayed, next_history, stop, cache);
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
        index += 1;
    }
    next_history.pop();
    if alpha < beta {
        search.search_statistics.add_normal_node_non_beta_cutoff();
    }
    //Make cache
    if !search.stop {
        make_cache(cache, &pv, &game_state, alpha, beta, depth_left, root_pliesplayed);
    }
    return pv;
}

pub fn get_occurences(history: &Vec<u64>, game_state: &GameState) -> usize {
    let mut occurences = 0;
    for gs in history.iter().rev() {
        if *gs == game_state.hash {
            occurences += 1;
        }
    }
    occurences
}

pub fn checkup(search: &mut Search, stop: &Arc<AtomicBool>) {
    search.search_statistics.refresh_time_elapsed();
    if search.tc.time_over(search.search_statistics.time_elapsed) || stop.load(Ordering::Relaxed) {
        search.stop = true;
    }
}

pub fn get_next_gm(mv_list: &Vec<GradedMove>) -> usize {
    if mv_list.len() == 0 {
        panic!("List has to be longer than 1")
    } else {
        let mut best_mv = &mv_list[0];
        let mut index = 0;
        for i in 1..mv_list.len() {
            if mv_list[i].score > best_mv.score {
                best_mv = &mv_list[i];
                index = i;
            }
        }
        return index;
    }
}

pub fn find_move(mv: &GameMove, mv_list: &Vec<GradedMove>, contains: bool) -> usize {
    let mut mv_index = 0;
    for gmvs in mv_list {
        let mvs = &gmvs.mv;
        if mvs.from == mv.from && mvs.to == mv.to && mvs.move_type == mv.move_type {
            break;
        }
        mv_index += 1;
    }
    if mv_index < mv_list.len() {
        return mv_index;
    } else if contains {
        panic!("Type 2 error");
    } else {
        return mv_index;
    }
}

pub fn make_cache(cache: &mut Cache, pv: &PrincipalVariation, game_state: &GameState, original_alpha: f64, beta: f64, depth_left: isize, root_plies_played: usize) {
    let beta_node: bool = pv.score >= beta;
    let alpha_node: bool = pv.score < original_alpha;

    let index = game_state.hash as usize & super::cache::CACHE_MASK;

    let ce = &cache.cache[game_state.hash as usize & super::cache::CACHE_MASK];
    let new_entry_val = depth_left as f64 * if beta_node || alpha_node { 0.7 } else { 1.0 };
    if let None = ce {
        let new_entry = CacheEntry::new(&game_state, depth_left as isize, pv.score, alpha_node, beta_node, match pv.pv.get(0) {
            Some(mv) => &mv,
            _ => panic!("Invalid pv!")
        });
        cache.cache[index] = Some(new_entry);
    } else {
        let old_entry: &CacheEntry = match ce {
            Some(s) => s,
            _ => panic!("Invalid if let!")
        };
        //Make replacement scheme better
        let mut old_entry_val = old_entry.depth as f64 * if old_entry.beta || old_entry.alpha { 0.7 } else { 1.0 };
        if old_entry.plies_played < root_plies_played as u16 {
            old_entry_val = -1.0;
        }
        if old_entry_val <= new_entry_val {
            let new_entry = CacheEntry::new(&game_state, depth_left as isize, pv.score, alpha_node, beta_node, match pv.pv.get(0) {
                Some(mv) => &mv,
                _ => panic!("Invalid pv!")
            });
            cache.cache[index] = Some(new_entry);
        }
    }
}

pub fn leaf_score(game_status: GameResult, color: isize, depth_left: isize) -> f64 {
    if game_status == GameResult::Draw {
        return 0.0;
    } else if game_status == GameResult::WhiteWin {
        return (MATE_SCORE + depth_left as f64) * color as f64;
    } else if game_status == GameResult::BlackWin {
        return (MATE_SCORE + depth_left as f64) * -color as f64;
    }
    panic!("Invalid Leaf");
}


pub fn check_end_condition(game_state: &GameState, has_legal_moves: bool, in_check: bool, history: &Vec<u64>) -> GameResult {
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
    if game_state.pieces[0][0] | game_state.pieces[1][0] | game_state.pieces[2][0] | game_state.pieces[3][0] | game_state.pieces[4][0]
        | game_state.pieces[0][1] | game_state.pieces[1][1] | game_state.pieces[2][1] | game_state.pieces[3][1] | game_state.pieces[4][1] == 0u64 {
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

#[derive(PartialEq)]
pub enum GameResult {
    Ingame,
    WhiteWin,
    BlackWin,
    Draw,
}

pub struct PrincipalVariation {
    pub pv: Vec<GameMove>,
    pub score: f64,
}

impl PrincipalVariation {
    pub fn new(depth_left: usize) -> PrincipalVariation {
        PrincipalVariation {
            pv: Vec::with_capacity(depth_left),
            score: -1000000.0,
        }
    }
}

impl Display for PrincipalVariation {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!("PV of length {} with score {}\n", self.pv.len(), self.score));
        for mv in &self.pv {
            res_str.push_str(&format!("{:?}\n", mv));
        }
        write!(formatter, "{}", res_str)
    }
}