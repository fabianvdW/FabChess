use super::PieceType;
use super::{EndGameDisplay, Evaluation, MidGameDisplay, ParallelEvaluation};

const PSQT_PAWN_MG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-5, -5, -5, -5, -5, -5, -5, -5],
    [-7, 3, 6, 10, 10, 6, 3, -7],
    [-14, -7, 15, 20, 20, 15, -7, -14],
    [-10, -2, 1, 12, 12, 1, -2, -10],
    [-7, -1, 0, 5, 5, 0, -1, -7],
    [-3, 10, 5, 5, 5, 5, 10, -3],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
const PSQT_PAWN_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-20, -20, -20, -20, -20, -20, -20, -20],
    [-10, -10, -10, -10, -10, -10, -10, -10],
    [-5, -5, -5, -5, -5, -5, -5, -5],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 5, 5, 5, 5, 5, 5, 5],
    [10, 10, 10, 10, 10, 10, 10, 10],
    [0, 0, 0, 0, 0, 0, 0, 0],
];

pub const PSQT_KNIGHT_MG: [[i16; 8]; 8] = [
    [-40, -20, -30, -30, -30, -30, -20, -40],
    [-40, -20, 0, 5, 5, 0, -20, -40],
    [-30, 0, 10, 10, 10, 10, 0, -30],
    [-30, 5, 20, 15, 15, 20, 5, -30],
    [-30, 30, 40, 40, 40, 20, 30, -30],
    [-30, 15, 25, 35, 35, 25, 15, -30],
    [-40, -20, 0, 5, 5, 0, -20, -40],
    [-50, -40, -30, -30, -30, -30, -40, -50],
];
pub const PSQT_KNIGHT_EG: [[i16; 8]; 8] = [
    [-50, -40, -30, -30, -30, -30, -40, -50],
    [-40, -25, -10, 0, 0, -10, -25, -40],
    [-30, -10, 5, 10, 10, 5, -10, -30],
    [-30, 0, 10, 20, 20, 10, 0, -30],
    [-30, 0, 10, 20, 20, 10, 0, -30],
    [-30, -10, 5, 10, 10, 5, -10, -30],
    [-40, -25, -10, 0, 0, -10, -25, -40],
    [-50, -40, -30, -30, -30, -30, -40, -50],
];
pub const PSQT_BISHOP_MG: [[i16; 8]; 8] = [
    [-50, -10, -10, -30, -30, -10, -10, -50],
    [-30, 10, 15, 0, 0, 15, 10, -30],
    [-10, 10, 15, 10, 10, 15, 10, -10],
    [-10, 15, 20, 25, 25, 20, 0, -10],
    [-10, 10, 20, 25, 25, 20, 0, -10],
    [-10, 10, 15, 10, 10, 15, 10, -10],
    [-30, 10, 15, 0, 0, 15, 10, -30],
    [-50, -10, -10, -30, -30, -10, -10, -50],
];

pub const PSQT_BISHOP_EG: [[i16; 8]; 8] = [
    [-50, -30, -30, -20, -20, -30, -30, -50],
    [-30, -10, -10, 5, 5, -10, -10, -30],
    [-20, 0, 0, 12, 12, 0, 0, -20],
    [-20, 0, 0, 12, 12, 0, 0, -20],
    [-20, 0, 0, 12, 12, 0, 0, -20],
    [-20, 0, 0, 12, 12, 0, 0, -20],
    [-30, -10, -10, 5, 5, -10, -10, -30],
    [-50, -30, -30, -20, -20, -30, -30, -50],
];

const PSQT_KING_MG: [[i16; 8]; 8] = [
    [40, 60, 20, 0, 0, 20, 60, 40],
    [40, 40, 0, 0, 0, 0, 20, 20],
    [-20, -40, -40, -40, -40, -40, -40, -20],
    [-40, -60, -60, -80, -80, -60, -60, -40],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
];
const PSQT_KING_EG: [[i16; 8]; 8] = [
    [-50, -30, -30, -30, -30, -30, -30, -50],
    [-30, -30, 0, 0, 0, 0, -30, -30],
    [-30, -10, 20, 30, 30, 20, -10, -30],
    [-30, -10, 30, 40, 40, 30, -10, -30],
    [-30, -10, 30, 40, 40, 30, -10, -30],
    [-30, -10, 20, 30, 30, 20, -10, -30],
    [-30, -20, -10, 0, 0, -10, -20, -30],
    [-50, -40, -30, -20, -20, -30, -40, -50],
];

