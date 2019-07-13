use super::{bitboards, EndGameDisplay, Evaluation, MidGameDisplay, ParallelEvaluation};
use crate::move_generation::movegen::{bishop_attack, knight_attack, rook_attack};
pub const ROOK_ON_OPEN_FILE_BONUS_MG: i16 = 20;
pub const ROOK_ON_SEVENTH_MG: i16 = 10;
pub const ROOK_ON_OPEN_FILE_BONUS_EG: i16 = 20;
pub const ROOK_ON_SEVENTH_EG: i16 = 10;
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG: [i16; 5] = [0, 0, 0, -40, -100];
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG: [i16; 5] = [0, 0, 0, -40, -100];

pub const KNIGHT_MOBILITY_BONUS_MG: [i16; 9] = [-75, -50, -15, -5, 5, 15, 25, 35, 45];
pub const KNIGHT_MOBILITY_BONUS_EG: [i16; 9] = [-75, -50, -15, -5, 3, 7, 12, 15, 20];
pub const BISHOP_MOBILITY_BONUS_MG: [i16; 14] =
    [-50, -25, 0, 15, 25, 35, 45, 55, 65, 70, 75, 80, 85, 90];
pub const BISHOP_MOBILITY_BONUS_EG: [i16; 14] =
    [-50, -25, 0, 8, 12, 17, 23, 27, 35, 40, 45, 50, 55, 60];
pub const ROOK_MOBILITY_BONUS_MG: [i16; 15] =
    [-30, -10, -5, 0, 2, 5, 7, 10, 13, 15, 17, 20, 25, 30, 35];
pub const ROOK_MOBILITY_BONUS_EG: [i16; 15] = [
    -80, -60, -40, -30, -25, 0, 25, 35, 45, 50, 55, 60, 65, 70, 75,
];
pub const QUEEN_MOBILITY_BONUS_MG: [i16; 28] = [
    -40, -30, -20, -10, -5, 0, 3, 5, 8, 10, 13, 15, 18, 20, 23, 25, 28, 30, 33, 35, 38, 40, 43, 45,
    48, 50, 53, 55,
];

pub const QUEEN_MOBILITY_BONUS_EG: [i16; 28] = [
    -40, -30, -20, -10, -5, 0, 3, 5, 8, 10, 13, 15, 18, 20, 23, 25, 30, 35, 40, 45, 50, 55, 60, 65,
    70, 75, 80, 85,
];

//pub const BISHOP_SUPPORTED_BONUS_MG_BY_RANK: [i16; 8] = [0, 0, 3, 8, 13, 18, 25, 0];
//pub const BISHOP_FULLY_BLOCKED: i16 = -150;
pub struct PiecewiseEvaluation {
    my_pawns: u64,
    my_rooks: u64,
    my_bishops: u64,
    my_knights: u64,
    my_queens: u64,
    is_white: bool,
    all_pawns: u64,
    my_pieces: u64,
    all_pieces_without_enemy_king: u64,
}

