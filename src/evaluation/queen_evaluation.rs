use super::{Evaluation, MidGameDisplay, EndGameDisplay};

pub const QUEEN_PIECE_VALUE_MG: f64 = 1500.0;
pub const QUEEN_PIECE_VALUE_EG: f64 = 1600.0;

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
        res_str.push_str("\tQueens-MidGame");
        println!("\t\tAmount of Queens: {} -> {}", self.amount_of_queens, self.amount_of_queens as f64 * QUEEN_PIECE_VALUE_MG);
        println!("\tSum: {}", self.eval_mg());
        res_str
    }
}

impl EndGameDisplay for QueenEvaluation {
    fn display_eg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tQueens-EndGame");
        println!("\t\tAmount of Queens: {} -> {}", self.amount_of_queens, self.amount_of_queens as f64 * QUEEN_PIECE_VALUE_EG);
        println!("\tSum: {}", self.eval_eg());
        res_str
    }
}

pub fn queen_eval(queen: u64) -> QueenEvaluation {
    QueenEvaluation { amount_of_queens: queen.count_ones() }
}