pub struct PSQT {
    pawns: u64,
    knights: u64,
    bishops: u64,
    rooks: u64,
    queens: u64,
    king: u64,
    is_white: bool,
}

impl PSQT {
    pub fn new(
        pawns: u64,
        knights: u64,
        bishops: u64,
        rooks: u64,
        queens: u64,
        king: u64,
        is_white: bool,
    ) -> PSQT {
        PSQT {
            pawns,
            knights,
            bishops,
            rooks,
            queens,
            king,
            is_white,
        }
    }
    pub fn copy(&self) -> PSQT {
        PSQT::new(
            self.pawns,
            self.knights,
            self.bishops,
            self.rooks,
            self.queens,
            self.king,
            self.is_white,
        )
    }
}

impl Evaluation for PSQT {
    fn eval_mg(&self) -> i16 {
        let mut res = 0;
        let mut copy = self.copy();
        while copy.pawns != 0u64 {
            let mut idx = copy.pawns.trailing_zeros() as usize;
            copy.pawns ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            res += PSQT_PAWN_MG[idx / 8][idx % 8];
        }
        while copy.knights != 0u64 {
            let mut idx = copy.knights.trailing_zeros() as usize;
            copy.knights ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            res += PSQT_KNIGHT_MG[idx / 8][idx % 8];
        }
        while copy.bishops != 0u64 {
            let mut idx = copy.bishops.trailing_zeros() as usize;
            copy.bishops ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            res += PSQT_BISHOP_MG[idx / 8][idx % 8];
        }
        let mut king_position = copy.king.trailing_zeros() as usize;
        if !self.is_white {
            king_position = 63 - king_position;
        }
        res += PSQT_KING_MG[king_position / 8][king_position % 8];
        res
    }
    fn eval_eg(&self) -> i16 {
        let mut res = 0;
        let mut copy = self.copy();
        while copy.pawns != 0u64 {
            let mut idx = copy.pawns.trailing_zeros() as usize;
            copy.pawns ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            res += PSQT_PAWN_EG[idx / 8][idx % 8];
        }
        while copy.knights != 0u64 {
            let mut idx = copy.knights.trailing_zeros() as usize;
            copy.knights ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            res += PSQT_KNIGHT_EG[idx / 8][idx % 8];
        }
        while copy.bishops != 0u64 {
            let mut idx = copy.bishops.trailing_zeros() as usize;
            copy.bishops ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            res += PSQT_BISHOP_EG[idx / 8][idx % 8];
        }
        let mut king_position = copy.king.trailing_zeros() as usize;
        if !self.is_white {
            king_position = 63 - king_position;
        }
        res += PSQT_KING_EG[king_position / 8][king_position % 8];
        res
    }
}

impl ParallelEvaluation for PSQT {
    fn eval_mg_eg(&self) -> (i16, i16) {
        let (mut mg_res, mut eg_res) = (0, 0);
        let mut copy = self.copy();
        while copy.pawns != 0u64 {
            let mut idx = copy.pawns.trailing_zeros() as usize;
            copy.pawns ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            mg_res += PSQT_PAWN_MG[idx / 8][idx % 8];
            eg_res += PSQT_PAWN_EG[idx / 8][idx % 8];
        }
        while copy.knights != 0u64 {
            let mut idx = copy.knights.trailing_zeros() as usize;
            copy.knights ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            mg_res += PSQT_KNIGHT_MG[idx / 8][idx % 8];
            eg_res += PSQT_KNIGHT_EG[idx / 8][idx % 8];
        }
        while copy.bishops != 0u64 {
            let mut idx = copy.bishops.trailing_zeros() as usize;
            copy.bishops ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            mg_res += PSQT_BISHOP_MG[idx / 8][idx % 8];
            eg_res += PSQT_BISHOP_EG[idx / 8][idx % 8];
        }
        let mut king_position = copy.king.trailing_zeros() as usize;
        if !self.is_white {
            king_position = 63 - king_position;
        }
        mg_res += PSQT_KING_MG[king_position / 8][king_position % 8];
        eg_res += PSQT_KING_EG[king_position / 8][king_position % 8];
        (mg_res, eg_res)
    }
}

