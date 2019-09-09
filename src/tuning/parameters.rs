use crate::evaluation::params::*;
use crate::evaluation::{EG, MG};
use std::fmt::{Debug, Formatter, Result};
use std::fs;
#[derive(Clone)]
pub struct SafetyTable {
    pub safety_table: [f64; 100],
}
impl Debug for SafetyTable {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        for i in 0..100 {
            res_str.push_str(&format!("{}, ", self.safety_table[i]));
        }
        write!(formatter, "safety_table: [{}];", res_str)
    }
}
#[derive(Clone, Debug)]
pub struct Parameters {
    pub tempo_bonus: [f64; 2],
    pub shielding_pawn_missing: [[f64; 4]; 2],
    pub shielding_pawn_onopen_missing: [[f64; 4]; 2],
    pub pawn_doubled: [f64; 2],
    pub pawn_isolated: [f64; 2],
    pub pawn_backward: [f64; 2],
    pub pawn_supported: [[[f64; 8]; 8]; 2],
    pub pawn_attack_center: [f64; 2],
    pub pawn_mobility: [f64; 2],
    pub pawn_passed: [[f64; 7]; 2],
    pub pawn_passed_notblocked: [[f64; 7]; 2],
    pub rook_behind_support_passer: [f64; 2],
    pub rook_behind_enemy_passer: [f64; 2],
    pub pawn_passed_weak: [f64; 2],
    pub knight_supported: [f64; 2],
    pub knight_outpost_table: [[[f64; 8]; 8]; 2],
    pub rook_on_open: [f64; 2],
    pub rook_on_seventh: [f64; 2],
    pub pawn_piece_value: [f64; 2],
    pub knight_piece_value: [f64; 2],
    pub knight_value_with_pawns: [f64; 17],
    pub bishop_piece_value: [f64; 2],
    pub bishop_pair: [f64; 2],
    pub rook_piece_value: [f64; 2],
    pub queen_piece_value: [f64; 2],
    pub diagonally_adjacent_squares_withpawns: [[f64; 5]; 2],
    pub knight_mobility: [[f64; 9]; 2],
    pub bishop_mobility: [[f64; 14]; 2],
    pub rook_mobility: [[f64; 15]; 2],
    pub queen_mobility: [[f64; 28]; 2],
    pub attack_weight: [[f64; 8]; 2],
    pub safety_table: [SafetyTable; 2],
    pub knight_attack_value: [f64; 2],
    pub bishop_attack_value: [f64; 2],
    pub rook_attack_value: [f64; 2],
    pub queen_attack_value: [f64; 2],
    pub psqt_pawn: [[[f64; 8]; 8]; 2],
    pub psqt_knight: [[[f64; 8]; 8]; 2],
    pub psqt_bishop: [[[f64; 8]; 8]; 2],
    pub psqt_king: [[[f64; 8]; 8]; 2],
}

pub fn psqt_to_string(psqt: &[[f64; 8]; 8]) -> String {
    let mut res_str = String::new();
    res_str.push_str("[");
    for x in psqt.iter() {
        res_str.push_str(&format!("{}, ", array_to_string(x)));
    }
    res_str.push_str("]");
    res_str
}

pub fn array_to_string(array: &[f64]) -> String {
    let mut res_str = String::new();
    res_str.push_str("[");
    for x in array.iter() {
        res_str.push_str(&format!("{}, ", x.round() as isize));
    }
    res_str.push_str("]");
    res_str
}

pub fn apply_gradient_arr(to: &mut [f64], gradient_arr: &[f64], norm: f64) {
    for i in 0..to.len() {
        to[i] += gradient_arr[i] / norm;
    }
}

pub fn apply_gradient_psqt(
    to: &mut [[[f64; 8]; 8]; 2],
    gradient_psqt: &[[[f64; 8]; 8]; 2],
    norm: f64,
) {
    for i in 0..2 {
        for j in 0..8 {
            for k in 0..8 {
                to[i][j][k] += gradient_psqt[i][j][k] / norm;
            }
        }
    }
}

