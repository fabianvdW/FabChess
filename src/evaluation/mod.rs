pub mod bishop_evaluation;
pub mod king_evaluation;
pub mod knight_evaluation;
pub mod passed_evaluation;
pub mod pawn_evaluation;
pub mod piecewise_evaluation;
pub mod psqt_evaluation;
pub mod queen_evaluation;
pub mod rook_evaluation;

const MG_LIMIT: i16 = 9100;
const EG_LIMIT: i16 = 2350;

use self::bishop_evaluation::{bishop_eval, BishopEvaluation, BISHOP_PIECE_VALUE_MG};
use self::king_evaluation::{king_eval, KingEvaluation};
use self::knight_evaluation::{knight_eval, KnightEvaluation, KNIGHT_PIECE_VALUE_MG};
use self::passed_evaluation::{passed_eval_black, passed_eval_white, PassedEvaluation};
use self::pawn_evaluation::{pawn_eval_black, pawn_eval_white, PawnEvaluation};
use self::piecewise_evaluation::{piecewise_eval, PiecewiseEvaluation};
use self::psqt_evaluation::{psqt_eval, PSQT};
use self::queen_evaluation::{queen_eval, QueenEvaluation, QUEEN_PIECE_VALUE_MG};
use self::rook_evaluation::{rook_eval, RookEvaluation, ROOK_PIECE_VALUE_MG};
use super::bitboards;
use super::board_representation::game_state::{GameState, PieceType};
use super::logging::{log, VERBOSE};
use super::move_generation::movegen;

pub trait Evaluation {
    fn eval_mg(&self) -> i16;
    fn eval_eg(&self) -> i16;
}

pub trait ParallelEvaluation {
    fn eval_mg_eg(&self) -> (i16, i16);
}

pub trait MidGameDisplay {
    fn display_mg(&self) -> String;
}

pub trait EndGameDisplay {
    fn display_eg(&self) -> String;
}

pub struct EvaluationResult {
    pub mg_eval: i16,
    pub eg_eval: i16,
    pub phase: f64,
    pub final_eval: i16,
}

