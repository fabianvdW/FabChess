use super::{Evaluation, MidGameDisplay, EndGameDisplay};

pub const ROOK_PIECE_VALUE_MG: f64 = 710.0;
pub const ROOK_PIECE_VALUE_EG: f64 = 920.0;

pub const ROOK_ON_OPEN_FILE_BONUS: f64 = 20.0;
pub const ROOK_ON_SEVENTH: f64 = 10.0;

pub struct RookEvaluation {
    amount_of_rooks: u32
}

impl Evaluation for RookEvaluation {
    fn eval_mg(&self) -> f64 {
        let mut res = 0.0;
        res += self.amount_of_rooks as f64 * ROOK_PIECE_VALUE_MG;
        res
    }
    fn eval_eg(&self) -> f64 {
        let mut res = 0.0;
        res += self.amount_of_rooks as f64 * ROOK_PIECE_VALUE_EG;
        res
    }
}

impl MidGameDisplay for RookEvaluation {
    fn display_mg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tRooks-MidGame");
        println!("\t\tAmount of Rooks: {} -> {}", self.amount_of_rooks, self.amount_of_rooks as f64 * ROOK_PIECE_VALUE_MG);
        println!("\tSum: {}", self.eval_mg());
        res_str
    }
}

impl EndGameDisplay for RookEvaluation {
    fn display_eg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tRooks-EndGame");
        println!("\t\tAmount of Rooks: {} -> {}", self.amount_of_rooks, self.amount_of_rooks as f64 * ROOK_PIECE_VALUE_EG);
        println!("\tSum: {}", self.eval_eg());
        res_str
    }
}

pub fn rook_eval(rook: u64) -> RookEvaluation {
    //missing Rooks on open file and rooks on sevent rank
    RookEvaluation { amount_of_rooks: rook.count_ones() }
}