impl MidGameDisplay for PSQT {
    fn display_mg(&self) -> String {
        let mut copy = self.copy();
        let mut pawn_score = 0;
        while copy.pawns != 0u64 {
            let mut idx = copy.pawns.trailing_zeros() as usize;
            copy.pawns ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            pawn_score += PSQT_PAWN_MG[idx / 8][idx % 8];
        }
        let mut knight_score = 0;
        while copy.knights != 0u64 {
            let mut idx = copy.knights.trailing_zeros() as usize;
            copy.knights ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            knight_score += PSQT_KNIGHT_MG[idx / 8][idx % 8];
        }
        let mut bishop_score = 0;
        while copy.bishops != 0u64 {
            let mut idx = copy.bishops.trailing_zeros() as usize;
            copy.bishops ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            bishop_score += PSQT_BISHOP_MG[idx / 8][idx % 8];
        }
        let mut king_position = copy.king.trailing_zeros() as usize;
        if !self.is_white {
            king_position = 63 - king_position;
        }
        let king_score = PSQT_KING_MG[king_position / 8][king_position % 8];

        let mut res_str = String::new();
        res_str.push_str("\tPSQT-MidGame\n");
        res_str.push_str(&format!(
            "\t\tPawns:    {} -> {}\n",
            self.pawns.count_ones(),
            pawn_score
        ));
        res_str.push_str(&format!(
            "\t\tKnights:  {} -> {}\n",
            self.knights.count_ones(),
            knight_score
        ));
        res_str.push_str(&format!(
            "\t\tBishops:  {} -> {}\n",
            self.bishops.count_ones(),
            bishop_score
        ));
        res_str.push_str(&format!("\t\tKing:          {}\n", king_score));
        res_str.push_str(&format!(
            "\tSum: {}\n",
            pawn_score + knight_score + bishop_score + king_score
        ));
        res_str
    }
}

impl EndGameDisplay for PSQT {
    fn display_eg(&self) -> String {
        let mut copy = self.copy();
        let mut pawn_score = 0;
        while copy.pawns != 0u64 {
            let mut idx = copy.pawns.trailing_zeros() as usize;
            copy.pawns ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            pawn_score += PSQT_PAWN_EG[idx / 8][idx % 8];
        }
        let mut knight_score = 0;
        while copy.knights != 0u64 {
            let mut idx = copy.knights.trailing_zeros() as usize;
            copy.knights ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            knight_score += PSQT_KNIGHT_EG[idx / 8][idx % 8];
        }
        let mut bishop_score = 0;
        while copy.bishops != 0u64 {
            let mut idx = copy.bishops.trailing_zeros() as usize;
            copy.bishops ^= 1u64 << idx;
            if !self.is_white {
                idx = 63 - idx;
            }
            bishop_score += PSQT_BISHOP_EG[idx / 8][idx % 8];
        }
        let mut king_position = copy.king.trailing_zeros() as usize;
        if !self.is_white {
            king_position = 63 - king_position;
        }
        let king_score = PSQT_KING_EG[king_position / 8][king_position % 8];

        let mut res_str = String::new();
        res_str.push_str("\tPSQT-EndGame\n");
        res_str.push_str(&format!(
            "\t\tPawns:    {} -> {}\n",
            self.pawns.count_ones(),
            pawn_score
        ));
        res_str.push_str(&format!(
            "\t\tKnights:  {} -> {}\n",
            self.knights.count_ones(),
            knight_score
        ));
        res_str.push_str(&format!(
            "\t\tBishops:  {} -> {}\n",
            self.bishops.count_ones(),
            bishop_score
        ));
        res_str.push_str(&format!("\t\tKing:          {}\n", king_score));
        res_str.push_str(&format!(
            "\tSum: {}\n",
            pawn_score + knight_score + bishop_score + king_score
        ));
        res_str
    }
}