pub fn eval_game_state(g: &GameState) -> EvaluationResult {
    let w_pawns = g.pieces[0][0];
    let w_knights = g.pieces[1][0];
    let w_bishops = g.pieces[2][0];
    let w_rooks = g.pieces[3][0];
    let w_queens = g.pieces[4][0];
    let w_king = g.pieces[5][0];
    let white_pieces = w_pawns | w_knights | w_bishops | w_rooks | w_queens | w_king;

    let b_pawns = g.pieces[0][1];
    let b_knights = g.pieces[1][1];
    let b_bishops = g.pieces[2][1];
    let b_rooks = g.pieces[3][1];
    let b_queens = g.pieces[4][1];
    let b_king = g.pieces[5][1];
    let black_pieces = b_pawns | b_knights | b_bishops | b_rooks | b_queens | b_king;

    let white_pawn_attacks =
        movegen::w_pawn_west_targets(w_pawns) | movegen::w_pawn_east_targets(w_pawns);
    let black_pawn_attacks =
        movegen::b_pawn_west_targets(b_pawns) | movegen::b_pawn_east_targets(b_pawns);
    let all_pawns = w_pawns | b_pawns;
    let pawns_on_board = all_pawns.count_ones() as usize;
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

    let white_pawns_eval = pawn_eval_white(
        w_pawns,
        w_pawns_front_span,
        w_pawns_attack_span,
        black_pawn_attacks,
    );
    let black_pawns_eval = pawn_eval_black(
        b_pawns,
        b_pawns_front_span,
        b_pawns_attack_span,
        white_pawn_attacks,
    );
    let white_passed_eval = passed_eval_white(w_pawns, b_pawns_all_front_spans, black_pieces);
    let black_passed_eval = passed_eval_black(b_pawns, w_pawns_all_front_spans, white_pieces);
    let white_knights_eval = knight_eval(w_knights, white_pawn_attacks, pawns_on_board);
    let black_knights_eval = knight_eval(b_knights, black_pawn_attacks, pawns_on_board);
    let white_bishops_eval = bishop_eval(w_bishops);
    let black_bishops_eval = bishop_eval(b_bishops);
    let white_rooks_eval = rook_eval(w_rooks);
    let black_rooks_eval = rook_eval(b_rooks);
    let white_queen_eval = queen_eval(w_queens);
    let black_queen_eval = queen_eval(b_queens);
    let white_king_eval = king_eval(w_king, w_pawns, b_pawns, true);
    let black_king_eval = king_eval(b_king, b_pawns, w_pawns, false);
    let white_psqt_eval = psqt_eval(
        w_pawns, w_knights, w_bishops, w_rooks, w_queens, w_king, true,
    );
    let black_psqt_eval = psqt_eval(
        b_pawns, b_knights, b_bishops, b_rooks, b_queens, b_king, false,
    );
    let white_piecewise_eval = piecewise_eval(w_pawns, w_rooks, w_bishops, true, all_pawns);
    let black_piecewise_eval = piecewise_eval(b_pawns, b_rooks, b_bishops, false, all_pawns);

    let phase = calculate_phase(
        w_queens, b_queens, w_knights, b_knights, w_bishops, b_bishops, w_rooks, b_rooks,
    );
    let mut mg_eval = 0;
    let mut eg_eval = 0;
    let (
        mut white_pawns_eval_mg,
        mut white_pawns_eval_eg,
        mut black_pawns_eval_mg,
        mut black_pawns_eval_eg,
    ) = (0, 0, 0, 0);
    let (
        mut white_passed_eval_mg,
        mut white_passed_eval_eg,
        mut black_passed_eval_mg,
        mut black_passed_eval_eg,
    ) = (0, 0, 0, 0);
    let (
        mut white_knights_eval_mg,
        mut white_knights_eval_eg,
        mut black_knights_eval_mg,
        mut black_knights_eval_eg,
    ) = (0, 0, 0, 0);
    let (
        mut white_bishops_eval_mg,
        mut white_bishops_eval_eg,
        mut black_bishops_eval_mg,
        mut black_bishops_eval_eg,
    ) = (0, 0, 0, 0);
    let (
        mut white_rooks_eval_mg,
        mut white_rooks_eval_eg,
        mut black_rooks_eval_mg,
        mut black_rooks_eval_eg,
    ) = (0, 0, 0, 0);
    let (
        mut white_queen_eval_mg,
        mut white_queen_eval_eg,
        mut black_queen_eval_mg,
        mut black_queen_eval_eg,
    ) = (0, 0, 0, 0);
    let (
        mut white_king_eval_mg,
        mut white_king_eval_eg,
        mut black_king_eval_mg,
        mut black_king_eval_eg,
    ) = (0, 0, 0, 0);
    let (
        mut white_psqt_eval_mg,
        mut white_psqt_eval_eg,
        mut black_psqt_eval_mg,
        mut black_psqt_eval_eg,
    ) = (0, 0, 0, 0);
    let (
        mut white_piecewise_eval_mg,
        mut white_piecewise_eval_eg,
        mut black_piecewise_eval_mg,
        mut black_piecewise_eval_eg,
    ) = (0, 0, 0, 0);
    //Non parallel eval
    {
        if phase != 128.0 {
            //Do EG evaluation
            white_pawns_eval_eg = white_pawns_eval.eval_eg();
            black_pawns_eval_eg = black_pawns_eval.eval_eg();
            white_knights_eval_eg = white_knights_eval.eval_eg();
            black_knights_eval_eg = black_knights_eval.eval_eg();
            white_bishops_eval_eg = white_bishops_eval.eval_eg();
            black_bishops_eval_eg = black_bishops_eval.eval_eg();
            white_rooks_eval_eg = white_rooks_eval.eval_eg();
            black_rooks_eval_eg = black_rooks_eval.eval_eg();
            white_queen_eval_eg = white_queen_eval.eval_eg();
            black_queen_eval_eg = black_queen_eval.eval_eg();
            white_king_eval_eg = white_king_eval.eval_eg();
            black_king_eval_eg = black_king_eval.eval_eg();
            eg_eval += white_pawns_eval_eg - black_pawns_eval_eg;
            eg_eval += white_knights_eval_eg - black_knights_eval_eg;
            eg_eval += white_bishops_eval_eg - black_bishops_eval_eg;
            eg_eval += white_rooks_eval_eg - black_rooks_eval_eg;
            eg_eval += white_queen_eval_eg - black_queen_eval_eg;
            eg_eval += white_king_eval_eg - black_king_eval_eg;
        }
        if phase != 0.0 {
            //Do MG evaluation
            white_pawns_eval_mg = white_pawns_eval.eval_mg();
            black_pawns_eval_mg = black_pawns_eval.eval_mg();
            white_knights_eval_mg = white_knights_eval.eval_mg();
            black_knights_eval_mg = black_knights_eval.eval_mg();
            white_bishops_eval_mg = white_bishops_eval.eval_mg();
            black_bishops_eval_mg = black_bishops_eval.eval_mg();
            white_rooks_eval_mg = white_rooks_eval.eval_mg();
            black_rooks_eval_mg = black_rooks_eval.eval_mg();
            white_queen_eval_mg = white_queen_eval.eval_mg();
            black_queen_eval_mg = black_queen_eval.eval_mg();
            white_king_eval_mg = white_king_eval.eval_mg();
            black_king_eval_mg = black_king_eval.eval_mg();
            mg_eval += white_pawns_eval_mg - black_pawns_eval_mg;
            mg_eval += white_knights_eval_mg - black_knights_eval_mg;
            mg_eval += white_bishops_eval_mg - black_bishops_eval_mg;
            mg_eval += white_rooks_eval_mg - black_rooks_eval_mg;
            mg_eval += white_queen_eval_mg - black_queen_eval_mg;
            mg_eval += white_king_eval_mg - black_king_eval_mg;
        }
    }
    //Do parallel evaluation
    {
        if phase != 0.0 && phase != 128.0 {
            let _e = white_passed_eval.eval_mg_eg();
            white_passed_eval_mg = _e.0;
            white_passed_eval_eg = _e.1;
            let _e = black_passed_eval.eval_mg_eg();
            black_passed_eval_mg = _e.0;
            black_passed_eval_eg = _e.1;
            let _e = white_psqt_eval.eval_mg_eg();
            white_psqt_eval_mg = _e.0;
            white_psqt_eval_eg = _e.1;
            let _e = black_psqt_eval.eval_mg_eg();
            black_psqt_eval_mg = _e.0;
            black_psqt_eval_eg = _e.1;
            let _e = white_piecewise_eval.eval_mg_eg();
            white_piecewise_eval_mg = _e.0;
            white_piecewise_eval_eg = _e.1;
            let _e = black_piecewise_eval.eval_mg_eg();
            black_piecewise_eval_mg = _e.0;
            black_piecewise_eval_eg = _e.1;
        } else if phase == 0.0 {
            white_passed_eval_eg = white_passed_eval.eval_eg();
            black_passed_eval_eg = black_passed_eval.eval_eg();
            white_psqt_eval_eg = white_psqt_eval.eval_eg();
            black_psqt_eval_eg = black_psqt_eval.eval_eg();
            white_piecewise_eval_eg = white_piecewise_eval.eval_eg();
            black_piecewise_eval_eg = black_piecewise_eval.eval_eg();
        } else if phase == 128.0 {
            white_passed_eval_mg = white_passed_eval.eval_mg();
            black_passed_eval_mg = black_passed_eval.eval_mg();
            white_psqt_eval_mg = white_psqt_eval.eval_mg();
            black_psqt_eval_mg = black_psqt_eval.eval_mg();
            white_piecewise_eval_mg = white_piecewise_eval.eval_mg();
            black_piecewise_eval_mg = black_piecewise_eval.eval_mg();
        }
        mg_eval += white_passed_eval_mg - black_passed_eval_mg;
        mg_eval += white_psqt_eval_mg - black_psqt_eval_mg;
        mg_eval += white_piecewise_eval_mg - black_piecewise_eval_mg;
        eg_eval += white_passed_eval_eg - black_passed_eval_eg;
        eg_eval += white_psqt_eval_eg - black_psqt_eval_eg;
        eg_eval += white_piecewise_eval_eg - black_piecewise_eval_eg;
    }
    //Phasing is done the same way stockfish does it
    let res = ((mg_eval as f64 * phase + eg_eval as f64 * (128.0 - phase)) / 128.0) as i16;
    if VERBOSE {
        make_log(
            &white_pawns_eval,
            white_pawns_eval_mg,
            white_pawns_eval_eg,
            &black_pawns_eval,
            black_pawns_eval_mg,
            black_pawns_eval_eg,
            &white_passed_eval,
            white_passed_eval_mg,
            white_passed_eval_eg,
            &black_passed_eval,
            black_passed_eval_mg,
            black_passed_eval_eg,
            &white_knights_eval,
            white_knights_eval_mg,
            white_knights_eval_eg,
            &black_knights_eval,
            black_knights_eval_mg,
            black_knights_eval_eg,
            &white_bishops_eval,
            white_bishops_eval_mg,
            white_bishops_eval_eg,
            &black_bishops_eval,
            black_bishops_eval_mg,
            black_bishops_eval_eg,
            &white_rooks_eval,
            white_rooks_eval_mg,
            white_rooks_eval_eg,
            &black_rooks_eval,
            black_rooks_eval_mg,
            black_rooks_eval_eg,
            &white_queen_eval,
            white_queen_eval_mg,
            white_queen_eval_eg,
            &black_queen_eval,
            black_queen_eval_mg,
            black_queen_eval_eg,
            &white_king_eval,
            white_king_eval_mg,
            white_king_eval_eg,
            &black_king_eval,
            black_king_eval_mg,
            black_king_eval_eg,
            &white_psqt_eval,
            white_psqt_eval_mg,
            white_psqt_eval_eg,
            &black_psqt_eval,
            black_psqt_eval_mg,
            black_psqt_eval_eg,
            &white_piecewise_eval,
            white_piecewise_eval_mg,
            white_piecewise_eval_eg,
            &black_piecewise_eval,
            black_piecewise_eval_mg,
            black_piecewise_eval_eg,
            phase,
            mg_eval,
            eg_eval,
            res,
        );
    }
    EvaluationResult {
        mg_eval,
        eg_eval,
        phase,
        final_eval: res,
    }
}