impl Parameters {
    pub fn write_to_file(&self, file: &str) {
        fs::write(file, self.to_string().as_str()).expect("Unable to write file");
    }
    pub fn to_string(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str(&format!(
            "pub const TEMPO_BONUS_MG: i16 = {};\n",
            self.tempo_bonus[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const TEMPO_BONUS_EG: i16 = {};\n",
            self.tempo_bonus[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const SHIELDING_PAWN_MISSING_MG: [i16;4] = {};\n",
            array_to_string(&self.shielding_pawn_missing[MG])
        ));
        res_str.push_str(&format!(
            "pub const SHIELDING_PAWN_MISSING_EG: [i16;4] = {};\n",
            array_to_string(&self.shielding_pawn_missing[EG])
        ));
        res_str.push_str(&format!(
            "pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG: [i16;4] = {};\n",
            array_to_string(&self.shielding_pawn_onopen_missing[MG])
        ));
        res_str.push_str(&format!(
            "pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG: [i16;4] = {};\n",
            array_to_string(&self.shielding_pawn_onopen_missing[EG])
        ));
        res_str.push_str(&format!(
            "pub const PAWN_DOUBLED_VALUE_MG: i16 = {};\n",
            self.pawn_doubled[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_DOUBLED_VALUE_EG: i16 = {};\n",
            self.pawn_doubled[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_ISOLATED_VALUE_MG: i16 = {};\n",
            self.pawn_isolated[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_ISOLATED_VALUE_EG: i16 = {};\n",
            self.pawn_isolated[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_BACKWARD_VALUE_MG: i16 = {};\n",
            self.pawn_backward[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_BACKWARD_VALUE_EG: i16 = {};\n",
            self.pawn_backward[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_SUPPORTED_VALUE_MG: [[i16;8];8] = {};\n",
            psqt_to_string(&self.pawn_supported[MG])
        ));
        res_str.push_str(&format!(
            "pub const PAWN_SUPPORTED_VALUE_EG: [[i16;8];8] = {};\n",
            psqt_to_string(&self.pawn_supported[EG])
        ));
        res_str.push_str(&format!(
            "pub const PAWN_ATTACK_CENTER_MG: i16 = {};\n",
            self.pawn_attack_center[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_ATTACK_CENTER_EG: i16 = {};\n",
            self.pawn_attack_center[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_MOBILITY_MG: i16 = {};\n",
            self.pawn_mobility[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_MOBILITY_EG: i16 = {};\n",
            self.pawn_mobility[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_PASSED_VALUES_MG: [i16;7] = {};\n",
            array_to_string(&self.pawn_passed[MG])
        ));
        res_str.push_str(&format!(
            "pub const PAWN_PASSED_VALUES_EG: [i16;7] = {};\n",
            array_to_string(&self.pawn_passed[EG])
        ));
        res_str.push_str(&format!(
            "pub const PAWN_PASSED_NOT_BLOCKED_VALUES_MG: [i16;7] = {};\n",
            array_to_string(&self.pawn_passed_notblocked[MG])
        ));
        res_str.push_str(&format!(
            "pub const PAWN_PASSED_NOT_BLOCKED_VALUES_EG: [i16;7] = {};\n",
            array_to_string(&self.pawn_passed_notblocked[EG])
        ));
        res_str.push_str(&format!(
            "pub const ROOK_BEHIND_SUPPORT_PASSER_MG: i16 = {};\n",
            self.rook_behind_support_passer[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_BEHIND_SUPPORT_PASSER_EG: i16 = {};\n",
            self.rook_behind_support_passer[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_BEHIND_ENEMY_PASSER_MG: i16 = {};\n",
            self.rook_behind_enemy_passer[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_BEHIND_ENEMY_PASSER_EG: i16 = {};\n",
            self.rook_behind_enemy_passer[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_PASSED_WEAK_MG: i16 = {};\n",
            self.pawn_passed_weak[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_PASSED_WEAK_EG: i16 = {};\n",
            self.pawn_passed_weak[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_SUPPORTED_BY_PAWN_MG: i16 = {};\n",
            self.knight_supported[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_SUPPORTED_BY_PAWN_EG: i16 = {};\n",
            self.knight_supported[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_OUTPOST_MG_TABLE: [[i16;8];8] = {};\n",
            psqt_to_string(&self.knight_outpost_table[MG])
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_OUTPOST_EG_TABLE: [[i16;8];8] = {};\n",
            psqt_to_string(&self.knight_outpost_table[EG])
        ));
        res_str.push_str(&format!(
            "pub const ROOK_ON_OPEN_FILE_BONUS_MG: i16 = {};\n",
            self.rook_on_open[MG].round() as isize,
        ));
        res_str.push_str(&format!(
            "pub const ROOK_ON_OPEN_FILE_BONUS_EG: i16 = {};\n",
            self.rook_on_open[EG].round() as isize,
        ));
        res_str.push_str(&format!(
            "pub const ROOK_ON_SEVENTH_MG: i16 = {};\n",
            self.rook_on_seventh[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_ON_SEVENTH_EG: i16 = {};\n",
            self.rook_on_seventh[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_PIECE_VALUE_MG: i16 = {};\n",
            self.pawn_piece_value[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_PIECE_VALUE_EG: i16 = {};\n",
            self.pawn_piece_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_PIECE_VALUE_MG: i16 = {};\n",
            self.knight_piece_value[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_PIECE_VALUE_EG: i16 = {};\n",
            self.knight_piece_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_VALUE_WITH_PAWNS: [i16;17] = {};\n",
            array_to_string(&self.knight_value_with_pawns)
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_PIECE_VALUE_MG: i16 = {};\n",
            self.bishop_piece_value[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_PIECE_VALUE_EG: i16 = {};\n",
            self.bishop_piece_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_PAIR_BONUS_MG: i16 = {};\n",
            self.bishop_pair[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_PAIR_BONUS_EG: i16 = {};\n",
            self.bishop_pair[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_PIECE_VALUE_MG: i16 = {};\n",
            self.rook_piece_value[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_PIECE_VALUE_EG: i16 = {};\n",
            self.rook_piece_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_PIECE_VALUE_MG: i16 = {};\n",
            self.queen_piece_value[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_PIECE_VALUE_EG: i16 = {};\n",
            self.queen_piece_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG: [i16;5] = {};\n",
            array_to_string(&self.diagonally_adjacent_squares_withpawns[MG])
        ));
        res_str.push_str(&format!(
            "pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG: [i16;5] = {};\n",
            array_to_string(&self.diagonally_adjacent_squares_withpawns[EG])
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_MOBILITY_BONUS_MG: [i16;9] = {};\n",
            array_to_string(&self.knight_mobility[MG])
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_MOBILITY_BONUS_EG: [i16;9] = {};\n",
            array_to_string(&self.knight_mobility[EG])
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_MOBILITY_BONUS_MG: [i16;14] = {};\n",
            array_to_string(&self.bishop_mobility[MG])
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_MOBILITY_BONUS_EG: [i16;14] = {};\n",
            array_to_string(&self.bishop_mobility[EG])
        ));
        res_str.push_str(&format!(
            "pub const ROOK_MOBILITY_BONUS_MG: [i16;15] = {};\n",
            array_to_string(&self.rook_mobility[MG])
        ));
        res_str.push_str(&format!(
            "pub const ROOK_MOBILITY_BONUS_EG: [i16;15] = {};\n",
            array_to_string(&self.rook_mobility[EG])
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_MOBILITY_BONUS_MG: [i16;28] = {};\n",
            array_to_string(&self.queen_mobility[MG])
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_MOBILITY_BONUS_EG: [i16;28] = {};\n",
            array_to_string(&self.queen_mobility[EG])
        ));
        res_str.push_str(&format!(
            "pub const ATTACK_WEIGHT_MG: [i16;8] = {};\n",
            array_to_string(&self.attack_weight[MG])
        ));
        res_str.push_str(&format!(
            "pub const SAFETY_TABLE_MG: [i16;100] = {};\n",
            array_to_string(&self.safety_table[MG].safety_table)
        ));
        res_str.push_str(&format!(
            "pub const ATTACK_WEIGHT_EG: [i16;8] = {};\n",
            array_to_string(&self.attack_weight[EG])
        ));
        res_str.push_str(&format!(
            "pub const SAFETY_TABLE_EG: [i16;100] = {};\n",
            array_to_string(&self.safety_table[EG].safety_table)
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_ATTACK_WORTH_MG: i16 = {};\n",
            self.knight_attack_value[MG].round() as isize,
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_ATTACK_WORTH_EG: i16 = {};\n",
            self.knight_attack_value[EG].round() as isize,
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_ATTACK_WORTH_MG: i16 = {};\n",
            self.bishop_attack_value[MG].round() as isize,
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_ATTACK_WORTH_EG: i16 = {};\n",
            self.bishop_attack_value[EG].round() as isize,
        ));
        res_str.push_str(&format!(
            "pub const ROOK_ATTACK_WORTH_MG: i16 = {};\n",
            self.rook_attack_value[MG].round() as isize,
        ));
        res_str.push_str(&format!(
            "pub const ROOK_ATTACK_WORTH_EG: i16 = {};\n",
            self.rook_attack_value[EG].round() as isize,
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_ATTACK_WORTH_MG: i16 = {};\n",
            self.queen_attack_value[MG].round() as isize,
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_ATTACK_WORTH_EG: i16 = {};\n",
            self.queen_attack_value[EG].round() as isize,
        ));
        res_str.push_str(&format!(
            "pub const PSQT_PAWN_MG: [[i16;8];8] = {};\n",
            psqt_to_string(&self.psqt_pawn[MG])
        ));
        res_str.push_str(&format!(
            "pub const PSQT_PAWN_EG: [[i16;8];8] = {};\n",
            psqt_to_string(&self.psqt_pawn[EG])
        ));
        res_str.push_str(&format!(
            "pub const PSQT_KNIGHT_MG: [[i16;8];8] = {};\n",
            psqt_to_string(&self.psqt_knight[MG])
        ));
        res_str.push_str(&format!(
            "pub const PSQT_KNIGHT_EG: [[i16;8];8] = {};\n",
            psqt_to_string(&self.psqt_knight[EG])
        ));
        res_str.push_str(&format!(
            "pub const PSQT_BISHOP_MG: [[i16;8];8] = {};\n",
            psqt_to_string(&self.psqt_bishop[MG])
        ));
        res_str.push_str(&format!(
            "pub const PSQT_BISHOP_EG: [[i16;8];8] = {};\n",
            psqt_to_string(&self.psqt_bishop[EG])
        ));
        res_str.push_str(&format!(
            "pub const PSQT_KING_MG: [[i16;8];8] = {};\n",
            psqt_to_string(&self.psqt_king[MG])
        ));
        res_str.push_str(&format!(
            "pub const PSQT_KING_EG: [[i16;8];8] = {};\n",
            psqt_to_string(&self.psqt_king[EG])
        ));
        res_str
    }

    pub fn default() -> Self {
        let mut shielding_pawn_missing: [[f64; 4]; 2] = [[0.; 4]; 2];
        for i in 0..4 {
            shielding_pawn_missing[MG][i] = f64::from(SHIELDING_PAWN_MISSING_MG[i]);
            shielding_pawn_missing[EG][i] = f64::from(SHIELDING_PAWN_MISSING_EG[i]);
        }
        let mut shielding_pawn_onopen_missing: [[f64; 4]; 2] = [[0.; 4]; 2];
        for i in 0..4 {
            shielding_pawn_onopen_missing[MG][i] =
                f64::from(SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG[i]);
            shielding_pawn_onopen_missing[EG][i] =
                f64::from(SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG[i]);
        }
        let mut pawn_passed: [[f64; 7]; 2] = [[0.; 7]; 2];
        for i in 0..7 {
            pawn_passed[MG][i] = f64::from(PAWN_PASSED_VALUES_MG[i]);
            pawn_passed[EG][i] = f64::from(PAWN_PASSED_VALUES_EG[i]);
        }
        let mut pawn_passed_notblocked: [[f64; 7]; 2] = [[0.; 7]; 2];
        for i in 0..7 {
            pawn_passed_notblocked[MG][i] = f64::from(PAWN_PASSED_NOT_BLOCKED_VALUES_MG[i]);
            pawn_passed_notblocked[EG][i] = f64::from(PAWN_PASSED_NOT_BLOCKED_VALUES_EG[i]);
        }
        let mut knight_outpost_table: [[[f64; 8]; 8]; 2] = [[[0.; 8]; 8]; 2];
        for i in 0..8 {
            for j in 0..8 {
                knight_outpost_table[MG][i][j] = f64::from(KNIGHT_OUTPOST_MG_TABLE[i][j]);
                knight_outpost_table[EG][i][j] = f64::from(KNIGHT_OUTPOST_EG_TABLE[i][j]);
            }
        }
        let mut knight_value_with_pawns: [f64; 17] = [0.; 17];
        for i in 0..17 {
            knight_value_with_pawns[i] = f64::from(KNIGHT_VALUE_WITH_PAWNS[i]);
        }
        let mut diagonally_adjacent_squares_withpawns: [[f64; 5]; 2] = [[0.; 5]; 2];
        for i in 0..5 {
            diagonally_adjacent_squares_withpawns[MG][i] =
                f64::from(DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG[i]);
            diagonally_adjacent_squares_withpawns[EG][i] =
                f64::from(DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG[i]);
        }
        let mut knight_mobility: [[f64; 9]; 2] = [[0.; 9]; 2];
        for i in 0..9 {
            knight_mobility[MG][i] = f64::from(KNIGHT_MOBILITY_BONUS_MG[i]);
            knight_mobility[EG][i] = f64::from(KNIGHT_MOBILITY_BONUS_EG[i]);
        }
        let mut bishop_mobility: [[f64; 14]; 2] = [[0.; 14]; 2];
        for i in 0..14 {
            bishop_mobility[MG][i] = f64::from(BISHOP_MOBILITY_BONUS_MG[i]);
            bishop_mobility[EG][i] = f64::from(BISHOP_MOBILITY_BONUS_EG[i]);
        }
        let mut rook_mobility: [[f64; 15]; 2] = [[0.; 15]; 2];
        for i in 0..15 {
            rook_mobility[MG][i] = f64::from(ROOK_MOBILITY_BONUS_MG[i]);
            rook_mobility[EG][i] = f64::from(ROOK_MOBILITY_BONUS_EG[i]);
        }
        let mut queen_mobility: [[f64; 28]; 2] = [[0.; 28]; 2];
        for i in 0..28 {
            queen_mobility[MG][i] = f64::from(QUEEN_MOBILITY_BONUS_MG[i]);
            queen_mobility[EG][i] = f64::from(QUEEN_MOBILITY_BONUS_EG[i]);
        }
        let mut attack_weight: [[f64; 8]; 2] = [[0.; 8]; 2];
        for i in 0..8 {
            attack_weight[MG][i] = f64::from(ATTACK_WEIGHT_MG[i]);
            attack_weight[EG][i] = f64::from(ATTACK_WEIGHT_EG[i]);
        }
        let mut safety_table: [SafetyTable; 2] = [
            SafetyTable {
                safety_table: [0.; 100],
            },
            SafetyTable {
                safety_table: [0.; 100],
            },
        ];
        for i in 0..100 {
            safety_table[MG].safety_table[i] = f64::from(SAFETY_TABLE_MG[i]);
            safety_table[EG].safety_table[i] = f64::from(SAFETY_TABLE_EG[i]);
        }

        let mut psqt_pawn: [[[f64; 8]; 8]; 2] = [[[0.; 8]; 8]; 2];
        for i in 0..8 {
            for j in 0..8 {
                psqt_pawn[MG][i][j] = f64::from(PSQT_PAWN_MG[i][j]);
                psqt_pawn[EG][i][j] = f64::from(PSQT_PAWN_EG[i][j]);
            }
        }
        let mut psqt_knight: [[[f64; 8]; 8]; 2] = [[[0.; 8]; 8]; 2];
        for i in 0..8 {
            for j in 0..8 {
                psqt_knight[MG][i][j] = f64::from(PSQT_KNIGHT_MG[i][j]);
                psqt_knight[EG][i][j] = f64::from(PSQT_KNIGHT_EG[i][j]);
            }
        }
        let mut psqt_bishop: [[[f64; 8]; 8]; 2] = [[[0.; 8]; 8]; 2];
        for i in 0..8 {
            for j in 0..8 {
                psqt_bishop[MG][i][j] = f64::from(PSQT_BISHOP_MG[i][j]);
                psqt_bishop[EG][i][j] = f64::from(PSQT_BISHOP_EG[i][j]);
            }
        }
        let mut psqt_king: [[[f64; 8]; 8]; 2] = [[[0.; 8]; 8]; 2];
        for i in 0..8 {
            for j in 0..8 {
                psqt_king[MG][i][j] = f64::from(PSQT_KING_MG[i][j]);
                psqt_king[EG][i][j] = f64::from(PSQT_KING_EG[i][j]);
            }
        }
        let mut psqt_pawn_supported: [[[f64; 8]; 8]; 2] = [[[0.; 8]; 8]; 2];
        for i in 0..8 {
            for j in 0..8 {
                psqt_pawn_supported[MG][i][j] = f64::from(PAWN_SUPPORTED_VALUE_MG[i][j]);
                psqt_pawn_supported[EG][i][j] = f64::from(PAWN_SUPPORTED_VALUE_EG[i][j]);
            }
        }
        Parameters {
            tempo_bonus: [f64::from(TEMPO_BONUS_MG), f64::from(TEMPO_BONUS_EG)],
            shielding_pawn_missing,
            shielding_pawn_onopen_missing,
            pawn_doubled: [
                f64::from(PAWN_DOUBLED_VALUE_MG),
                f64::from(PAWN_DOUBLED_VALUE_EG),
            ],
            pawn_isolated: [
                f64::from(PAWN_ISOLATED_VALUE_MG),
                f64::from(PAWN_ISOLATED_VALUE_EG),
            ],
            pawn_backward: [
                f64::from(PAWN_BACKWARD_VALUE_MG),
                f64::from(PAWN_BACKWARD_VALUE_EG),
            ],
            pawn_supported: psqt_pawn_supported,
            pawn_attack_center: [
                f64::from(PAWN_ATTACK_CENTER_MG),
                f64::from(PAWN_ATTACK_CENTER_EG),
            ],
            pawn_mobility: [f64::from(PAWN_MOBILITY_MG), f64::from(PAWN_MOBILITY_EG)],
            pawn_passed,
            pawn_passed_notblocked,
            rook_behind_support_passer: [
                f64::from(ROOK_BEHIND_SUPPORT_PASSER_MG),
                f64::from(ROOK_BEHIND_SUPPORT_PASSER_EG),
            ],
            rook_behind_enemy_passer: [
                f64::from(ROOK_BEHIND_ENEMY_PASSER_MG),
                f64::from(ROOK_BEHIND_ENEMY_PASSER_EG),
            ],
            pawn_passed_weak: [
                f64::from(PAWN_PASSED_WEAK_MG),
                f64::from(PAWN_PASSED_WEAK_EG),
            ],
            knight_supported: [
                f64::from(KNIGHT_SUPPORTED_BY_PAWN_MG),
                f64::from(KNIGHT_SUPPORTED_BY_PAWN_EG),
            ],
            knight_outpost_table,
            rook_on_open: [
                f64::from(ROOK_ON_OPEN_FILE_BONUS_MG),
                f64::from(ROOK_ON_OPEN_FILE_BONUS_EG),
            ],
            rook_on_seventh: [f64::from(ROOK_ON_SEVENTH_MG), f64::from(ROOK_ON_SEVENTH_EG)],
            pawn_piece_value: [
                f64::from(PAWN_PIECE_VALUE_MG),
                f64::from(PAWN_PIECE_VALUE_EG),
            ],
            knight_piece_value: [
                f64::from(KNIGHT_PIECE_VALUE_MG),
                f64::from(KNIGHT_PIECE_VALUE_EG),
            ],
            knight_value_with_pawns,
            bishop_piece_value: [
                f64::from(BISHOP_PIECE_VALUE_MG),
                f64::from(BISHOP_PIECE_VALUE_EG),
            ],
            bishop_pair: [
                f64::from(BISHOP_PAIR_BONUS_MG),
                f64::from(BISHOP_PAIR_BONUS_EG),
            ],
            rook_piece_value: [
                f64::from(ROOK_PIECE_VALUE_MG),
                f64::from(ROOK_PIECE_VALUE_EG),
            ],
            queen_piece_value: [
                f64::from(QUEEN_PIECE_VALUE_MG),
                f64::from(QUEEN_PIECE_VALUE_EG),
            ],
            knight_attack_value: [
                f64::from(KNIGHT_ATTACK_WORTH_MG),
                f64::from(KNIGHT_ATTACK_WORTH_EG),
            ],
            bishop_attack_value: [
                f64::from(BISHOP_ATTACK_WORTH_MG),
                f64::from(BISHOP_ATTACK_WORTH_EG),
            ],
            rook_attack_value: [
                f64::from(ROOK_ATTACK_WORTH_MG),
                f64::from(ROOK_ATTACK_WORTH_EG),
            ],
            queen_attack_value: [
                f64::from(QUEEN_ATTACK_WORTH_MG),
                f64::from(QUEEN_ATTACK_WORTH_EG),
            ],
            diagonally_adjacent_squares_withpawns,
            knight_mobility,
            bishop_mobility,
            rook_mobility,
            queen_mobility,
            attack_weight,
            safety_table,
            psqt_pawn,
            psqt_knight,
            psqt_bishop,
            psqt_king,
        }
    }

    pub fn zero() -> Self {
        Parameters {
            tempo_bonus: [0.; 2],
            shielding_pawn_missing: [[0.; 4]; 2],
            shielding_pawn_onopen_missing: [[0.; 4]; 2],
            pawn_doubled: [0.; 2],
            pawn_isolated: [0.; 2],
            pawn_backward: [0.; 2],
            pawn_supported: [[[0.; 8]; 8]; 2],
            pawn_attack_center: [0.; 2],
            pawn_mobility: [0.; 2],
            pawn_passed: [[0.; 7]; 2],
            pawn_passed_notblocked: [[0.; 7]; 2],
            rook_behind_support_passer: [0.; 2],
            rook_behind_enemy_passer: [0.; 2],
            pawn_passed_weak: [0.; 2],
            knight_supported: [0.; 2],
            knight_outpost_table: [[[0.; 8]; 8]; 2],
            rook_on_open: [0.; 2],
            rook_on_seventh: [0.; 2],
            pawn_piece_value: [0.; 2],
            knight_piece_value: [0.; 2],
            knight_value_with_pawns: [0.; 17],
            bishop_piece_value: [0.; 2],
            bishop_pair: [0.; 2],
            rook_piece_value: [0.; 2],
            queen_piece_value: [0.; 2],
            diagonally_adjacent_squares_withpawns: [[0.; 5]; 2],
            knight_mobility: [[0.; 9]; 2],
            bishop_mobility: [[0.; 14]; 2],
            rook_mobility: [[0.; 15]; 2],
            queen_mobility: [[0.; 28]; 2],
            attack_weight: [[0.; 8]; 2],
            safety_table: [
                SafetyTable {
                    safety_table: [0.; 100],
                },
                SafetyTable {
                    safety_table: [0.; 100],
                },
            ],
            knight_attack_value: [0.; 2],
            bishop_attack_value: [0.; 2],
            rook_attack_value: [0.; 2],
            queen_attack_value: [0.; 2],
            psqt_pawn: [[[0.; 8]; 8]; 2],
            psqt_knight: [[[0.; 8]; 8]; 2],
            psqt_bishop: [[[0.; 8]; 8]; 2],
            psqt_king: [[[0.; 8]; 8]; 2],
        }
    }

    pub fn apply_gradient(&mut self, gradient: &Parameters, norm: f64) {
        for i in 0..2 {
            apply_gradient_arr(
                &mut self.shielding_pawn_missing[i],
                &gradient.shielding_pawn_missing[i],
                norm,
            );
            apply_gradient_arr(
                &mut self.shielding_pawn_onopen_missing[i],
                &gradient.shielding_pawn_onopen_missing[i],
                norm,
            );
        }

        for i in 0..2 {
            self.tempo_bonus[i] += gradient.tempo_bonus[i] / norm;
            self.pawn_doubled[i] += gradient.pawn_doubled[i] / norm;
            self.pawn_isolated[i] += gradient.pawn_isolated[i] / norm;
            self.pawn_backward[i] += gradient.pawn_backward[i] / norm;
            self.pawn_attack_center[i] += gradient.pawn_attack_center[i] / norm;
            self.pawn_mobility[i] += gradient.pawn_mobility[i] / norm;
            self.rook_behind_support_passer[i] += gradient.rook_behind_support_passer[i] / norm;
            self.rook_behind_enemy_passer[i] += gradient.rook_behind_enemy_passer[i] / norm;
            self.pawn_passed_weak[i] += gradient.pawn_passed_weak[i] / norm;
            self.knight_supported[i] += gradient.knight_supported[i] / norm;
            self.rook_on_open[i] += gradient.rook_on_open[i] / norm;
            self.rook_on_seventh[i] += gradient.rook_on_seventh[i] / norm;
            self.pawn_piece_value[i] += gradient.pawn_piece_value[i] / norm;
            self.knight_piece_value[i] += gradient.knight_piece_value[i] / norm;
            self.bishop_piece_value[i] += gradient.bishop_piece_value[i] / norm;
            self.bishop_pair[i] += gradient.bishop_pair[i] / norm;
            self.rook_piece_value[i] += gradient.rook_piece_value[i] / norm;
            self.queen_piece_value[i] += gradient.queen_piece_value[i] / norm;
            self.knight_attack_value[i] += gradient.knight_attack_value[i] / norm;
            self.bishop_attack_value[i] += gradient.bishop_attack_value[i] / norm;
            self.rook_attack_value[i] += gradient.rook_attack_value[i] / norm;
            self.queen_attack_value[i] += gradient.queen_attack_value[i] / norm;
        }
        for i in 0..2 {
            apply_gradient_arr(&mut self.pawn_passed[i], &gradient.pawn_passed[i], norm);
            apply_gradient_arr(
                &mut self.pawn_passed_notblocked[i],
                &gradient.pawn_passed_notblocked[i],
                norm,
            );
        }
        apply_gradient_psqt(&mut self.pawn_supported, &gradient.pawn_supported, norm);
        apply_gradient_psqt(
            &mut self.knight_outpost_table,
            &gradient.knight_outpost_table,
            norm,
        );
        apply_gradient_psqt(&mut self.psqt_pawn, &gradient.psqt_pawn, norm);
        apply_gradient_psqt(&mut self.psqt_knight, &gradient.psqt_knight, norm);
        apply_gradient_psqt(&mut self.psqt_bishop, &gradient.psqt_bishop, norm);
        apply_gradient_psqt(&mut self.psqt_king, &gradient.psqt_king, norm);

        apply_gradient_arr(
            &mut self.knight_value_with_pawns,
            &gradient.knight_value_with_pawns,
            norm,
        );

        for i in 0..2 {
            apply_gradient_arr(
                &mut self.diagonally_adjacent_squares_withpawns[i],
                &gradient.diagonally_adjacent_squares_withpawns[i],
                norm,
            );
            apply_gradient_arr(
                &mut self.knight_mobility[i],
                &gradient.knight_mobility[i],
                norm,
            );
            apply_gradient_arr(
                &mut self.bishop_mobility[i],
                &gradient.bishop_mobility[i],
                norm,
            );
            apply_gradient_arr(&mut self.rook_mobility[i], &gradient.rook_mobility[i], norm);
            apply_gradient_arr(
                &mut self.queen_mobility[i],
                &gradient.queen_mobility[i],
                norm,
            );
            apply_gradient_arr(&mut self.attack_weight[i], &gradient.attack_weight[i], norm);
            apply_gradient_arr(
                &mut self.safety_table[i].safety_table,
                &gradient.safety_table[i].safety_table,
                norm,
            );
        }
    }
}
