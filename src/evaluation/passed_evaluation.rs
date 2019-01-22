use super::{Evaluation, ParallelEvaluation, MidGameDisplay, EndGameDisplay, bitboards};

const PAWN_PASSED_VALUES_MG: [f64; 7] = [0.0, -20.0, -10.0, 10.0, 70.0, 120.0, 200.0];
const PAWN_PASSED_NOT_BLOCKED_VALUES_MG: [f64; 7] = [0.0, 0.0, 0.0, 25.0, 40.0, 130.0, 210.0];
const PAWN_PASSED_VALUES_EG: [f64; 7] = [0.0, -40.0, -20.0, 20.0, 140.0, 240.0, 400.0];
const PAWN_PASSED_NOT_BLOCKED_VALUES_EG: [f64; 7] = [0.0, 0.0, 0.0, 50.0, 80.0, 260.0, 420.0];

pub struct PassedEvaluation {
    passed_pawns: u64,
    passed_not_blocked_pawns: u64,
    is_white: bool,
}

impl PassedEvaluation {
    pub fn new(passed_pawns: u64, passed_not_blocked_pawns: u64, is_white: bool) -> PassedEvaluation {
        PassedEvaluation { passed_pawns, passed_not_blocked_pawns, is_white }
    }
    pub fn copy(&self) -> PassedEvaluation {
        PassedEvaluation::new(self.passed_pawns, self.passed_not_blocked_pawns, self.is_white)
    }
}

impl Evaluation for PassedEvaluation {
    fn eval_mg(&self) -> f64 {
        let mut res = 0.0;
        let mut cp = self.copy();
        while cp.passed_pawns != 0u64 {
            let idx = cp.passed_pawns.trailing_zeros() as usize;
            res += PAWN_PASSED_VALUES_MG[if cp.is_white { idx / 8 } else { 7 - idx / 8 }];
            cp.passed_pawns ^= 1u64 << idx;
        }
        while cp.passed_not_blocked_pawns != 0u64 {
            let idx = cp.passed_not_blocked_pawns.trailing_zeros() as usize;
            res += PAWN_PASSED_NOT_BLOCKED_VALUES_MG[if cp.is_white { idx / 8 } else { 7 - idx / 8 }];
            cp.passed_not_blocked_pawns ^= 1u64 << idx;
        }
        res
    }
    fn eval_eg(&self) -> f64 {
        let mut res = 0.0;
        let mut cp = self.copy();
        while cp.passed_pawns != 0u64 {
            let idx = cp.passed_pawns.trailing_zeros() as usize;
            res += PAWN_PASSED_VALUES_EG[if cp.is_white { idx / 8 } else { 7 - idx / 8 }];
            cp.passed_pawns ^= 1u64 << idx;
        }
        while cp.passed_not_blocked_pawns != 0u64 {
            let idx = cp.passed_not_blocked_pawns.trailing_zeros() as usize;
            res += PAWN_PASSED_NOT_BLOCKED_VALUES_EG[if cp.is_white { idx / 8 } else { 7 - idx / 8 }];
            cp.passed_not_blocked_pawns ^= 1u64 << idx;
        }
        res
    }
}

impl ParallelEvaluation for PassedEvaluation {
    fn eval_mg_eg(&self) -> (f64, f64) {
        let mut mg = 0.0;
        let mut eg = 0.0;
        let mut cp = self.copy();
        while cp.passed_pawns != 0u64 {
            let idx = cp.passed_pawns.trailing_zeros() as usize;
            mg += PAWN_PASSED_VALUES_MG[if cp.is_white { idx / 8 } else { 7 - idx / 8 }];
            eg += PAWN_PASSED_VALUES_EG[if cp.is_white { idx / 8 } else { 7 - idx / 8 }];
            cp.passed_pawns ^= 1u64 << idx;
        }
        while cp.passed_not_blocked_pawns != 0u64 {
            let idx = cp.passed_not_blocked_pawns.trailing_zeros() as usize;
            mg += PAWN_PASSED_NOT_BLOCKED_VALUES_MG[if cp.is_white { idx / 8 } else { 7 - idx / 8 }];
            eg += PAWN_PASSED_NOT_BLOCKED_VALUES_EG[if cp.is_white { idx / 8 } else { 7 - idx / 8 }];
            cp.passed_not_blocked_pawns ^= 1u64 << idx;
        }
        (mg, eg)
    }
}

