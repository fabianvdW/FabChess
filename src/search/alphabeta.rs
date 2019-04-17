use std::fmt::{Display, Formatter, Result};
use super::search::Search;
use super::super::GameState;
use super::super::board_representation::game_state::{GameMove, GameMoveType, PieceType};
use super::super::movegen;
use super::cache::CacheEntry;
use super::quiesence::{q_search, see, is_capture};
use super::GradedMove;
use super::statistics::SearchStatistics;
use crate::evaluation::eval_game_state;

pub const MATE_SCORE: f64 = 3000.0;
pub const HH_INCREMENT: usize = 1;
pub const BF_INCREMENT: usize = 1;
//Roadmap
//Late Move Reduction
//Aspiration Windows

pub fn principal_variation_search(mut alpha: f64, mut beta: f64, depth_left: isize, game_state: &GameState, color: isize, stats: &mut SearchStatistics, current_depth: usize, search: &mut Search) -> PrincipalVariation {
    stats.add_normal_node(current_depth);
    if stats.nodes_searched % 4096 == 0 {
        checkup(stats);
    }

    let mut pv: PrincipalVariation = PrincipalVariation::new(depth_left as usize);

    if stats.stop {
        return pv;
    }
    let (mut legal_moves, in_check) = movegen::generate_moves(&game_state);
    let game_status = check_end_condition(&game_state, legal_moves.len() > 0, in_check, &search);
    if game_status != GameResult::Ingame {
        pv.score = leaf_score(game_status, color, depth_left);
        return pv;
    }
    if depth_left <= 0 {
        stats.add_q_root();
        pv = q_search(alpha, beta, &game_state, color, 0, stats, legal_moves, in_check, current_depth, search);
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
                let mut sval = see(&game_state, &mv, beta - alpha >= 0.002);
                if sval >= 0.0 {
                    sval += 25000.0;
                }
                graded_moves.push(GradedMove::new(mv, sval));
            }
        } else {
            //History Heuristic
            graded_moves.push(GradedMove::new(mv, search.hh_score[mv.from][mv.to] as f64 / 10000000.0));
        }
    }

    {
        //Killer moves
        if let Some(s) = search.killer_moves[current_depth][0] {
            let mv_index = find_move(&s, &graded_moves, false);
            if mv_index < graded_moves.len() {
                graded_moves[mv_index].score += 20000.0;
            }
        }
        if let Some(s) = search.killer_moves[current_depth][1] {
            let mv_index = find_move(&s, &graded_moves, false);
            if mv_index < graded_moves.len() {
                graded_moves[mv_index].score += 20000.0;
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
    if !in_pv {
        let ce = &search.cache.cache[game_state.hash as usize & super::cache::CACHE_MASK];
        if let Some(s) = ce {
            let ce: &CacheEntry = s;
            if ce.hash == game_state.hash {
                stats.add_cache_hit_ns();
                if ce.occurences == 0 && ce.depth >= depth_left as i8 {
                    if !ce.alpha && !ce.beta {
                        stats.add_cache_hit_replace_ns();
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
                            stats.add_cache_hit_aj_replace_ns();
                            pv.score = ce.score;
                            pv.pv.push(CacheEntry::u16_to_mv(ce.mv, &game_state));
                            return pv;
                        }
                    }
                }
                let mv = CacheEntry::u16_to_mv(ce.mv, &game_state);
                let mv_index = find_move(&mv, &graded_moves, true);
                graded_moves[mv_index].score = 29900.0;
            }
        }
    }

    if beta - alpha <= 0.002 && depth_left >= 4 && !in_check &&
        (game_state.pieces[1][game_state.color_to_move] | game_state.pieces[2][game_state.color_to_move] |
            game_state.pieces[3][game_state.color_to_move] | game_state.pieces[4][game_state.color_to_move]) != 0u64 {
        if eval_game_state(&game_state).final_eval * color as f64 >= beta {
            let rat = -principal_variation_search(-beta, -beta + 0.001, depth_left - 4, &movegen::make_nullmove(&game_state), -color, stats, current_depth + 1, search).score;
            if rat >= beta {
                stats.add_nm_pruning();
                pv.score = rat;
                return pv;
            }
        }
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
        if depth_left > 2 && alpha - beta <= 0.002 && !in_check && !isc && index >= 5 && !isp && gmv.score < 18000.0 {
            //let reduction = if index >= 10 { depth_left / 3 } else { 1 };
            let mut reduction = (((depth_left - 1isize) as f64).sqrt() + ((index - 1) as f64).sqrt()) as isize;
            if reduction > depth_left - 2 {
                reduction = depth_left - 2
            }
            following_pv = principal_variation_search(-beta, -alpha, depth_left - 1 - reduction, &next_state, -color, stats, current_depth + 1, search);
            if -following_pv.score > alpha {
                following_pv = principal_variation_search(-beta, -alpha, depth_left - 1, &next_state, -color, stats, current_depth + 1, search);
            }
        } else if depth_left <= 2 || !in_pv || index == 0 {
            following_pv = principal_variation_search(-beta, -alpha, depth_left - 1, &next_state, -color, stats, current_depth + 1, search);
        } else {
            following_pv = principal_variation_search(-alpha - 0.001, -alpha, depth_left - 1, &next_state, -color, stats, current_depth + 1, search);
            let rating = -following_pv.score;
            if rating > alpha {
                following_pv = principal_variation_search(-beta, -alpha, depth_left - 1, &next_state, -color, stats, current_depth + 1, search);
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
            stats.add_normal_node_beta_cutoff(index);
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
        }
        index += 1;
    }
    if alpha < beta {
        stats.add_normal_node_non_beta_cutoff();
    }
    //Make cache
    make_cache(search, &pv, &game_state, alpha, beta, depth_left);
    return pv;
}

pub fn checkup(stats: &mut SearchStatistics) {
    stats.refresh_time_elapsed();
    if stats.time_elapsed > 3000 {
        stats.stop = true;
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

pub fn make_cache(search: &mut Search, pv: &PrincipalVariation, game_state: &GameState, original_alpha: f64, beta: f64, depth_left: isize) {
    let beta_node: bool = pv.score >= beta;
    let alpha_node: bool = pv.score < original_alpha;

    let index = game_state.hash as usize & super::cache::CACHE_MASK;

    let ce = &search.cache.cache[game_state.hash as usize & super::cache::CACHE_MASK];
    let new_entry = CacheEntry::new(&game_state, depth_left as isize, pv.score, alpha_node, beta_node, match pv.pv.get(0) {
        Some(mv) => &mv,
        _ => panic!("Invalid pv!")
    });
    if let None = ce {
        search.cache.cache[index] = Some(new_entry);
    } else {
        let old_entry: &CacheEntry = match ce {
            Some(s) => s,
            _ => panic!("Invalid if let!")
        };
        //Make replacement scheme better
        if old_entry.occurences == 0 && old_entry.depth <= depth_left as i8 {
            search.cache.cache[index] = Some(new_entry);
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


pub fn check_end_condition(game_state: &GameState, has_legal_moves: bool, in_check: bool, search: &Search) -> GameResult {
    if in_check & !has_legal_moves {
        if game_state.color_to_move == 0 {
            return GameResult::BlackWin;
        } else {
            return GameResult::WhiteWin;
        }
    }
    if !in_check & !has_legal_moves {
        return GameResult::Draw;
    }
    if game_state.pieces[0][0] | game_state.pieces[1][0] | game_state.pieces[2][0] | game_state.pieces[3][0] | game_state.pieces[4][0]
        | game_state.pieces[0][1] | game_state.pieces[1][1] | game_state.pieces[2][1] | game_state.pieces[3][1] | game_state.pieces[4][1] == 0u64 {
        return GameResult::Draw;
    }
    if game_state.half_moves >= 100 {
        return GameResult::Draw;
    }
    //Missing 3-fold repetition
    //This is checked by looking up in cache
    //Cache entry has field occurences which is set when cached entry happened in game
    //Entry with occurences>0 can't be overwritten
    //In the rare case of two played positions mapping to same cache entry, check hash and if not equal go to next(wrapping)
    //If occurences ==2 return DRAW
    if game_state.half_moves > 0 {
        let ce = &search.cache.cache[game_state.hash as usize & super::cache::CACHE_MASK];
        if let Some(entry) = ce {
            let mut occ_entry = entry;
            if occ_entry.occurences > 0 {
                while occ_entry.hash != game_state.hash {
                    let next_entry = &search.cache.cache[(occ_entry.hash + 1) as usize & super::cache::CACHE_MASK];
                    occ_entry = match next_entry {
                        Some(s) => s,
                        _ => panic!("Can't be!")
                    };
                }
                if occ_entry.occurences == 2 {
                    return GameResult::Draw;
                }
            }
        }
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