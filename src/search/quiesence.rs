use super::super::board_representation::game_state::{GameState, GameMove, GameMoveType, PieceType};
use super::super::evaluation::{eval_game_state, self};
use super::super::move_generation::movegen;
use super::alphabeta::{MoveFilter, GameResult, check_end_condition, leaf_score};
use super::statistics::SearchStatistics;
use super::super::misc;

lazy_static! {
pub static ref PIECE_VALUES:[f64;6] = [100.0,325.0,325.0,500.0,1000.0,999999999.99];
}
pub fn q_search_root(mut alpha: f64, beta: f64, game_state: &GameState, color: isize, legal_moves: Vec<GameMove>, stats: &mut SearchStatistics) -> f64 {
    let evaluation = eval_game_state(&game_state);
    let stand_pat = evaluation.final_eval * color as f64;
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }
    let mv = MoveFilter { legal_moves, move_type: GameMoveType::Capture(PieceType::Pawn) };
    for capture_move in mv.into_iter() {
        //IF see>0 && delta pruning do
        if !passes_delta_pruning(&capture_move, &game_state, evaluation.phase, stand_pat, alpha) {
            stats.add_q_delta_cutoff();
            continue;
        }
        if see(&game_state, &capture_move) < 0.0 {
            stats.add_q_see_cutoff();
            continue;
        }
        let score = -q_search(-beta, -alpha, &movegen::make_move(&game_state, &capture_move), -color, -1, stats);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

pub fn q_search(mut alpha: f64, beta: f64, game_state: &GameState, color: isize, depth_left: isize, stats: &mut SearchStatistics) -> f64 {
    stats.add_q_node();
    let evaluation = eval_game_state(&game_state);
    let stand_pat = evaluation.final_eval * color as f64;
    if stand_pat >= beta {
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }
    let (moves, in_check) = movegen::generate_moves(&game_state);
    let game_status = check_end_condition(&game_state, moves.len() > 0, in_check);
    if game_status != GameResult::Ingame {
        return leaf_score(game_status, color, depth_left);
    }

    let mv = MoveFilter { legal_moves: moves, move_type: GameMoveType::Capture(PieceType::Pawn) };
    //We are missing promotions and enpassants
    for capture_move in mv.into_iter() {
        //IF see>0 && delta pruning do
        if !passes_delta_pruning(&capture_move, &game_state, evaluation.phase, stand_pat, alpha) {
            stats.add_q_delta_cutoff();
            continue;
        }
        if see(&game_state, &capture_move) < 0.0 {
            stats.add_q_see_cutoff();
            continue;
        }
        let score = -q_search(-beta, -alpha, &movegen::make_move(&game_state, &capture_move), -color, depth_left - 1, stats);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

pub fn passes_delta_pruning(capture_move: &GameMove, game_state: &GameState, phase: f64, eval: f64, alpha: f64) -> bool {
    if phase == 0.0 {
        return true;
    }
    if let GameMoveType::Promotion(_, _) = capture_move.move_type {
        return true;
    }
    let captured_piece = match &capture_move.move_type {
        GameMoveType::Capture(c) => c,
        _ => panic!("No capture!")
    };
    eval + evaluation::piece_value(&captured_piece, phase) + 200.0 >= alpha
}

pub fn see(game_state: &GameState, mv: &GameMove) -> f64 {
    let mut gain = Vec::with_capacity(32);
    let may_xray = game_state.pieces[2][0] | game_state.pieces[2][1] | game_state.pieces[3][0] | game_state.pieces[3][1] |
        game_state.pieces[4][0] | game_state.pieces[4][1];
    let mut from_set = 1u64 << mv.from;
    let mut occ = get_occupied_board(&game_state);
    let mut attadef = attacks_to(&game_state, mv.to, occ);
    gain.push(capture_value(&mv));
    let mut color_to_move = game_state.color_to_move;
    let mut attacked_piece = match mv.piece_type {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => 5
    };
    let mut index = 0;
    let mut deleted_pieces = 0u64;
    while from_set != 0u64 {
        index += 1;
        gain.push(PIECE_VALUES[attacked_piece] - gain[index - 1]);
        if (-gain[index - 1]).max(gain[index]) < 0.0 {
            break;
        }
        attadef ^= from_set;
        occ ^= from_set;
        if from_set & may_xray != 0u64 {
            //Recalculate rays
            attadef |= recalculate_sliders(&game_state, color_to_move, mv.to, occ);
        }
        color_to_move = 1 - color_to_move;
        let res = least_valuable_piece(attadef, color_to_move, &game_state);
        from_set = res.0;
        attacked_piece = res.1;
    }
    while index > 1 {
        index -= 1;
        gain[index - 1] = -((-gain[index - 1]).max(gain[index]));
    }
    gain[0]
}

pub fn recalculate_sliders(game_state: &GameState, color_to_move: usize, square: usize, occ: u64) -> u64 {
    //Bishops
    movegen::bishop_attack(square, occ) & (game_state.pieces[2][color_to_move] | game_state.pieces[4][color_to_move])
        | movegen::rook_attack(square, occ) & (game_state.pieces[3][color_to_move] | game_state.pieces[4][color_to_move])
}

pub fn attacks_to(game_state: &GameState, square: usize, occ: u64) -> u64 {
    let square_board = 1u64 << square;
    movegen::attackers_from_white(square_board, square, game_state.pieces[0][0], game_state.pieces[1][0], game_state.pieces[2][0] | game_state.pieces[4][0], game_state.pieces[3][0] | game_state.pieces[4][0], occ).0
        | movegen::attackers_from_black(square_board, square, game_state.pieces[0][1], game_state.pieces[1][1], game_state.pieces[2][1] | game_state.pieces[4][1], game_state.pieces[3][1] | game_state.pieces[4][1], occ).0
}

pub fn capture_value(mv: &GameMove) -> f64 {
    match &mv.move_type {
        GameMoveType::Capture(c) => {
            match c {
                PieceType::Pawn => 100.0,
                PieceType::Knight => 300.0,
                PieceType::Bishop => 310.0,
                PieceType::Rook => 500.0,
                PieceType::Queen => 900.0,
                PieceType::King => 999999999999.99,
            }
        }
        _ => panic!("No capture")
    }
}

pub fn get_occupied_board(game_state: &GameState) -> u64 {
    game_state.pieces[0][0] | game_state.pieces[1][0] | game_state.pieces[2][0] | game_state.pieces[3][0] | game_state.pieces[4][0] | game_state.pieces[5][0]
        | game_state.pieces[0][1] | game_state.pieces[1][1] | game_state.pieces[2][1] | game_state.pieces[3][1] | game_state.pieces[4][1] | game_state.pieces[5][1]
}

pub fn least_valuable_piece(from_board: u64, color_to_move: usize, game_state: &GameState) -> (u64, usize) {
    for i in 0..6 {
        let subset = game_state.pieces[i][color_to_move] & from_board;
        if subset != 0u64 {
            return (1u64 << subset.trailing_zeros(), i);
        }
    }
    (0u64, 1000)
}