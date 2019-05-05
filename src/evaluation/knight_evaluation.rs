use super::{bitboards, EndGameDisplay, Evaluation, MidGameDisplay};

pub const KNIGHT_PIECE_VALUE_MG: i16 = 500;
pub const KNIGHT_PIECE_VALUE_EG: i16 = 500;
pub const KNIGHT_VALUE_WITH_PAWNS: [i16; 17] = [
    -30, -27, -25, -22, -20, -17, -15, -12, -10, -7, -5, -2, 0, 2, 5, 7, 10,
];
pub const KNIGHT_SUPPORTED_BY_PAWN_MG: i16 = 10;
pub const KNIGHT_SUPPORTED_BY_PAWN_EG: i16 = 5;
pub const KNIGHT_OUTPOST_MG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 7, 8, 10, 10, 8, 7, 5],
    [5, 10, 10, 15, 15, 10, 10, 5],
    [10, 15, 15, 20, 20, 15, 15, 10],
    [15, 20, 20, 25, 25, 20, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const KNIGHT_OUTPOST_EG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 7, 8, 10, 10, 8, 7, 5],
    [5, 10, 10, 15, 15, 10, 10, 5],
    [10, 15, 15, 20, 20, 15, 15, 10],
    [15, 20, 20, 25, 25, 20, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub struct KnightEvaluation {
    amount_of_knights: i16,
    pawns_on_board: usize,
    supported_knights: i16,
    outpost_knights: u64,
    is_white: bool,
}

impl Evaluation for KnightEvaluation {
    fn eval_mg(&self) -> i16 {
        let mut res = 0;
        res += self.amount_of_knights
            * (KNIGHT_PIECE_VALUE_MG + KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board]);
        res += self.supported_knights * KNIGHT_SUPPORTED_BY_PAWN_MG;
        let mut outposts = self.outpost_knights;
        while outposts != 0u64 {
            let mut index = outposts.trailing_zeros() as usize;
            outposts ^= 1 << index;
            if !self.is_white {
                index = 63 - index;
            }
            res += KNIGHT_OUTPOST_MG_TABLE[index / 8][index % 8];
        }
        res
    }
    fn eval_eg(&self) -> i16 {
        let mut res = 0;
        res += self.amount_of_knights
            * (KNIGHT_PIECE_VALUE_EG + KNIGHT_VALUE_WITH_PAWNS[self.pawns_on_board]);
        res += self.supported_knights * KNIGHT_SUPPORTED_BY_PAWN_EG;
        let mut outposts = self.outpost_knights;
        while outposts != 0u64 {
            let mut index = outposts.trailing_zeros() as usize;
            outposts ^= 1 << index;
            if !self.is_white {
                index = 63 - index;
            }
            res += KNIGHT_OUTPOST_EG_TABLE[index / 8][index % 8];
        }
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
            self.supported_knights * KNIGHT_SUPPORTED_BY_PAWN_EG
        ));
        let mut res = 0;
        let mut outposts = self.outpost_knights;
        while outposts != 0u64 {
            let mut index = outposts.trailing_zeros() as usize;
            outposts ^= 1 << index;
            if !self.is_white {
                index = 63 - index;
            }
            res += KNIGHT_OUTPOST_MG_TABLE[index / 8][index % 8];
        }
        res_str.push_str(&format!(
            "\t\tOutpost Knights: {} -> {}\n",
            self.outpost_knights.count_ones(),
            res
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
            self.supported_knights * KNIGHT_SUPPORTED_BY_PAWN_EG
        ));
        let mut res = 0;
        let mut outposts = self.outpost_knights;
        while outposts != 0u64 {
            let mut index = outposts.trailing_zeros() as usize;
            outposts ^= 1 << index;
            if !self.is_white {
                index = 63 - index;
            }
            res += KNIGHT_OUTPOST_EG_TABLE[index / 8][index % 8];
        }
        res_str.push_str(&format!(
            "\t\tOutpost Knights: {} -> {}\n",
            self.outpost_knights.count_ones(),
            res
        ));
        res_str.push_str(&format!("\tSum: {}\n", self.eval_eg()));
        res_str
    }
}

pub fn knight_eval(
    knight: u64,
    my_pawns_attacks: u64,
    pawns_on_board: usize,
    enemy_pawns: u64,
    is_white: bool,
) -> KnightEvaluation {
    let supported_knights = my_pawns_attacks & knight;
    //Roughly determine outpost
    let mut outposts = 0u64;
    let mut supp = supported_knights;
    while supp != 0u64 {
        let supp_knight_index = supp.trailing_zeros() as usize;
        let outpost = 1u64 << supp_knight_index;
        let mut front_span = if is_white {
            bitboards::w_front_span(outpost)
        } else {
            bitboards::b_front_span(outpost)
        };
        front_span = bitboards::west_one(front_span) | bitboards::east_one(front_span);
        let file_int = supp_knight_index % 8;
        let file = bitboards::FILES[file_int];
        if enemy_pawns & front_span != 0u64 {
            outposts |= outpost;
        }
        supp &= !file;
    }
    KnightEvaluation {
        amount_of_knights: knight.count_ones() as i16,
        pawns_on_board,
        supported_knights: supported_knights.count_ones() as i16,
        outpost_knights: outposts,
        is_white,
    }
}
