use super::{EndGameDisplay, Evaluation, MidGameDisplay};

pub const QUEEN_PIECE_VALUE_MG: i16 = 1500;
pub const QUEEN_PIECE_VALUE_EG: i16 = 1600;

pub struct QueenEvaluation {
    amount_of_queens: i16,
}

impl Evaluation for QueenEvaluation {
    fn eval_mg(&self) -> i16 {
        let mut res = 0;
        res += self.amount_of_queens * QUEEN_PIECE_VALUE_MG;
        res
    }
    fn eval_eg(&self) -> i16 {
        let mut res = 0;
        res += self.amount_of_queens * QUEEN_PIECE_VALUE_EG;
        res
    }
}

impl MidGameDisplay for QueenEvaluation {
    fn display_mg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tQueens-MidGame\n");
        res_str.push_str(&format!(
            "\t\tAmount of Queens: {} -> {}\n",
            self.amount_of_queens,
            self.amount_of_queens * QUEEN_PIECE_VALUE_MG
        ));
        res_str.push_str(&format!("\tSum: {}\n", self.eval_mg()));
        res_str
    }
}

impl EndGameDisplay for QueenEvaluation {
    fn display_eg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tQueens-EndGame\n");
        res_str.push_str(&format!(
            "\t\tAmount of Queens: {} -> {}\n",
            self.amount_of_queens,
            self.amount_of_queens * QUEEN_PIECE_VALUE_EG
        ));
        res_str.push_str(&format!("\tSum: {}\n", self.eval_eg()));
        res_str
    }
}

pub fn queen_eval(queen: u64) -> QueenEvaluation {
    QueenEvaluation {
        amount_of_queens: queen.count_ones() as i16,
    }
}