impl Evaluation for PiecewiseEvaluation {
    fn eval_mg(&self) -> i16 {
        let mut res = 0;
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
            let targets = rook_attack(idx, self.all_pieces_without_enemy_king) & !self.my_pieces;
            res += ROOK_MOBILITY_BONUS_MG[targets.count_ones() as usize];
            rooks ^= 1u64 << idx;
        }
        let mut bishops = self.my_bishops;
        while bishops != 0u64 {
            let idx = bishops.trailing_zeros() as usize;
            let pos = 1u64 << idx;
            let bonus = DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG
                [(bitboards::DIAGONALLY_ADJACENT[idx] & self.my_pawns).count_ones() as usize];
            if bonus <= 0 || pos & (*bitboards::CENTER) != 0u64 {
                res += bonus;
            }
            let targets = bishop_attack(idx, self.all_pieces_without_enemy_king) & !self.my_pieces;
            res += BISHOP_MOBILITY_BONUS_MG[targets.count_ones() as usize];
            bishops ^= pos;
        }
        let mut knights = self.my_knights;
        while knights != 0u64 {
            let idx = knights.trailing_zeros() as usize;
            let pos = 1u64 << idx;
            let targets = knight_attack(idx) & !self.my_pieces;
            res += KNIGHT_MOBILITY_BONUS_MG[targets.count_ones() as usize];
            knights ^= pos;
        }
        let mut queens = self.my_queens;
        while queens != 0u64 {
            let idx = queens.trailing_zeros() as usize;
            let targets = (bishop_attack(idx, self.all_pieces_without_enemy_king)
                | rook_attack(idx, self.all_pieces_without_enemy_king))
                & !self.my_pieces;
            res += QUEEN_MOBILITY_BONUS_MG[targets.count_ones() as usize];
            queens ^= 1u64 << idx;
        }
        res
    }
    fn eval_eg(&self) -> i16 {
        let mut res = 0;
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
            let targets = rook_attack(idx, self.all_pieces_without_enemy_king) & !self.my_pieces;
            res += ROOK_MOBILITY_BONUS_EG[targets.count_ones() as usize];
            rooks ^= 1u64 << idx;
        }
        let mut bishops = self.my_bishops;
        while bishops != 0u64 {
            let idx = bishops.trailing_zeros() as usize;
            let pos = 1u64 << idx;
            let bonus = DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG
                [(bitboards::DIAGONALLY_ADJACENT[idx] & self.my_pawns).count_ones() as usize];
            if bonus <= 0 || pos & (*bitboards::CENTER) != 0u64 {
                res += bonus;
            }
            let targets = bishop_attack(idx, self.all_pieces_without_enemy_king) & !self.my_pieces;
            res += BISHOP_MOBILITY_BONUS_EG[targets.count_ones() as usize];
            bishops ^= pos;
        }
        let mut knights = self.my_knights;
        while knights != 0u64 {
            let idx = knights.trailing_zeros() as usize;
            let pos = 1u64 << idx;
            let targets = knight_attack(idx) & !self.my_pieces;
            res += KNIGHT_MOBILITY_BONUS_EG[targets.count_ones() as usize];
            knights ^= pos;
        }
        let mut queens = self.my_queens;
        while queens != 0u64 {
            let idx = queens.trailing_zeros() as usize;
            let targets = (bishop_attack(idx, self.all_pieces_without_enemy_king)
                | rook_attack(idx, self.all_pieces_without_enemy_king))
                & !self.my_pieces;
            res += QUEEN_MOBILITY_BONUS_EG[targets.count_ones() as usize];
            queens ^= 1u64 << idx;
        }
        res
    }
}

impl ParallelEvaluation for PiecewiseEvaluation {
    fn eval_mg_eg(&self) -> (i16, i16) {
        let mut mg_res = 0;
        let mut eg_res = 0;
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
            let targets = rook_attack(idx, self.all_pieces_without_enemy_king) & !self.my_pieces;
            mg_res += ROOK_MOBILITY_BONUS_MG[targets.count_ones() as usize];
            eg_res += ROOK_MOBILITY_BONUS_EG[targets.count_ones() as usize];
            rooks ^= 1u64 << idx;
        }
        let mut bishops = self.my_bishops;
        while bishops != 0u64 {
            let idx = bishops.trailing_zeros() as usize;
            let pos = 1u64 << idx;
            let bonus_mg = DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG
                [(bitboards::DIAGONALLY_ADJACENT[idx] & self.my_pawns).count_ones() as usize];
            if bonus_mg < 0 || pos & (*bitboards::CENTER) != 0u64 {
                mg_res += bonus_mg;
            }
            let bonus_eg = DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG
                [(bitboards::DIAGONALLY_ADJACENT[idx] & self.my_pawns).count_ones() as usize];
            if bonus_eg < 0 || pos & (*bitboards::CENTER) != 0u64 {
                eg_res += bonus_eg;
            }
            let targets = bishop_attack(idx, self.all_pieces_without_enemy_king) & !self.my_pieces;
            mg_res += BISHOP_MOBILITY_BONUS_MG[targets.count_ones() as usize];
            eg_res += BISHOP_MOBILITY_BONUS_EG[targets.count_ones() as usize];
            bishops ^= pos;
        }
        let mut knights = self.my_knights;
        while knights != 0u64 {
            let idx = knights.trailing_zeros() as usize;
            let pos = 1u64 << idx;
            let targets = knight_attack(idx) & !self.my_pieces;
            mg_res += KNIGHT_MOBILITY_BONUS_MG[targets.count_ones() as usize];
            eg_res += KNIGHT_MOBILITY_BONUS_EG[targets.count_ones() as usize];
            knights ^= pos;
        }
        let mut queens = self.my_queens;
        while queens != 0u64 {
            let idx = queens.trailing_zeros() as usize;
            let targets = (bishop_attack(idx, self.all_pieces_without_enemy_king)
                | rook_attack(idx, self.all_pieces_without_enemy_king))
                & !self.my_pieces;
            mg_res += QUEEN_MOBILITY_BONUS_MG[targets.count_ones() as usize];
            eg_res += QUEEN_MOBILITY_BONUS_EG[targets.count_ones() as usize];
            queens ^= 1u64 << idx;
        }
        (mg_res, eg_res)
    }
}

