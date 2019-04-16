use std::fmt::{Display, Formatter, Result};
use super::search::Search;
use super::super::GameState;
use super::super::board_representation::game_state::{GameMove, GameMoveType, PieceType};
use super::super::movegen;
use super::cache::CacheEntry;
use super::quiesence::q_search;
use super::statistics::SearchStatistics;
use crate::evaluation::eval_game_state;

pub const MATE_SCORE: f64 = 3000.0;

//Roadmap
//Finish principal variation search

pub fn principal_variation_search(mut alpha: f64, mut beta: f64, depth_left: isize, game_state: &GameState, color: isize, stats: &mut SearchStatistics, current_depth: usize, search: &mut Search) -> PrincipalVariation {
    stats.add_normal_node(current_depth);

    let mut pv: PrincipalVariation = PrincipalVariation::new(depth_left as usize);
    let (mut legal_moves, in_check) = movegen::generate_moves(&game_state);
    let game_status = check_end_condition(&game_state, legal_moves.len() > 0, in_check);
    if game_status != GameResult::Ingame {
        pv.score = leaf_score(game_status, color, depth_left);
        return pv;
    }
    if depth_left <= 0 {
        stats.add_q_root();
        pv = q_search(alpha, beta, &game_state, color, 0, stats, legal_moves, in_check, current_depth, search);
        return pv;
        //pv.score = crate::evaluation::eval_game_state(&game_state).final_eval * color as f64;
        //return pv;
        //return eval_game_state(&game_state).final_eval * color as f64;
    }

    let mut in_pv = false;
    {
        if let Some(ce) = search.principal_variation[current_depth] {
            if ce.hash == game_state.hash {
                in_pv = true;
                let mv = CacheEntry::u16_to_mv(ce.mv, &game_state);
                let mv_index = find_move(&mv, &legal_moves);
                legal_moves.swap(0, mv_index);
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
                let mv_index = find_move(&mv, &legal_moves);
                legal_moves.swap(0, mv_index);
            }
        }
    }

    let mut index = 0;
    for mv in legal_moves {
        let next_state = movegen::make_move(&game_state, &mv);
        let mut following_pv: PrincipalVariation;
        if depth_left <= 2 || !in_pv || index == 0 {
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

pub fn find_move(mv: &GameMove, mv_list: &Vec<GameMove>) -> usize {
    let mut mv_index = 0;
    for mvs in mv_list {
        if mvs.from == mv.from && mvs.to == mv.to && mvs.move_type == mv.move_type {
            break;
        }
        mv_index += 1;
    }
    if mv_index < mv_list.len() {
        return mv_index;
    } else {
        panic!("Type 2 error");
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


pub fn check_end_condition(game_state: &GameState, has_legal_moves: bool, in_check: bool) -> GameResult {
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