pub fn calculate_phase(
    w_queens: u64,
    b_queens: u64,
    w_knights: u64,
    b_knights: u64,
    w_bishops: u64,
    b_bishops: u64,
    w_rooks: u64,
    b_rooks: u64,
) -> f64 {
    let mut npm = (w_queens | b_queens).count_ones() as i16 * QUEEN_PIECE_VALUE_MG
        + (w_bishops | b_bishops).count_ones() as i16 * BISHOP_PIECE_VALUE_MG
        + (w_rooks | b_rooks).count_ones() as i16 * ROOK_PIECE_VALUE_MG
        + (w_knights | b_knights).count_ones() as i16 * KNIGHT_PIECE_VALUE_MG;
    if npm < EG_LIMIT {
        npm = EG_LIMIT;
    }
    if npm > MG_LIMIT {
        npm = MG_LIMIT;
    }
    (npm - EG_LIMIT) as f64 * 128.0 / ((MG_LIMIT - EG_LIMIT) as f64)
}

pub fn piece_value(piece_type: &PieceType, phase: f64) -> i16 {
    if let PieceType::Pawn = piece_type {
        return ((pawn_evaluation::PAWN_PIECE_VALUE_MG as f64 * phase
            + pawn_evaluation::PAWN_PIECE_VALUE_EG as f64 * (128.0 - phase))
            / 128.0) as i16;
    } else if let PieceType::Knight = piece_type {
        return ((knight_evaluation::KNIGHT_PIECE_VALUE_MG as f64 * phase
            + knight_evaluation::KNIGHT_PIECE_VALUE_EG as f64 * (128.0 - phase))
            / 128.0) as i16;
    } else if let PieceType::Bishop = piece_type {
        return ((bishop_evaluation::BISHOP_PIECE_VALUE_MG as f64 * phase
            + bishop_evaluation::BISHOP_PIECE_VALUE_EG as f64 * (128.0 - phase))
            / 128.0) as i16;
    } else if let PieceType::Rook = piece_type {
        return ((rook_evaluation::ROOK_PIECE_VALUE_MG as f64 * phase
            + rook_evaluation::ROOK_PIECE_VALUE_EG as f64 * (128.0 - phase))
            / 128.0) as i16;
    } else if let PieceType::Queen = piece_type {
        return ((queen_evaluation::QUEEN_PIECE_VALUE_MG as f64 * phase
            + queen_evaluation::QUEEN_PIECE_VALUE_EG as f64 * (128.0 - phase))
            / 128.0) as i16;
    } else {
        panic!("Invalid piece type!");
    }
}