impl MidGameDisplay for PiecewiseEvaluation {
    fn display_mg(&self) -> String {
        let mut rooks_on_open = 0;
        let mut rooks_on_seventh = 0;
        let mut rook_mobility = 0;
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
            let targets = rook_attack(idx, self.all_pieces_without_enemy_king) & !self.my_pieces;
            rook_mobility += ROOK_MOBILITY_BONUS_MG[targets.count_ones() as usize];
            rooks ^= 1u64 << idx;
        }
        let mut bishop_adjacent_score = 0;
        let mut bishop_mobility = 0;
        let mut bishops = self.my_bishops;
        while bishops != 0u64 {
            let idx = bishops.trailing_zeros() as usize;
            let pos = 1u64 << idx;
            let bonus = DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG
                [(bitboards::DIAGONALLY_ADJACENT[idx] & self.my_pawns).count_ones() as usize];
            if bonus < 0 || pos & (*bitboards::CENTER) != 0u64 {
                bishop_adjacent_score += bonus
            }
            let targets = bishop_attack(idx, self.all_pieces_without_enemy_king) & !self.my_pieces;
            bishop_mobility += BISHOP_MOBILITY_BONUS_MG[targets.count_ones() as usize];
            bishops ^= pos;
        }
        let mut knight_mobility = 0;
        let mut knights = self.my_knights;
        while knights != 0u64 {
            let idx = knights.trailing_zeros() as usize;
            let pos = 1u64 << idx;
            let targets = knight_attack(idx) & !self.my_pieces;
            knight_mobility += KNIGHT_MOBILITY_BONUS_MG[targets.count_ones() as usize];
            knights ^= pos;
        }
        let mut queen_mobility = 0;
        let mut queens = self.my_queens;
        while queens != 0u64 {
            let idx = queens.trailing_zeros() as usize;
            let targets = (bishop_attack(idx, self.all_pieces_without_enemy_king)
                | rook_attack(idx, self.all_pieces_without_enemy_king))
                & !self.my_pieces;
            queen_mobility += QUEEN_MOBILITY_BONUS_MG[targets.count_ones() as usize];
            queens ^= 1u64 << idx;
        }
        let mut res_str = String::new();
        res_str.push_str("\tPiecewiseEvaluation-MidGame\n");
        res_str.push_str(&format!(
            "\t\tRooks on open file:             {} -> {}\n",
            rooks_on_open,
            rooks_on_open * ROOK_ON_OPEN_FILE_BONUS_MG
        ));
        res_str.push_str(&format!(
            "\t\tRooks on seventh rank:          {} -> {}\n",
            rooks_on_seventh,
            rooks_on_seventh * ROOK_ON_SEVENTH_MG
        ));
        res_str.push_str(&format!(
            "\t\tBishop diagonal adjacent pawns:      {}\n",
            bishop_adjacent_score
        ));
        res_str.push_str(&format!(
            "\t\tKnight mobility:                {} -> {}\n",
            self.my_knights.count_ones(),
            knight_mobility
        ));
        res_str.push_str(&format!(
            "\t\tBishop mobility:                {} -> {}\n",
            self.my_bishops.count_ones(),
            bishop_mobility
        ));
        res_str.push_str(&format!(
            "\t\tRook mobility:                  {} -> {}\n",
            self.my_rooks.count_ones(),
            rook_mobility
        ));
        res_str.push_str(&format!(
            "\t\tQueen mobility:                 {} -> {}\n",
            self.my_queens.count_ones(),
            queen_mobility
        ));
        res_str.push_str(&format!(
            "\tSum: {}\n",
            bishop_adjacent_score
                + rooks_on_open * ROOK_ON_OPEN_FILE_BONUS_MG
                + rooks_on_seventh * ROOK_ON_SEVENTH_MG
                + knight_mobility
                + bishop_mobility
                + rook_mobility
                + queen_mobility
        ));
        res_str
    }
}

