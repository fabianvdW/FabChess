use super::{bitboards, EndGameDisplay, Evaluation, MidGameDisplay, ParallelEvaluation};

pub const ROOK_ON_OPEN_FILE_BONUS_MG: f64 = 20.0;
pub const ROOK_ON_SEVENTH_MG: f64 = 10.0;
pub const ROOK_ON_OPEN_FILE_BONUS_EG: f64 = 20.0;
pub const ROOK_ON_SEVENTH_EG: f64 = 10.0;
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG: [f64; 5] = [30.0, 15.0, 0.0, -15.0, -30.0];
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG: [f64; 5] = [30.0, 15.0, 0.0, -15.0, -30.0];

pub struct PiecewiseEvaluation {
    my_pawns: u64,
    my_rooks: u64,
    my_bishops: u64,
    is_white: bool,
    all_pawns: u64,
}

impl Evaluation for PiecewiseEvaluation {
    fn eval_mg(&self) -> f64 {
        let mut res = 0.0;
        let mut rooks = self.my_rooks;
        while rooks != 0u64 {
            let idx = rooks.trailing_zeros() as usize;
            if self.is_white {
                if idx / 8 == 6 {
                    res += ROOK_ON_SEVENTH_MG;
                }
            } else {
                if idx / 8 == 1 {
                    res += ROOK_ON_SEVENTH_MG;
                }
            }
            if bitboards::FILES[idx % 8] & self.all_pawns == 0u64 {
                res += ROOK_ON_OPEN_FILE_BONUS_MG;
            }
            rooks ^= 1u64 << idx;
        }
        let mut bishops = self.my_bishops;
        while bishops != 0u64 {
            let idx = bishops.trailing_zeros() as usize;
            res += DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG
                [(bitboards::DIAGONALLY_ADJACENT[idx] & self.my_pawns).count_ones() as usize];
            bishops ^= 1u64 << idx;
        }
        res
    }
    fn eval_eg(&self) -> f64 {
        let mut res = 0.0;
        let mut rooks = self.my_rooks;
        while rooks != 0u64 {
            let idx = rooks.trailing_zeros() as usize;
            if self.is_white {
                if idx / 8 == 6 {
                    res += ROOK_ON_SEVENTH_EG
                }
            } else {
                if idx / 8 == 1 {
                    res += ROOK_ON_SEVENTH_EG;
                }
            }
            if bitboards::FILES[idx % 8] & self.all_pawns == 0u64 {
                res += ROOK_ON_OPEN_FILE_BONUS_EG;
            }
            rooks ^= 1u64 << idx;
        }
        let mut bishops = self.my_bishops;
        while bishops != 0u64 {
            let idx = bishops.trailing_zeros() as usize;
            res += DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG
                [(bitboards::DIAGONALLY_ADJACENT[idx] & self.my_pawns).count_ones() as usize];
            bishops ^= 1u64 << idx;
        }
        res
    }
}

impl ParallelEvaluation for PiecewiseEvaluation {
    fn eval_mg_eg(&self) -> (f64, f64) {
        let mut mg_res = 0.0;
        let mut eg_res = 0.0;
        let mut rooks = self.my_rooks;
        while rooks != 0u64 {
            let idx = rooks.trailing_zeros() as usize;
            if self.is_white {
                if idx / 8 == 6 {
                    mg_res += ROOK_ON_SEVENTH_MG;
                    eg_res += ROOK_ON_SEVENTH_EG;
                }
            } else {
                if idx / 8 == 1 {
                    mg_res += ROOK_ON_SEVENTH_MG;
                    eg_res += ROOK_ON_SEVENTH_EG;
                }
            }
            if bitboards::FILES[idx % 8] & self.all_pawns == 0u64 {
                mg_res += ROOK_ON_OPEN_FILE_BONUS_MG;
                eg_res += ROOK_ON_OPEN_FILE_BONUS_EG;
            }
            rooks ^= 1u64 << idx;
        }
        let mut bishops = self.my_bishops;
        while bishops != 0u64 {
            let idx = bishops.trailing_zeros() as usize;
            mg_res += DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG
                [(bitboards::DIAGONALLY_ADJACENT[idx] & self.my_pawns).count_ones() as usize];
            eg_res += DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG
                [(bitboards::DIAGONALLY_ADJACENT[idx] & self.my_pawns).count_ones() as usize];
            bishops ^= 1u64 << idx;
        }
        (mg_res, eg_res)
    }
}

