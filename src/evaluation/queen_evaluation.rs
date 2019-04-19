use super::{Evaluation, MidGameDisplay, EndGameDisplay};

pub const QUEEN_PIECE_VALUE_MG: f64 = 1200.0;
pub const QUEEN_PIECE_VALUE_EG: f64 = 1250.0;

pub struct QueenEvaluation {
    amount_of_queens: u32
}

impl Evaluation for QueenEvaluation {
    fn eval_mg(&self) -> f64 {
        let mut res = 0.0;
        res += self.amount_of_queens as f64 * QUEEN_PIECE_VALUE_MG;
        res
    }
    fn eval_eg(&self) -> f64 {
        let mut res = 0.0;
        res += self.amount_of_queens as f64 * QUEEN_PIECE_VALUE_EG;
        res
    }
}

impl MidGameDisplay for QueenEvaluation {
    fn display_mg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tQueens-MidGame\n");
        res_str.push_str(&format!("\t\tAmount of Queens: {} -> {}\n", self.amount_of_queens, self.amount_of_queens as f64 * QUEEN_PIECE_VALUE_MG));
        res_str.push_str(&format!("\tSum: {}\n", self.eval_mg()));
        res_str
    }
}

impl EndGameDisplay for QueenEvaluation {
    fn display_eg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tQueens-EndGame\n");
        res_str.push_str(&format!("\t\tAmount of Queens: {} -> {}\n", self.amount_of_queens, self.amount_of_queens as f64 * QUEEN_PIECE_VALUE_EG));
        res_str.push_str(&format!("\tSum: {}\n", self.eval_eg()));
        res_str
    }
}

pub fn queen_eval(queen: u64) -> QueenEvaluation {
    QueenEvaluation { amount_of_queens: queen.count_ones() }
}