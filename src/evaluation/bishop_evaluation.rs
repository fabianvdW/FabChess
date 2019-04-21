use super::{EndGameDisplay, Evaluation, MidGameDisplay};

pub const BISHOP_PIECE_VALUE_EG: f64 = 510.0;
pub const BISHOP_PIECE_VALUE_MG: f64 = 510.0;
pub const BISHOP_PAIR_BONUS: f64 = 50.0;

pub struct BishopEvaluation {
    amount_of_bishops: u32,
    bishop_pair: bool,
}

impl Evaluation for BishopEvaluation {
    fn eval_mg(&self) -> f64 {
        let mut res = 0.0;
        res += self.amount_of_bishops as f64 * BISHOP_PIECE_VALUE_MG;
        if self.bishop_pair {
            res += BISHOP_PAIR_BONUS;
        }
        res
    }
    fn eval_eg(&self) -> f64 {
        let mut res = 0.0;
        res += self.amount_of_bishops as f64 * BISHOP_PIECE_VALUE_EG;
        if self.bishop_pair {
            res += BISHOP_PAIR_BONUS;
        }
        res
    }
}

impl MidGameDisplay for BishopEvaluation {
    fn display_mg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tBishops-MidGame\n");
        res_str.push_str(&format!(
            "\t\tAmount of Bishops: {} -> {}\n",
            self.amount_of_bishops,
            self.amount_of_bishops as f64 * BISHOP_PIECE_VALUE_MG
        ));
        if self.bishop_pair {
            res_str.push_str(&format!("\t\tBishop Pair:       {}\n", BISHOP_PAIR_BONUS));
        }
        res_str.push_str(&format!("\tSum: {}\n", self.eval_mg()));
        res_str
    }
}

impl EndGameDisplay for BishopEvaluation {
    fn display_eg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tBishops-EndGame\n");
        res_str.push_str(&format!(
            "\t\tAmount of Bishops: {} -> {}\n",
            self.amount_of_bishops,
            self.amount_of_bishops as f64 * BISHOP_PIECE_VALUE_EG
        ));
        if self.bishop_pair {
            res_str.push_str(&format!("\t\tBishop Pair:       {}\n", BISHOP_PAIR_BONUS));
        }
        res_str.push_str(&format!("\tSum: {}\n", self.eval_eg()));
        res_str
    }
}

pub fn bishop_eval(bishop: u64) -> BishopEvaluation {
    let bishops = bishop.count_ones();
    BishopEvaluation {
        amount_of_bishops: bishops,
        bishop_pair: bishops > 1u32,
    }
}
