pub mod pawn_evaluation;
pub mod knight_evaluation;
pub mod bishop_evaluation;
pub mod rook_evaluation;
pub mod queen_evaluation;
pub mod king_evaluation;
pub mod psqt_evaluation;
pub mod passed_evaluation;

const VERBOSE: bool = true;

use super::move_generation::movegen;
use super::board_representation::game_state::GameState;
use super::bitboards;
use self::pawn_evaluation::{pawn_eval_white, pawn_eval_black};
use self::passed_evaluation::{passed_eval_white, passed_eval_black};
use self::knight_evaluation::{knight_eval, KNIGHT_PIECE_VALUE_MG};
use self::bishop_evaluation::{bishop_eval, BISHOP_PIECE_VALUE_MG};
use self::rook_evaluation::{rook_eval, ROOK_PIECE_VALUE_MG};
use self::queen_evaluation::{queen_eval, QUEEN_PIECE_VALUE_MG};
use self::king_evaluation::king_eval;
use self::psqt_evaluation::psqt_eval;

pub trait Evaluation {
    fn eval_mg(&self) -> f64;
    fn eval_eg(&self) -> f64;
}

pub trait ParallelEvaluation {
    fn eval_mg_eg(&self) -> (f64, f64);
}

pub trait MidGameDisplay {
    fn display_mg(&self) -> String;
}

pub trait EndGameDisplay {
    fn display_eg(&self) -> String;
}