impl MidGameDisplay for PiecewiseEvaluation {
    fn display_mg(&self) -> String {
        let mut rooks_on_open = 0usize;
        let mut rooks_on_seventh = 0usize;
        let mut rooks = self.my_rooks;
        while rooks != 0u64 {
            let idx = rooks.trailing_zeros() as usize;
            if self.is_white {
                if idx / 8 == 6 {
                    rooks_on_seventh += 1;
                }
            } else {
                if idx / 8 == 1 {
                    rooks_on_seventh += 1;
                }
            }
            if bitboards::FILES[idx % 8] & self.all_pawns == 0u64 {
                rooks_on_open += 1;
            }
            rooks ^= 1u64 << idx;
        }
        let mut bishop_adjacent_score = 0.0;
        let mut bishops = self.my_bishops;
        while bishops != 0u64 {
            let idx = bishops.trailing_zeros() as usize;
            bishop_adjacent_score += DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG
                [(bitboards::DIAGONALLY_ADJACENT[idx] & self.my_pawns).count_ones() as usize];
            bishops ^= 1u64 << idx;
        }

        let mut res_str = String::new();
        res_str.push_str("\tPiecewiseEvaluation-MidGame\n");
        res_str.push_str(&format!(
            "\t\tRooks on open file:             {} -> {}\n",
            rooks_on_open,
            rooks_on_open as f64 * ROOK_ON_OPEN_FILE_BONUS_MG
        ));
        res_str.push_str(&format!(
            "\t\tRooks on seventh rank:          {} -> {}\n",
            rooks_on_seventh,
            rooks_on_seventh as f64 * ROOK_ON_SEVENTH_MG
        ));
        res_str.push_str(&format!(
            "\t\tBishop diagonal adjacent pawns:      {}\n",
            bishop_adjacent_score
        ));
        res_str.push_str(&format!(
            "\tSum: {}\n",
            bishop_adjacent_score
                + rooks_on_open as f64 * ROOK_ON_OPEN_FILE_BONUS_MG
                + rooks_on_seventh as f64 * ROOK_ON_SEVENTH_MG
        ));
        res_str
    }
}

impl EndGameDisplay for PiecewiseEvaluation {
    fn display_eg(&self) -> String {
        let mut rooks_on_open = 0usize;
        let mut rooks_on_seventh = 0usize;
        let mut rooks = self.my_rooks;
        while rooks != 0u64 {
            let idx = rooks.trailing_zeros() as usize;
            if self.is_white {
                if idx / 8 == 6 {
                    rooks_on_seventh += 1;
                }
            } else {
                if idx / 8 == 1 {
                    rooks_on_seventh += 1;
                }
            }
            if bitboards::FILES[idx % 8] & self.all_pawns == 0u64 {
                rooks_on_open += 1;
            }
            rooks ^= 1u64 << idx;
        }
        let mut bishop_adjacent_score = 0.0;
        let mut bishops = self.my_bishops;
        while bishops != 0u64 {
            let idx = bishops.trailing_zeros() as usize;
            bishop_adjacent_score += DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG
                [(bitboards::DIAGONALLY_ADJACENT[idx] & self.my_pawns).count_ones() as usize];
            bishops ^= 1u64 << idx;
        }

        let mut res_str = String::new();
        res_str.push_str("\tPiecewiseEvaluation-EndGame\n");
        res_str.push_str(&format!(
            "\t\tRooks on open file:             {} -> {}\n",
            rooks_on_open,
            rooks_on_open as f64 * ROOK_ON_OPEN_FILE_BONUS_EG
        ));
        res_str.push_str(&format!(
            "\t\tRooks on seventh rank:          {} -> {}\n",
            rooks_on_seventh,
            rooks_on_seventh as f64 * ROOK_ON_SEVENTH_EG
        ));
        res_str.push_str(&format!(
            "\t\tBishop diagonal adjacent pawns:      {}\n",
            bishop_adjacent_score
        ));
        res_str.push_str(&format!(
            "\tSum: {}\n",
            bishop_adjacent_score
                + rooks_on_open as f64 * ROOK_ON_OPEN_FILE_BONUS_EG
                + rooks_on_seventh as f64 * ROOK_ON_SEVENTH_EG
        ));
        res_str
    }
}

pub fn piecewise_eval(
    my_pawns: u64,
    my_rooks: u64,
    my_bishops: u64,
    is_white: bool,
    all_pawns: u64,
) -> PiecewiseEvaluation {
    PiecewiseEvaluation {
        my_pawns,
        my_rooks,
        my_bishops,
        is_white,
        all_pawns,
    }
}
