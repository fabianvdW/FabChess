use std::fmt::{Display, Formatter, Result};
use super::super::GameState;
use super::super::board_representation::game_state::{GameMove, GameMoveType, PieceType};
use super::super::movegen;
use super::quiesence::q_search;
use super::statistics::SearchStatistics;
use crate::evaluation::eval_game_state;

pub const MATE_SCORE: f64 = 3000.0;

pub fn search_gamestate(game_state: &GameState) {}

pub fn principal_variation_search(mut alpha: f64, mut beta: f64, depth_left: isize, game_state: &GameState, color: isize, stats: &mut SearchStatistics, current_depth: usize) -> PrincipalVariation {
    stats.add_normal_node(current_depth);
    let mut pv: PrincipalVariation = PrincipalVariation::new(depth_left as usize);
    let (legal_moves, in_check) = movegen::generate_moves(&game_state);
    let game_status = check_end_condition(&game_state, legal_moves.len() > 0, in_check);
    if game_status != GameResult::Ingame {
        pv.score = leaf_score(game_status, color, depth_left);
        return pv;
    }
    if depth_left <= 0 {
        stats.add_q_root();
        pv.score = q_search(alpha, beta, &game_state, color, 0, stats, legal_moves, in_check, current_depth);
        return pv;
        //return eval_game_state(&game_state).final_eval * color as f64;
    }
    let mut index = 0;
    for mv in legal_moves {
        let mut following_pv = principal_variation_search(-beta, -alpha, depth_left - 1, &movegen::make_move(&game_state, &mv), -color, stats, current_depth + 1);
        let rating = -following_pv.score;
        if rating > alpha {
            pv.pv.clear();
            pv.pv.push(mv);
            pv.pv.append(&mut following_pv.pv);
            alpha = rating;
        }
        if alpha >= beta {
            stats.add_normal_node_beta_cutoff(index);
            pv.score = beta;
            return pv;
        }
        index += 1;
    }
    stats.add_normal_node_non_beta_cutoff();
    pv.score = alpha;
    return pv;
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