pub fn eval_game_state(g: &GameState) -> f64 {
    //White
    let w_pawns = g.pieces[0][0];
    let w_knights = g.pieces[1][0];
    let w_bishops = g.pieces[2][0];
    let w_rooks = g.pieces[3][0];
    let w_queens = g.pieces[4][0];
    let w_king = g.pieces[5][0];

    let white_pieces = w_pawns | w_knights | w_bishops | w_rooks | w_queens | w_king;
    //Black
    let b_pawns = g.pieces[0][1];
    let b_knights = g.pieces[1][1];
    let b_bishops = g.pieces[2][1];
    let b_rooks = g.pieces[3][1];
    let b_queens = g.pieces[4][1];
    let b_king = g.pieces[5][1];

    let black_pieces = b_pawns | b_knights | b_bishops | b_rooks | b_queens | b_king;

    //Calculate the Phase of the game
    let mg_limit = 9100.0;
    let eg_limit = 2350.0;
    let mut npm = (w_queens | b_queens).count_ones() as f64 * QUEEN_PIECE_VALUE_MG +
        (w_bishops | b_bishops).count_ones() as f64 * BISHOP_PIECE_VALUE_MG +
        (w_rooks | b_rooks).count_ones() as f64 * ROOK_PIECE_VALUE_MG +
        (w_knights | b_knights).count_ones() as f64 * KNIGHT_PIECE_VALUE_MG;
    if npm < eg_limit {
        npm = eg_limit;
    }
    if npm > mg_limit {
        npm = mg_limit;
    }
    let phase = (npm - eg_limit) * 128.0 / (mg_limit - eg_limit);


    let mut mg_eval = 0.0;
    let mut eg_eval = 0.0;

    let white_pawn_attacks = movegen::w_pawn_west_targets(w_pawns) | movegen::w_pawn_east_targets(w_pawns);
    let black_pawn_attacks = movegen::b_pawn_west_targets(b_pawns) | movegen::b_pawn_east_targets(b_pawns);
    let all_pawns = w_pawns | b_pawns;
    let pawns_on_board = all_pawns.count_ones() as usize;
    //Pawns
    {
        //White general bitboards
        let w_pawns_front_span = bitboards::w_front_span(w_pawns);
        let w_pawns_west_attack_front_span = bitboards::west_one(w_pawns_front_span);
        let w_pawns_east_attack_front_span = bitboards::east_one(w_pawns_front_span);
        let w_pawns_attack_span = w_pawns_east_attack_front_span | w_pawns_west_attack_front_span;
        let w_pawns_all_front_spans = w_pawns_front_span | w_pawns_attack_span;
        //Black general bitboards
        let b_pawns_front_span = bitboards::b_front_span(b_pawns);
        let b_pawns_west_attack_front_span = bitboards::west_one(b_pawns_front_span);
        let b_pawns_east_attack_front_span = bitboards::east_one(b_pawns_front_span);
        let b_pawns_attack_span = b_pawns_east_attack_front_span | b_pawns_west_attack_front_span;
        let b_pawns_all_front_spans = b_pawns_front_span | b_pawns_attack_span;

        let white_pawns_eval = pawn_eval_white(w_pawns, w_pawns_front_span, w_pawns_attack_span, black_pawn_attacks);
        let black_pawns_eval = pawn_eval_black(b_pawns, b_pawns_front_span, b_pawns_attack_span, white_pawn_attacks);
        let (white_passed_eval_mg, white_passed_eval_eg) = passed_eval_white(w_pawns, b_pawns_all_front_spans, black_pieces).eval_mg_eg();
        let (black_passed_eval_mg, black_passed_eval_eg) = passed_eval_black(b_pawns, w_pawns_all_front_spans, white_pieces).eval_mg_eg();
        mg_eval += white_pawns_eval.eval_mg() + white_passed_eval_mg - black_pawns_eval.eval_mg() - black_passed_eval_mg;
        eg_eval += white_pawns_eval.eval_eg() + white_passed_eval_eg - black_pawns_eval.eval_eg() - black_passed_eval_eg;
    }
    //Knights
    {
        let white_knights_eval = knight_eval(w_knights, white_pawn_attacks, pawns_on_board);
        let black_knights_eval = knight_eval(b_knights, black_pawn_attacks, pawns_on_board);
        mg_eval += white_knights_eval.eval_mg() - black_knights_eval.eval_mg();
        eg_eval += white_knights_eval.eval_eg() - black_knights_eval.eval_eg();
    }
    //Bishops
    {
        let white_bishops_eval = bishop_eval(w_bishops);
        let black_bishops_eval = bishop_eval(b_bishops);
        mg_eval += white_bishops_eval.eval_mg() - black_bishops_eval.eval_mg();
        eg_eval += white_bishops_eval.eval_eg() - black_bishops_eval.eval_eg();
    }
    //Rooks
    {
        let white_rooks_eval = rook_eval(w_rooks);
        let black_rooks_eval = rook_eval(b_rooks);
        mg_eval += white_rooks_eval.eval_mg() - black_rooks_eval.eval_mg();
        eg_eval += white_rooks_eval.eval_eg() - black_rooks_eval.eval_eg();
    }
    //Queen(s)
    {
        let white_queen_eval = queen_eval(w_queens);
        let black_queen_eval = queen_eval(b_queens);
        mg_eval += white_queen_eval.eval_mg() - black_queen_eval.eval_mg();
        eg_eval += white_queen_eval.eval_eg() - black_queen_eval.eval_eg();
    }
    //King Safety
    {
        let white_king_eval = king_eval(w_king, w_pawns, b_pawns, true);
        let black_king_eval = king_eval(b_king, b_pawns, w_pawns, false);
        mg_eval += white_king_eval.eval_mg() - black_king_eval.eval_mg();
        eg_eval += white_king_eval.eval_eg() - black_king_eval.eval_eg();
    }
    //PSQT
    {
        let (white_psqt_eval_mg, white_psqt_eval_eg) = psqt_eval(w_pawns, w_knights, w_bishops, w_rooks, w_queens, w_king, true).eval_mg_eg();
        let (black_psqt_eval_mg, black_psqt_eval_eg) = psqt_eval(b_pawns, b_knights, b_bishops, b_rooks, b_queens, b_king, false).eval_mg_eg();
        mg_eval += white_psqt_eval_mg - black_psqt_eval_mg;
        eg_eval += white_psqt_eval_eg - black_psqt_eval_eg;
    }
    let res = (mg_eval * phase + eg_eval * (128.0 - phase)) / 128.0;
    if VERBOSE {
        println!("Phase: {}", phase);
        println!("=> ({} * {} + {}*(128-{}))/128={}", mg_eval, phase, eg_eval, phase, res);
    }
    res / 100.0
}