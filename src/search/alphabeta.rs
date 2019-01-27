use super::super::GameState;
use super::super::board_representation::game_state::{GameMove, GameMoveType, PieceType};
use super::super::movegen;
use super::super::evaluation::{self, eval_game_state};
use super::quiesence::q_search_root;
use super::statistics::SearchStatistics;

pub const MATE_SCORE: f64 = 300.0;

pub fn search_gamestate(game_state: &GameState) {}

pub fn principal_variation_search(alpha: f64, beta: f64, depth_left: isize, game_state: &GameState, color: isize, stats: &mut SearchStatistics) -> f64 {
    stats.add_normal_node();
    let (legal_moves, in_check) = movegen::generate_moves(&game_state);
    let game_status = check_end_condition(&game_state, legal_moves.len() > 0, in_check);
    if game_status != GameResult::Ingame {
        return leaf_score(game_status, color, depth_left);
    }
    if depth_left <= 0 {
        return q_search_root(alpha, beta, &game_state, color, legal_moves,stats);
    }
    0.0
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
            return GameResult::WhiteWin;
        } else {
            return GameResult::BlackWin;
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
    GameResult::Ingame
}

pub fn match_move_type(move_type: &GameMoveType) -> usize {
    match move_type {
        GameMoveType::Quiet => 0,
        GameMoveType::Capture(_) => 1,
        GameMoveType::EnPassant => 2,
        GameMoveType::Castle => 3,
        GameMoveType::Promotion(..) => 4
    }
}

pub struct MoveFilter {
    pub legal_moves: Vec<GameMove>,
    pub move_type: GameMoveType,
}

impl<'t> IntoIterator for &'t MoveFilter {
    type Item = &'t GameMove;
    type IntoIter = MoveFilterIterator<'t>;
    fn into_iter(self) -> Self::IntoIter {
        MoveFilterIterator {
            legal_moves: &self.legal_moves,
            move_type: match_move_type(&self.move_type),
            index: 0,
        }
    }
}

pub struct MoveFilterIterator<'a> {
    legal_moves: &'a Vec<GameMove>,
    move_type: usize,
    index: usize,
}

impl<'a> Iterator for MoveFilterIterator<'a> {
    type Item = &'a GameMove;
    fn next(&mut self) -> Option<&'a GameMove> {
        while self.index < self.legal_moves.len() {
            if match_move_type(&self.legal_moves[self.index].move_type) == self.move_type {
                return Some(&self.legal_moves[self.index]);
            } else {
                self.index += 1;
            }
        }
        None
    }
}

#[derive(PartialEq)]
pub enum GameResult {
    Ingame,
    WhiteWin,
    BlackWin,
    Draw,
}