impl EndGameDisplay for PiecewiseEvaluation {
    fn display_eg(&self) -> String {
        let mut rooks_on_open = 0;
        let mut rooks_on_seventh = 0;
        let mut rook_mobility = 0;
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
            let targets = rook_attack(idx, self.all_pieces_without_enemy_king) & !self.my_pieces;
            rook_mobility += ROOK_MOBILITY_BONUS_EG[targets.count_ones() as usize];
            rooks ^= 1u64 << idx;
        }
        let mut bishop_adjacent_score = 0;
        let mut bishop_mobility = 0;
        let mut bishops = self.my_bishops;
        while bishops != 0u64 {
            let idx = bishops.trailing_zeros() as usize;
            let pos = 1u64 << idx;
            let bonus = DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG
                [(bitboards::DIAGONALLY_ADJACENT[idx] & self.my_pawns).count_ones() as usize];
            if bonus < 0 || pos & (*bitboards::CENTER) != 0u64 {
                bishop_adjacent_score += bonus
            }
            let targets = bishop_attack(idx, self.all_pieces_without_enemy_king) & !self.my_pieces;
            bishop_mobility += BISHOP_MOBILITY_BONUS_EG[targets.count_ones() as usize];
            bishops ^= pos;
        }
        let mut knight_mobility = 0;
        let mut knights = self.my_knights;
        while knights != 0u64 {
            let idx = knights.trailing_zeros() as usize;
            let pos = 1u64 << idx;
            let targets = knight_attack(idx) & !self.my_pieces;
            knight_mobility += KNIGHT_MOBILITY_BONUS_EG[targets.count_ones() as usize];
            knights ^= pos;
        }
        let mut queen_mobility = 0;
        let mut queens = self.my_queens;
        while queens != 0u64 {
            let idx = queens.trailing_zeros() as usize;
            let targets = (bishop_attack(idx, self.all_pieces_without_enemy_king)
                | rook_attack(idx, self.all_pieces_without_enemy_king))
                & !self.my_pieces;
            queen_mobility += QUEEN_MOBILITY_BONUS_EG[targets.count_ones() as usize];
            queens ^= 1u64 << idx;
        }
        let mut res_str = String::new();
        res_str.push_str("\tPiecewiseEvaluation-EndGame\n");
        res_str.push_str(&format!(
            "\t\tRooks on open file:             {} -> {}\n",
            rooks_on_open,
            rooks_on_open * ROOK_ON_OPEN_FILE_BONUS_EG
        ));
        res_str.push_str(&format!(
            "\t\tRooks on seventh rank:          {} -> {}\n",
            rooks_on_seventh,
            rooks_on_seventh * ROOK_ON_SEVENTH_EG
        ));
        res_str.push_str(&format!(
            "\t\tBishop diagonal adjacent pawns:      {}\n",
            bishop_adjacent_score
        ));
        res_str.push_str(&format!(
            "\t\tKnight mobility:                {} -> {}\n",
            self.my_knights.count_ones(),
            knight_mobility
        ));
        res_str.push_str(&format!(
            "\t\tBishop mobility:                {} -> {}\n",
            self.my_bishops.count_ones(),
            bishop_mobility
        ));
        res_str.push_str(&format!(
            "\t\tRook mobility:                  {} -> {}\n",
            self.my_rooks.count_ones(),
            rook_mobility
        ));
        res_str.push_str(&format!(
            "\t\tQueen mobility:                 {} -> {}\n",
            self.my_queens.count_ones(),
            queen_mobility
        ));
        res_str.push_str(&format!(
            "\tSum: {}\n",
            bishop_adjacent_score
                + rooks_on_open * ROOK_ON_OPEN_FILE_BONUS_EG
                + rooks_on_seventh * ROOK_ON_SEVENTH_EG
                + bishop_mobility
                + rook_mobility
                + queen_mobility
        ));
        res_str
    }
}

pub fn piecewise_eval(
    my_pawns: u64,
    my_rooks: u64,
    my_bishops: u64,
    my_knights: u64,
    my_queens: u64,
    is_white: bool,
    all_pawns: u64,
    my_pieces: u64,
    all_pieces_without_enemy_king: u64,
) -> PiecewiseEvaluation {
    PiecewiseEvaluation {
        my_pawns,
        my_rooks,
        my_bishops,
        my_knights,
        my_queens,
        is_white,
        all_pawns,
        my_pieces,
        all_pieces_without_enemy_king,
    }
}