pub fn make_log(
    white_pawns_eval: &PawnEvaluation,
    white_pawns_eval_mg: i16,
    white_pawns_eval_eg: i16,
    black_pawns_eval: &PawnEvaluation,
    black_pawns_eval_mg: i16,
    black_pawns_eval_eg: i16,
    white_passed_eval: &PassedEvaluation,
    white_passed_eval_mg: i16,
    white_passed_eval_eg: i16,
    black_passed_eval: &PassedEvaluation,
    black_passed_eval_mg: i16,
    black_passed_eval_eg: i16,
    white_knights_eval: &KnightEvaluation,
    white_knights_eval_mg: i16,
    white_knights_eval_eg: i16,
    black_knights_eval: &KnightEvaluation,
    black_knights_eval_mg: i16,
    black_knights_eval_eg: i16,
    white_bishops_eval: &BishopEvaluation,
    white_bishops_eval_mg: i16,
    white_bishops_eval_eg: i16,
    black_bishops_eval: &BishopEvaluation,
    black_bishops_eval_mg: i16,
    black_bishops_eval_eg: i16,
    white_rooks_eval: &RookEvaluation,
    white_rooks_eval_mg: i16,
    white_rooks_eval_eg: i16,
    black_rooks_eval: &RookEvaluation,
    black_rooks_eval_mg: i16,
    black_rooks_eval_eg: i16,
    white_queen_eval: &QueenEvaluation,
    white_queen_eval_mg: i16,
    white_queen_eval_eg: i16,
    black_queen_eval: &QueenEvaluation,
    black_queen_eval_mg: i16,
    black_queen_eval_eg: i16,
    white_king_eval: &KingEvaluation,
    white_king_eval_mg: i16,
    white_king_eval_eg: i16,
    black_king_eval: &KingEvaluation,
    black_king_eval_mg: i16,
    black_king_eval_eg: i16,
    white_psqt_eval: &PSQT,
    white_psqt_eval_mg: i16,
    white_psqt_eval_eg: i16,
    black_psqt_eval: &PSQT,
    black_psqt_eval_mg: i16,
    black_psqt_eval_eg: i16,
    white_piecewise_eval: &PiecewiseEvaluation,
    white_piecewise_eval_mg: i16,
    white_piecewise_eval_eg: i16,
    black_piecewise_eval: &PiecewiseEvaluation,
    black_piecewise_eval_mg: i16,
    black_piecewise_eval_eg: i16,
    phase: f64,
    mg_eval: i16,
    eg_eval: i16,
    res: i16,
) {
    let mut verbose_mg = 0;
    let mut verbose_eg = 0;
    //Pawns
    {
        log("White\n");
        if phase != 0.0 {
            log(&white_pawns_eval.display_mg());
        }
        if phase != 128.0 {
            log(&white_pawns_eval.display_eg());
        }
        log("Black\n");
        if phase != 0.0 {
            log(&black_pawns_eval.display_mg());
        }
        if phase != 128.0 {
            log(&black_pawns_eval.display_eg());
        }
        if phase != 0.0 {
            log(&format!(
                "MGEval: {} + {} - {} = {}\n",
                verbose_mg,
                white_pawns_eval_mg,
                black_pawns_eval_mg,
                verbose_mg + white_pawns_eval_mg - black_pawns_eval_mg
            ));
            verbose_mg += white_pawns_eval_mg - black_pawns_eval_mg;
        }
        if phase != 128.0 {
            log(&format!(
                "EGEval: {} + {} - {} = {}\n",
                verbose_eg,
                white_pawns_eval_eg,
                black_pawns_eval_eg,
                verbose_eg + white_pawns_eval_eg - black_pawns_eval_eg
            ));
            verbose_eg += white_pawns_eval_eg - black_pawns_eval_eg;
        }
    }
    //Passed
    {
        log("White\n");
        if phase != 0.0 {
            log(&white_passed_eval.display_mg());
        }
        if phase != 128.0 {
            log(&white_passed_eval.display_eg());
        }
        log("Black\n");
        if phase != 0.0 {
            log(&black_passed_eval.display_mg());
        }
        if phase != 128.0 {
            log(&black_passed_eval.display_eg());
        }
        if phase != 0.0 {
            log(&format!(
                "MGEval: {} + {} - {} = {}\n",
                verbose_mg,
                white_passed_eval_mg,
                black_passed_eval_mg,
                verbose_mg + white_passed_eval_mg - black_passed_eval_mg
            ));
            verbose_mg += white_passed_eval_mg - black_passed_eval_mg;
        }
        if phase != 128.0 {
            log(&format!(
                "EGEval: {} + {} - {} = {}\n",
                verbose_eg,
                white_passed_eval_eg,
                black_passed_eval_eg,
                verbose_eg + white_passed_eval_eg - black_passed_eval_eg
            ));
            verbose_eg += white_passed_eval_eg - black_passed_eval_eg;
        }
    }
    //Knights
    {
        log("White\n");
        if phase != 0.0 {
            log(&white_knights_eval.display_mg());
        }
        if phase != 128.0 {
            log(&white_knights_eval.display_eg());
        }
        log("Black\n");
        if phase != 0.0 {
            log(&black_knights_eval.display_mg());
        }
        if phase != 128.0 {
            log(&black_knights_eval.display_eg());
        }
        if phase != 0.0 {
            log(&format!(
                "MGEval: {} + {} - {} = {}\n",
                verbose_mg,
                white_knights_eval_mg,
                black_knights_eval_mg,
                verbose_mg + white_knights_eval_mg - black_knights_eval_mg
            ));
            verbose_mg += white_knights_eval_mg - black_knights_eval_mg;
        }
        if phase != 128.0 {
            log(&format!(
                "EGEval: {} + {} - {} = {}\n",
                verbose_eg,
                white_knights_eval_eg,
                black_knights_eval_eg,
                verbose_eg + white_knights_eval_eg - black_knights_eval_eg
            ));
            verbose_eg += white_knights_eval_eg - black_knights_eval_eg;
        }
    }
    //Bishops
    {
        log("White\n");
        if phase != 0.0 {
            log(&white_bishops_eval.display_mg());
        }
        if phase != 128.0 {
            log(&white_bishops_eval.display_eg());
        }
        log("Black\n");
        if phase != 0.0 {
            log(&black_bishops_eval.display_mg());
        }
        if phase != 128.0 {
            log(&black_bishops_eval.display_eg());
        }
        if phase != 0.0 {
            log(&format!(
                "MGEval: {} + {} - {} = {}\n",
                verbose_mg,
                white_bishops_eval_mg,
                black_bishops_eval_mg,
                verbose_mg + white_bishops_eval_mg - black_bishops_eval_mg
            ));
            verbose_mg += white_bishops_eval_mg - black_bishops_eval_mg;
        }
        if phase != 128.0 {
            log(&format!(
                "EGEval: {} + {} - {} = {}\n",
                verbose_eg,
                white_bishops_eval_eg,
                black_bishops_eval_eg,
                verbose_eg + white_bishops_eval_eg - black_bishops_eval_eg
            ));
            verbose_eg += white_bishops_eval_eg - black_bishops_eval_eg;
        }
    }
    //Rooks
    {
        log("White\n");
        if phase != 0.0 {
            log(&white_rooks_eval.display_mg());
        }
        if phase != 128.0 {
            log(&white_rooks_eval.display_eg());
        }
        log("Black\n");
        if phase != 0.0 {
            log(&black_rooks_eval.display_mg());
        }
        if phase != 128.0 {
            log(&black_rooks_eval.display_eg());
        }
        if phase != 0.0 {
            log(&format!(
                "MGEval: {} + {} - {} = {}\n",
                verbose_mg,
                white_rooks_eval_mg,
                black_rooks_eval_mg,
                verbose_mg + white_rooks_eval_mg - black_rooks_eval_mg
            ));
            verbose_mg += white_rooks_eval_mg - black_rooks_eval_mg;
        }
        if phase != 128.0 {
            log(&format!(
                "EGEval: {} + {} - {} = {}\n",
                verbose_eg,
                white_rooks_eval_eg,
                black_rooks_eval_eg,
                verbose_eg + white_rooks_eval_eg - black_rooks_eval_eg
            ));
            verbose_eg += white_rooks_eval_eg - black_rooks_eval_eg;
        }
    }
    //Queen(s)
    {
        log("White\n");
        if phase != 0.0 {
            log(&white_queen_eval.display_mg());
        }
        if phase != 128.0 {
            log(&white_queen_eval.display_eg());
        }
        log("Black\n");
        if phase != 0.0 {
            log(&black_queen_eval.display_mg());
        }
        if phase != 128.0 {
            log(&black_queen_eval.display_eg());
        }
        if phase != 0.0 {
            log(&format!(
                "MGEval: {} + {} - {} = {}\n",
                verbose_mg,
                white_queen_eval_mg,
                black_queen_eval_mg,
                verbose_mg + white_queen_eval_mg - black_queen_eval_mg
            ));
            verbose_mg += white_queen_eval_mg - black_queen_eval_mg;
        }
        if phase != 128.0 {
            log(&format!(
                "EGEval: {} + {} - {} = {}\n",
                verbose_eg,
                white_queen_eval_eg,
                black_queen_eval_eg,
                verbose_eg + white_queen_eval_eg - black_queen_eval_eg
            ));
            verbose_eg += white_queen_eval_eg - black_queen_eval_eg;
        }
    }
    //King safety
    {
        log("White\n");
        if phase != 0.0 {
            log(&white_king_eval.display_mg());
        }
        if phase != 128.0 {
            log(&white_king_eval.display_eg());
        }
        log("Black\n");
        if phase != 0.0 {
            log(&black_king_eval.display_mg());
        }
        if phase != 128.0 {
            log(&black_king_eval.display_eg());
        }
        if phase != 0.0 {
            log(&format!(
                "MGEval: {} + {} - {} = {}\n",
                verbose_mg,
                white_king_eval_mg,
                black_king_eval_mg,
                verbose_mg + white_king_eval_mg - black_king_eval_mg
            ));
            verbose_mg += white_king_eval_mg - black_king_eval_mg;
        }
        if phase != 128.0 {
            log(&format!(
                "EGEval: {} + {} - {} = {}\n",
                verbose_eg,
                white_king_eval_eg,
                black_king_eval_eg,
                verbose_eg + white_king_eval_eg - black_king_eval_eg
            ));
            verbose_eg += white_king_eval_eg - black_king_eval_eg;
        }
    }
    //PSQT
    {
        log("White\n");
        if phase != 0.0 {
            log(&white_psqt_eval.display_mg());
        }
        if phase != 128.0 {
            log(&white_psqt_eval.display_eg());
        }
        log("Black\n");
        if phase != 0.0 {
            log(&black_psqt_eval.display_mg());
        }
        if phase != 128.0 {
            log(&black_psqt_eval.display_eg());
        }
        if phase != 0.0 {
            log(&format!(
                "MGEval: {} + {} - {} = {}\n",
                verbose_mg,
                white_psqt_eval_mg,
                black_psqt_eval_mg,
                verbose_mg + white_psqt_eval_mg - black_psqt_eval_mg
            ));
            verbose_mg += white_psqt_eval_mg - black_psqt_eval_mg;
        }
        if phase != 128.0 {
            log(&format!(
                "EGEval: {} + {} - {} = {}\n",
                verbose_eg,
                white_psqt_eval_eg,
                black_psqt_eval_eg,
                verbose_eg + white_psqt_eval_eg - black_psqt_eval_eg
            ));
            verbose_eg += white_psqt_eval_eg - black_psqt_eval_eg;
        }
    }
    //Piecwise
    {
        log("White\n");
        if phase != 0.0 {
            log(&white_piecewise_eval.display_mg());
        }
        if phase != 128.0 {
            log(&white_piecewise_eval.display_eg());
        }
        log("Black\n");
        if phase != 0.0 {
            log(&black_piecewise_eval.display_mg());
        }
        if phase != 128.0 {
            log(&black_piecewise_eval.display_eg());
        }
        if phase != 0.0 {
            log(&format!(
                "MGEval: {} + {} - {} = {}\n",
                verbose_mg,
                white_piecewise_eval_mg,
                black_piecewise_eval_mg,
                verbose_mg + white_piecewise_eval_mg - black_piecewise_eval_mg
            ));
            //verbose_mg += white_piecewise_eval_mg - black_piecewise_eval_mg;
        }
        if phase != 128.0 {
            log(&format!(
                "EGEval: {} + {} - {} = {}\n",
                verbose_eg,
                white_piecewise_eval_eg,
                black_piecewise_eval_eg,
                verbose_eg + white_piecewise_eval_eg - black_piecewise_eval_eg
            ));
            //verbose_eg += white_piecewise_eval_eg - black_piecewise_eval_eg;
        }
    }
    log(&format!("Phase: {}\n", phase));
    log(&format!(
        "=> ({} * {} + {}*(128-{}))/128={}\n",
        mg_eval, phase, eg_eval, phase, res
    ));
    log(&format!("{}", res));
}
