use super::{EndGameDisplay, Evaluation, MidGameDisplay};

pub const ROOK_PIECE_VALUE_MG: i16 = 710;
pub const ROOK_PIECE_VALUE_EG: i16 = 920;

pub struct RookEvaluation {
    amount_of_rooks: i16,
}

impl Evaluation for RookEvaluation {
    fn eval_mg(&self) -> i16 {
        let mut res = 0;
        res += self.amount_of_rooks * ROOK_PIECE_VALUE_MG;
        res
    }
    fn eval_eg(&self) -> i16 {
        let mut res = 0;
        res += self.amount_of_rooks * ROOK_PIECE_VALUE_EG;
        res
    }
}

impl MidGameDisplay for RookEvaluation {
    fn display_mg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tRooks-MidGame\n");
        res_str.push_str(&format!(
            "\t\tAmount of Rooks: {} -> {}\n",
            self.amount_of_rooks,
            self.amount_of_rooks * ROOK_PIECE_VALUE_MG
        ));
        res_str.push_str(&format!("\tSum: {}\n", self.eval_mg()));
        res_str
    }
}

impl EndGameDisplay for RookEvaluation {
    fn display_eg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tRooks-EndGame\n");
        res_str.push_str(&format!(
            "\t\tAmount of Rooks: {} -> {}\n",
            self.amount_of_rooks,
            self.amount_of_rooks * ROOK_PIECE_VALUE_EG
        ));
        res_str.push_str(&format!("\tSum: {}\n", self.eval_eg()));
        res_str
    }
}

pub fn rook_eval(rook: u64) -> RookEvaluation {
    RookEvaluation {
        amount_of_rooks: rook.count_ones() as i16,
    }
}
