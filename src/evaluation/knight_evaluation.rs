use super::{Evaluation, MidGameDisplay, EndGameDisplay};

pub const KNIGHT_PIECE_VALUE_MG: f64 = 500.0;
pub const KNIGHT_PIECE_VALUE_EG: f64 = 500.0;
pub const KNIGHT_VALUE_WITH_PAWNS: [f64; 17] = [-30.0, -27.5, -25.0, -22.5, -20.0, -17.5, -15.0, -12.5, -10.0, -7.5, -5.0, -2.5, 0.0, 2.5, 5.0, 7.5, 10.0];
pub const KNIGHT_SUPPORTED_BY_PAWN: f64 = 30.0;

pub struct KnightEvaluation {
    amount_of_knights: u32,
    pawns_on_board: usize,
    supported_knights: u32,
}

impl Evaluation for KnightEvaluation {
    fn eval_mg(&self) -> f64 {
        let mut res = 0.0;
        res += self.amount_of_knights as f64 * (KNIGHT_PIECE_VALUE_MG + KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board]);
        res += self.supported_knights as f64 * KNIGHT_SUPPORTED_BY_PAWN;
        res
    }
    fn eval_eg(&self) -> f64 {
        let mut res = 0.0;
        res += self.amount_of_knights as f64 * (KNIGHT_PIECE_VALUE_EG + KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board]);
        res += self.supported_knights as f64 * KNIGHT_SUPPORTED_BY_PAWN;
        res
    }
}

impl MidGameDisplay for KnightEvaluation {
    fn display_mg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tKnights-MidGame\n");
        res_str.push_str(&format!("\t\tAmount of Knights: {} * ({} + {}) -> {}\n", self.amount_of_knights, KNIGHT_PIECE_VALUE_MG, KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board]
                 , self.amount_of_knights as f64 * (KNIGHT_PIECE_VALUE_MG + KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board])));
        res_str.push_str(&format!("\t\tSupported Knights: {} -> {}\n", self.supported_knights, self.supported_knights as f64 * KNIGHT_SUPPORTED_BY_PAWN));
        res_str.push_str(&format!("\tSum: {}\n", self.eval_mg()));
        res_str
    }
}

impl EndGameDisplay for KnightEvaluation {
    fn display_eg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tKnights-EndGame\n");
        res_str.push_str(&format!("\t\tAmount of Knights: {} * ({} + {}) -> {}\n", self.amount_of_knights, KNIGHT_PIECE_VALUE_EG, KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board]
                 , self.amount_of_knights as f64 * (KNIGHT_PIECE_VALUE_EG + KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board])));
        res_str.push_str(&format!("\t\tSupported Knights: {} -> {}\n", self.supported_knights, self.supported_knights as f64 * KNIGHT_SUPPORTED_BY_PAWN));
        res_str.push_str(&format!("\tSum: {}\n", self.eval_eg()));
        res_str
    }
}

pub fn knight_eval(knight: u64, my_pawns_attacks: u64, pawns_on_board: usize) -> KnightEvaluation {
    let supported_knights = (my_pawns_attacks & knight).count_ones();
    KnightEvaluation {
        amount_of_knights: knight.count_ones(),
        pawns_on_board,
        supported_knights,
    }
}