pub fn psqt_eval(
    pawns: u64,
    knights: u64,
    bishops: u64,
    rooks: u64,
    queens: u64,
    king: u64,
    is_white: bool,
) -> PSQT {
    PSQT::new(pawns, knights, bishops, rooks, queens, king, is_white)
}

#[inline(always)]
pub fn psqt_incremental_move_piece(
    piece: &PieceType,
    mut from_square: usize,
    mut to_square: usize,
    is_black: bool,
    psqt_mg: i16,
    psqt_eg: i16,
) -> (i16, i16) {
    if is_black {
        from_square = 63 - from_square;
        to_square = 63 - to_square;
    }
    let mut psqt_mg_plus: i16 = 0;
    let mut psqt_eg_plus: i16 = 0;
    if let PieceType::Pawn = piece {
        psqt_mg_plus += PSQT_PAWN_MG[to_square / 8][to_square % 8]
            - PSQT_PAWN_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_PAWN_EG[to_square / 8][to_square % 8]
            - PSQT_PAWN_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Knight = piece {
        psqt_mg_plus += PSQT_KNIGHT_MG[to_square / 8][to_square % 8]
            - PSQT_KNIGHT_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_KNIGHT_EG[to_square / 8][to_square % 8]
            - PSQT_KNIGHT_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Bishop = piece {
        psqt_mg_plus += PSQT_BISHOP_MG[to_square / 8][to_square % 8]
            - PSQT_BISHOP_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_BISHOP_EG[to_square / 8][to_square % 8]
            - PSQT_BISHOP_EG[from_square / 8][from_square % 8];
    } else if let PieceType::King = piece {
        psqt_mg_plus += PSQT_KING_MG[to_square / 8][to_square % 8]
            - PSQT_KING_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_KING_EG[to_square / 8][to_square % 8]
            - PSQT_KING_EG[from_square / 8][from_square % 8];
    }
    if is_black {
        psqt_mg_plus *= -1;
        psqt_eg_plus *= -1;
    }
    (psqt_mg + psqt_mg_plus, psqt_eg + psqt_eg_plus)
}

#[inline(always)]
pub fn psqt_incremental_delete_piece(
    piece: &PieceType,
    mut from_square: usize,
    is_black: bool,
    psqt_mg: i16,
    psqt_eg: i16,
) -> (i16, i16) {
    if is_black {
        from_square = 63 - from_square;
    }
    let mut psqt_mg_plus = 0;
    let mut psqt_eg_plus = 0;
    if let PieceType::Pawn = piece {
        psqt_mg_plus += -PSQT_PAWN_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += -PSQT_PAWN_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Knight = piece {
        psqt_mg_plus += -PSQT_KNIGHT_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += -PSQT_KNIGHT_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Bishop = piece {
        psqt_mg_plus += -PSQT_BISHOP_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += -PSQT_BISHOP_EG[from_square / 8][from_square % 8];
    } else if let PieceType::King = piece {
        psqt_mg_plus += -PSQT_KING_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += -PSQT_KING_EG[from_square / 8][from_square % 8];
    }
    if is_black {
        psqt_mg_plus *= -1;
        psqt_eg_plus *= -1;
    }
    (psqt_mg + psqt_mg_plus, psqt_eg + psqt_eg_plus)
}

#[inline(always)]
pub fn psqt_incremental_add_piece(
    piece: &PieceType,
    mut from_square: usize,
    is_black: bool,
    psqt_mg: i16,
    psqt_eg: i16,
) -> (i16, i16) {
    if is_black {
        from_square = 63 - from_square;
    }
    let mut psqt_mg_plus = 0;
    let mut psqt_eg_plus = 0;
    if let PieceType::Pawn = piece {
        psqt_mg_plus += PSQT_PAWN_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_PAWN_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Knight = piece {
        psqt_mg_plus += PSQT_KNIGHT_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_KNIGHT_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Bishop = piece {
        psqt_mg_plus += PSQT_BISHOP_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_BISHOP_EG[from_square / 8][from_square % 8];
    } else if let PieceType::King = piece {
        psqt_mg_plus += PSQT_KING_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_KING_EG[from_square / 8][from_square % 8];
    }
    if is_black {
        psqt_mg_plus *= -1;
        psqt_eg_plus *= -1;
    }
    (psqt_mg + psqt_mg_plus, psqt_eg + psqt_eg_plus)
}

pub fn psqt_slow(pieces: &[[u64; 2]; 6]) -> (i16, i16) {
    let pieces = pieces.clone();
    let mut mg_res = 0;
    let mut eg_res = 0;
    let pawns_white = eval_pawns(pieces[0][0], false);
    let pawns_black = eval_pawns(pieces[0][1], true);
    let knights_white = eval_knights(pieces[1][0], false);
    let knights_black = eval_knights(pieces[1][1], true);
    let bishops_white = eval_bishops(pieces[2][0], false);
    let bishops_black = eval_bishops(pieces[2][1], true);
    let king_white = eval_king(pieces[5][0], false);
    let king_black = eval_king(pieces[5][1], true);
    mg_res += pawns_white.0 - pawns_black.0 + knights_white.0 - knights_black.0 + bishops_white.0
        - bishops_black.0
        + king_white.0
        - king_black.0;
    eg_res += pawns_white.1 - pawns_black.1 + knights_white.1 - knights_black.1 + bishops_white.1
        - bishops_black.1
        + king_white.1
        - king_black.1;
    (mg_res, eg_res)
}

#[inline(always)]
pub fn eval_pawns(mut pawns: u64, is_black: bool) -> (i16, i16) {
    let mut mg_res = 0;
    let mut eg_res = 0;
    while pawns != 0u64 {
        let mut idx = pawns.trailing_zeros() as usize;
        pawns ^= 1 << idx;
        if is_black {
            idx = 63 - idx;
        }
        mg_res += PSQT_PAWN_MG[idx / 8][idx % 8];
        eg_res += PSQT_PAWN_EG[idx / 8][idx % 8];
    }
    (mg_res, eg_res)
}

#[inline(always)]
pub fn eval_knights(mut knights: u64, is_black: bool) -> (i16, i16) {
    let mut mg_res = 0;
    let mut eg_res = 0;
    while knights != 0u64 {
        let mut idx = knights.trailing_zeros() as usize;
        knights ^= 1 << idx;
        if is_black {
            idx = 63 - idx;
        }
        mg_res += PSQT_KNIGHT_MG[idx / 8][idx % 8];
        eg_res += PSQT_KNIGHT_EG[idx / 8][idx % 8];
    }
    (mg_res, eg_res)
}

#[inline(always)]
pub fn eval_bishops(mut bishops: u64, is_black: bool) -> (i16, i16) {
    let mut mg_res = 0;
    let mut eg_res = 0;
    while bishops != 0u64 {
        let mut idx = bishops.trailing_zeros() as usize;
        bishops ^= 1 << idx;
        if is_black {
            idx = 63 - idx;
        }
        mg_res += PSQT_BISHOP_MG[idx / 8][idx % 8];
        eg_res += PSQT_BISHOP_EG[idx / 8][idx % 8];
    }
    (mg_res, eg_res)
}

#[inline(always)]
pub fn eval_king(king: u64, is_black: bool) -> (i16, i16) {
    let mut mg_res = 0;
    let mut eg_res = 0;
    let mut idx = king.trailing_zeros() as usize;
    if is_black {
        idx = 63 - idx;
    }
    mg_res += PSQT_KING_MG[idx / 8][idx % 8];
    eg_res += PSQT_KING_EG[idx / 8][idx % 8];
    (mg_res, eg_res)
}
