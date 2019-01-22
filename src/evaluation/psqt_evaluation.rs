use super::{Evaluation, ParallelEvaluation};

const PSQT_PAWN_MG: [[f64; 8]; 8] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [-5.0, -5.0, -5.0, -5.0, -5.0, -5.0, -5.0, -5.0],
    [-7.0, 3.0, 6.0, 10.0, 10.0, 6.0, 3.0, -7.0],
    [-14.0, -7.0, 15.0, 20.0, 20.0, 15.0, -7.0, -14.0],
    [-10.0, -2.0, 1.0, 12.0, 12.0, 1.0, -2.0, -10.0],
    [-7.0, -1.0, 0.0, 5.0, 5.0, 0.0, -1.0, -7.0],
    [-3.0, 10.0, 5.0, 5.0, 5.0, 5.0, 10.0, -3.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
];
const PSQT_PAWN_EG: [[f64; 8]; 8] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [-20.0, -20.0, -20.0, -20.0, -20.0, -20.0, -20.0, -20.0],
    [-10.0, -10.0, -10.0, -10.0, -10.0, -10.0, -10.0, -10.0],
    [-5.0, -5.0, -5.0, -5.0, -5.0, -5.0, -5.0, -5.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
    [10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
];

pub const PSQT_KNIGHT_MG: [[f64; 8]; 8] = [
    [-50.0, -40.0, -30.0, -30.0, -30.0, -30.0, -40.0, -50.0],
    [-40.0, -20.0, 0.0, 5.0, 5.0, 0.0, -20.0, -40.0],
    [-30.0, 0.0, 10.0, 20.0, 20.0, 10.0, 0.0, -30.0],
    [-30.0, 5.0, 20.0, 40.0, 40.0, 20.0, 5.0, -30.0],
    [-30.0, 5.0, 20.0, 40.0, 40.0, 20.0, 5.0, -30.0],
    [-30.0, 0.0, 10.0, 20.0, 20.0, 10.0, 0.0, -30.0],
    [-40.0, -20.0, 0.0, 5.0, 5.0, 0.0, -20.0, -40.0],
    [-50.0, -40.0, -30.0, -30.0, -30.0, -30.0, -40.0, -50.0],
];
pub const PSQT_KNIGHT_EG: [[f64; 8]; 8] = [
    [-50.0, -40.0, -30.0, -30.0, -30.0, -30.0, -40.0, -50.0],
    [-40.0, -25.0, -10.0, 0.0, 0.0, -10.0, -25.0, -40.0],
    [-30.0, -10.0, 5.0, 10.0, 10.0, 5.0, -10.0, -30.0],
    [-30.0, 0.0, 10.0, 20.0, 20.0, 10.0, 0.0, -30.0],
    [-30.0, 0.0, 10.0, 20.0, 20.0, 10.0, 0.0, -30.0],
    [-30.0, -10.0, 5.0, 10.0, 10.0, 5.0, -10.0, -30.0],
    [-40.0, -25.0, -10.0, 0.0, 0.0, -10.0, -25.0, -40.0],
    [-50.0, -40.0, -30.0, -30.0, -30.0, -30.0, -40.0, -50.0],
];
pub const PSQT_BISHOP_MG: [[f64; 8]; 8] = [
    [-50.0, -10.0, -10.0, -30.0, -30.0, -10.0, -10.0, -50.0],
    [-30.0, 10.0, 15.0, 0.0, 0.0, 15.0, 10.0, -30.0],
    [-10.0, 10.0, 15.0, 10.0, 10.0, 15.0, 10.0, -10.0],
    [-10.0, 15.0, 20.0, 25.0, 25.0, 20.0, 0.0, -10.0],
    [-10.0, 10.0, 20.0, 25.0, 25.0, 20.0, 0.0, -10.0],
    [-10.0, 10.0, 15.0, 10.0, 10.0, 15.0, 10.0, -10.0],
    [-30.0, 10.0, 15.0, 0.0, 0.0, 15.0, 10.0, -30.0],
    [-50.0, -10.0, -10.0, -30.0, -30.0, -10.0, -10.0, -50.0],
];

pub const PSQT_BISHOP_EG: [[f64; 8]; 8] = [
    [-50.0, -30.0, -30.0, -20.0, -20.0, -30.0, -30.0, -50.0],
    [-30.0, -10.0, -10.0, 5.0, 5.0, -10.0, -10.0, -30.0],
    [-20.0, 0.0, 0.0, 12.0, 12.0, 0.0, 0.0, -20.0],
    [-20.0, 0.0, 0.0, 12.0, 12.0, 0.0, 0.0, -20.0],
    [-20.0, 0.0, 0.0, 12.0, 12.0, 0.0, 0.0, -20.0],
    [-20.0, 0.0, 0.0, 12.0, 12.0, 0.0, 0.0, -20.0],
    [-30.0, -10.0, -10.0, 5.0, 5.0, -10.0, -10.0, -30.0],
    [-50.0, -30.0, -30.0, -20.0, -20.0, -30.0, -30.0, -50.0],
];

const PSQT_KING_MG: [[f64; 8]; 8] = [
    [40.0, 60.0, 20.0, 0.0, 0.0, 20.0, 60.0, 40.0],
    [40.0, 40.0, 0.0, 0.0, 0.0, 0.0, 20.0, 20.0],
    [-20.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -20.0],
    [-40.0, -60.0, -60.0, -80.0, -80.0, -60.0, -60.0, -40.0],
    [-60.0, -80.0, -80.0, -100.0, -100.0, -80.0, -80.0, -60.0],
    [-60.0, -80.0, -80.0, -100.0, -100.0, -80.0, -80.0, -60.0],
    [-60.0, -80.0, -80.0, -100.0, -100.0, -80.0, -80.0, -60.0],
    [-60.0, -80.0, -80.0, -100.0, -100.0, -80.0, -80.0, -60.0],
];
const PSQT_KING_EG: [[f64; 8]; 8] = [
    [-100.0, -60.0, -60.0, -60.0, -60.0, -60.0, -60.0, -100.0],
    [-60.0, -60.0, 0.0, 0.0, 0.0, 0.0, -60.0, -60.0],
    [-60.0, -20.0, 40.0, 60.0, 60.0, 40.0, -20.0, -60.0],
    [-60.0, -20.0, 60.0, 80.0, 80.0, 60.0, -20.0, -60.0],
    [-60.0, -20.0, 60.0, 80.0, 80.0, 60.0, -20.0, -60.0],
    [-60.0, -20.0, 40.0, 60.0, 60.0, 40.0, -20.0, -60.0],
    [-60.0, -40.0, -20.0, 0.0, 0.0, -20.0, -40.0, -60.0],
    [-100.0, -80.0, -60.0, -40.0, -40.0, -60.0, -80.0, -100.0]
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
    pub fn new(pawns: u64, knights: u64, bishops: u64, rooks: u64, queens: u64, king: u64, is_white: bool) -> PSQT {
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
        PSQT::new(self.pawns, self.knights, self.bishops, self.rooks, self.queens, self.king, self.is_white)
    }
}

impl Evaluation for PSQT {
    fn eval_mg(&self) -> f64 {
        let mut res = 0.0;
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
    fn eval_eg(&self) -> f64 {
        let mut res = 0.0;
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
    fn eval_mg_eg(&self) -> (f64, f64) {
        let (mut mg_res, mut eg_res) = (0.0, 0.0);
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

pub fn psqt_eval(pawns: u64, knights: u64, bishops: u64, rooks: u64, queens: u64, king: u64, is_white: bool) -> PSQT {
    PSQT::new(pawns, knights, bishops, rooks, queens, king, is_white)
}