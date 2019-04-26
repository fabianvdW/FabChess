use super::{EndGameDisplay, Evaluation, MidGameDisplay};

pub const KNIGHT_PIECE_VALUE_MG: i16 = 500;
pub const KNIGHT_PIECE_VALUE_EG: i16 = 500;
pub const KNIGHT_VALUE_WITH_PAWNS: [i16; 17] = [
    -30, -27, -25, -22, -20, -17, -15, -12, -10, -7, -5, -2, 0, 2, 5, 7, 10,
];
pub const KNIGHT_SUPPORTED_BY_PAWN: i16 = 30;

pub struct KnightEvaluation {
    amount_of_knights: i16,
    pawns_on_board: usize,
    supported_knights: i16,
}

impl Evaluation for KnightEvaluation {
    fn eval_mg(&self) -> i16 {
        let mut res = 0;
        res += self.amount_of_knights
            * (KNIGHT_PIECE_VALUE_MG + KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board]);
        res += self.supported_knights * KNIGHT_SUPPORTED_BY_PAWN;
        res
    }
    fn eval_eg(&self) -> i16 {
        let mut res = 0;
        res += self.amount_of_knights
            * (KNIGHT_PIECE_VALUE_EG + KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board]);
        res += self.supported_knights * KNIGHT_SUPPORTED_BY_PAWN;
        res
    }
}

impl MidGameDisplay for KnightEvaluation {
    fn display_mg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tKnights-MidGame\n");
        res_str.push_str(&format!(
            "\t\tAmount of Knights: {} * ({} + {}) -> {}\n",
            self.amount_of_knights,
            KNIGHT_PIECE_VALUE_MG,
            KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board],
            self.amount_of_knights
                * (KNIGHT_PIECE_VALUE_MG + KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board])
        ));
        res_str.push_str(&format!(
            "\t\tSupported Knights: {} -> {}\n",
            self.supported_knights,
            self.supported_knights * KNIGHT_SUPPORTED_BY_PAWN
        ));
        res_str.push_str(&format!("\tSum: {}\n", self.eval_mg()));
        res_str
    }
}

impl EndGameDisplay for KnightEvaluation {
    fn display_eg(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tKnights-EndGame\n");
        res_str.push_str(&format!(
            "\t\tAmount of Knights: {} * ({} + {}) -> {}\n",
            self.amount_of_knights,
            KNIGHT_PIECE_VALUE_EG,
            KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board],
            self.amount_of_knights
                * (KNIGHT_PIECE_VALUE_EG + KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board])
        ));
        res_str.push_str(&format!(
            "\t\tSupported Knights: {} -> {}\n",
            self.supported_knights,
            self.supported_knights * KNIGHT_SUPPORTED_BY_PAWN
        ));
        res_str.push_str(&format!("\tSum: {}\n", self.eval_eg()));
        res_str
    }
}

pub fn knight_eval(knight: u64, my_pawns_attacks: u64, pawns_on_board: usize) -> KnightEvaluation {
    let supported_knights = (my_pawns_attacks & knight).count_ones() as i16;
    KnightEvaluation {
        amount_of_knights: knight.count_ones() as i16,
        pawns_on_board,
        supported_knights,
    }
}