impl MidGameDisplay for PassedEvaluation {
    fn display_mg(&self) -> String {
        let mut cp = self.copy();
        let mut passer_score = 0.0;
        while cp.passed_pawns != 0u64 {
            let idx = cp.passed_pawns.trailing_zeros() as usize;
            passer_score += PAWN_PASSED_VALUES_MG[if cp.is_white { idx / 8 } else { 7 - idx / 8 }];
            cp.passed_pawns ^= 1u64 << idx;
        }
        let mut passed_not_blocked_score = 0.0;
        while cp.passed_not_blocked_pawns != 0u64 {
            let idx = cp.passed_not_blocked_pawns.trailing_zeros() as usize;
            passed_not_blocked_score += PAWN_PASSED_NOT_BLOCKED_VALUES_MG[if cp.is_white { idx / 8 } else { 7 - idx / 8 }];
            cp.passed_not_blocked_pawns ^= 1u64 << idx;
        }

        let mut res_str = String::new();
        res_str.push_str("\tPassed-MidGame");
        println!("\t\tPassed Pawns: {} -> {}", self.passed_pawns.count_ones(), passer_score);
        println!("\t\tPassed and not blocked Pawns:\t{} -> {}", self.passed_not_blocked_pawns.count_ones(), passed_not_blocked_score);
        println!("\tSum: {}", passer_score + passed_not_blocked_score);
        res_str
    }
}

impl EndGameDisplay for PassedEvaluation {
    fn display_eg(&self) -> String {
        let mut cp = self.copy();
        let mut passer_score = 0.0;
        while cp.passed_pawns != 0u64 {
            let idx = cp.passed_pawns.trailing_zeros() as usize;
            passer_score += PAWN_PASSED_VALUES_EG[if cp.is_white { idx / 8 } else { 7 - idx / 8 }];
            cp.passed_pawns ^= 1u64 << idx;
        }
        let mut passed_not_blocked_score = 0.0;
        while cp.passed_not_blocked_pawns != 0u64 {
            let idx = cp.passed_not_blocked_pawns.trailing_zeros() as usize;
            passed_not_blocked_score += PAWN_PASSED_NOT_BLOCKED_VALUES_EG[if cp.is_white { idx / 8 } else { 7 - idx / 8 }];
            cp.passed_not_blocked_pawns ^= 1u64 << idx;
        }

        let mut res_str = String::new();
        res_str.push_str("\tPassed-EndGame");
        println!("\t\tPassed Pawns: {} -> {}", self.passed_pawns.count_ones(), passer_score);
        println!("\t\tPassed and not blocked Pawns:\t{} -> {}", self.passed_not_blocked_pawns.count_ones(), passed_not_blocked_score);
        println!("\tSum: {}", passer_score + passed_not_blocked_score);
        res_str
    }
}

pub fn passed_eval_white(w_pawns: u64, b_pawns_all_front_spans: u64, enemy_pieces: u64) -> PassedEvaluation {
    let (passed_pawns, passed_not_blocked) = w_passed_pawns(w_pawns & !bitboards::w_rear_span(w_pawns), b_pawns_all_front_spans, enemy_pieces);
    PassedEvaluation{passed_pawns,passed_not_blocked_pawns:passed_not_blocked,is_white:true}
}

pub fn passed_eval_black(b_pawns: u64, w_pawns_all_front_spans: u64, enemy_pieces: u64) -> PassedEvaluation {
    let (passed_pawns, passed_not_blocked) = b_passed_pawns(b_pawns & !bitboards::b_rear_span(b_pawns), w_pawns_all_front_spans, enemy_pieces);
    PassedEvaluation{passed_pawns,passed_not_blocked_pawns:passed_not_blocked,is_white:false}
}

pub fn w_passed_pawns(w_pawns: u64, b_pawns_all_front_spans: u64, enemy_pieces: u64) -> (u64, u64) {
    let mut passed_board = w_pawns & !b_pawns_all_front_spans;
    let passed_board_cl = passed_board.clone();
    let mut passed_not_blocked = 0u64;
    while passed_board != 0u64 {
        let idx = passed_board.trailing_zeros() as usize;
        let piece = 1u64 << idx;
        if bitboards::w_front_span(piece) & enemy_pieces == 0u64 {
            passed_not_blocked |= piece;
        }
        passed_board ^= piece;
    }
    (passed_board_cl, passed_not_blocked)
}

pub fn b_passed_pawns(b_pawns: u64, w_pawns_all_front_spans: u64, enemy_pieces: u64) -> (u64, u64) {
    let mut passed_board = b_pawns & !w_pawns_all_front_spans;
    let passed_board_cl = passed_board.clone();
    let mut passed_not_blocked = 0u64;
    while passed_board != 0u64 {
        let idx = passed_board.trailing_zeros() as usize;
        let piece = 1u64 << idx;
        if bitboards::b_front_span(piece) & enemy_pieces == 0u64 {
            passed_not_blocked |= piece;
        }
        passed_board ^= piece;
    }
    (passed_board_cl, passed_not_blocked)
}