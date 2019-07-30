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
    pub pawn_supported: [f64; 2],
    pub pawn_attack_center: [f64; 2],
    pub pawn_passed: [[f64; 7]; 2],
    pub pawn_passed_notblocked: [[f64; 7]; 2],
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
    pub attack_weight: [f64; 8],
    pub safety_table: SafetyTable,
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
            "pub const PAWN_SUPPORTED_VALUE_MG: i16 = {};\n",
            self.pawn_supported[MG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_SUPPORTED_VALUE_EG: i16 = {};\n",
            self.pawn_supported[EG].round() as isize
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
            "pub const ATTACK_WEIGHT: [i16;8] = {};\n",
            array_to_string(&self.attack_weight)
        ));
        res_str.push_str(&format!(
            "pub const SAFETY_TABLE: [i16;100] = {};\n",
            array_to_string(&self.safety_table.safety_table)
        ));
        res_str.push_str(&format!("pub const KNIGHT_ATTACK_WORTH: i16 = 2;\n"));
        res_str.push_str(&format!("pub const BISHOP_ATTACK_WORTH: i16 = 2;\n"));
        res_str.push_str(&format!("pub const ROOK_ATTACK_WORTH: i16 = 3;\n"));
        res_str.push_str(&format!("pub const QUEEN_ATTACK_WORTH: i16 = 5;\n"));
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
            shielding_pawn_missing[MG][i] = SHIELDING_PAWN_MISSING_MG[i] as f64;
            shielding_pawn_missing[EG][i] = SHIELDING_PAWN_MISSING_EG[i] as f64;
        }
        let mut shielding_pawn_onopen_missing: [[f64; 4]; 2] = [[0.; 4]; 2];
        for i in 0..4 {
            shielding_pawn_onopen_missing[MG][i] = SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG[i] as f64;
            shielding_pawn_onopen_missing[EG][i] = SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG[i] as f64;
        }
        let mut pawn_passed: [[f64; 7]; 2] = [[0.; 7]; 2];
        for i in 0..7 {
            pawn_passed[MG][i] = PAWN_PASSED_VALUES_MG[i] as f64;
            pawn_passed[EG][i] = PAWN_PASSED_VALUES_EG[i] as f64;
        }
        let mut pawn_passed_notblocked: [[f64; 7]; 2] = [[0.; 7]; 2];
        for i in 0..7 {
            pawn_passed_notblocked[MG][i] = PAWN_PASSED_NOT_BLOCKED_VALUES_MG[i] as f64;
            pawn_passed_notblocked[EG][i] = PAWN_PASSED_NOT_BLOCKED_VALUES_EG[i] as f64;
        }
        let mut knight_outpost_table: [[[f64; 8]; 8]; 2] = [[[0.; 8]; 8]; 2];
        for i in 0..8 {
            for j in 0..8 {
                knight_outpost_table[MG][i][j] = KNIGHT_OUTPOST_MG_TABLE[i][j] as f64;
                knight_outpost_table[EG][i][j] = KNIGHT_OUTPOST_EG_TABLE[i][j] as f64;
            }
        }
        let mut knight_value_with_pawns: [f64; 17] = [0.; 17];
        for i in 0..17 {
            knight_value_with_pawns[i] = KNIGHT_VALUE_WITH_PAWNS[i] as f64;
        }
        let mut diagonally_adjacent_squares_withpawns: [[f64; 5]; 2] = [[0.; 5]; 2];
        for i in 0..5 {
            diagonally_adjacent_squares_withpawns[MG][i] =
                DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG[i] as f64;
            diagonally_adjacent_squares_withpawns[EG][i] =
                DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG[i] as f64;
        }
        let mut knight_mobility: [[f64; 9]; 2] = [[0.; 9]; 2];
        for i in 0..9 {
            knight_mobility[MG][i] = KNIGHT_MOBILITY_BONUS_MG[i] as f64;
            knight_mobility[EG][i] = KNIGHT_MOBILITY_BONUS_EG[i] as f64;
        }
        let mut bishop_mobility: [[f64; 14]; 2] = [[0.; 14]; 2];
        for i in 0..14 {
            bishop_mobility[MG][i] = BISHOP_MOBILITY_BONUS_MG[i] as f64;
            bishop_mobility[EG][i] = BISHOP_MOBILITY_BONUS_EG[i] as f64;
        }
        let mut rook_mobility: [[f64; 15]; 2] = [[0.; 15]; 2];
        for i in 0..15 {
            rook_mobility[MG][i] = ROOK_MOBILITY_BONUS_MG[i] as f64;
            rook_mobility[EG][i] = ROOK_MOBILITY_BONUS_EG[i] as f64;
        }
        let mut queen_mobility: [[f64; 28]; 2] = [[0.; 28]; 2];
        for i in 0..28 {
            queen_mobility[MG][i] = QUEEN_MOBILITY_BONUS_MG[i] as f64;
            queen_mobility[EG][i] = QUEEN_MOBILITY_BONUS_EG[i] as f64;
        }
        let mut attack_weight: [f64; 8] = [0.; 8];
        for i in 0..8 {
            attack_weight[i] = ATTACK_WEIGHT[i] as f64;
        }
        let mut safety_table: SafetyTable = SafetyTable {
            safety_table: [0.; 100],
        };
        for i in 0..100 {
            safety_table.safety_table[i] = SAFETY_TABLE[i] as f64;
        }
        let mut psqt_pawn: [[[f64; 8]; 8]; 2] = [[[0.; 8]; 8]; 2];
        for i in 0..8 {
            for j in 0..8 {
                psqt_pawn[MG][i][j] = PSQT_PAWN_MG[i][j] as f64;
                psqt_pawn[EG][i][j] = PSQT_PAWN_EG[i][j] as f64;
            }
        }
        let mut psqt_knight: [[[f64; 8]; 8]; 2] = [[[0.; 8]; 8]; 2];
        for i in 0..8 {
            for j in 0..8 {
                psqt_knight[MG][i][j] = PSQT_KNIGHT_MG[i][j] as f64;
                psqt_knight[EG][i][j] = PSQT_KNIGHT_EG[i][j] as f64;
            }
        }
        let mut psqt_bishop: [[[f64; 8]; 8]; 2] = [[[0.; 8]; 8]; 2];
        for i in 0..8 {
            for j in 0..8 {
                psqt_bishop[MG][i][j] = PSQT_BISHOP_MG[i][j] as f64;
                psqt_bishop[EG][i][j] = PSQT_BISHOP_EG[i][j] as f64;
            }
        }
        let mut psqt_king: [[[f64; 8]; 8]; 2] = [[[0.; 8]; 8]; 2];
        for i in 0..8 {
            for j in 0..8 {
                psqt_king[MG][i][j] = PSQT_KING_MG[i][j] as f64;
                psqt_king[EG][i][j] = PSQT_KING_EG[i][j] as f64;
            }
        }
        Parameters {
            tempo_bonus: [TEMPO_BONUS_MG as f64, TEMPO_BONUS_EG as f64],
            shielding_pawn_missing,
            shielding_pawn_onopen_missing,
            pawn_doubled: [PAWN_DOUBLED_VALUE_MG as f64, PAWN_DOUBLED_VALUE_EG as f64],
            pawn_isolated: [PAWN_ISOLATED_VALUE_MG as f64, PAWN_ISOLATED_VALUE_EG as f64],
            pawn_backward: [PAWN_BACKWARD_VALUE_MG as f64, PAWN_BACKWARD_VALUE_EG as f64],
            pawn_supported: [
                PAWN_SUPPORTED_VALUE_MG as f64,
                PAWN_SUPPORTED_VALUE_EG as f64,
            ],
            pawn_attack_center: [PAWN_ATTACK_CENTER_MG as f64, PAWN_ATTACK_CENTER_EG as f64],
            pawn_passed,
            pawn_passed_notblocked,
            knight_supported: [
                KNIGHT_SUPPORTED_BY_PAWN_MG as f64,
                KNIGHT_SUPPORTED_BY_PAWN_EG as f64,
            ],
            knight_outpost_table,
            rook_on_open: [
                ROOK_ON_OPEN_FILE_BONUS_MG as f64,
                ROOK_ON_OPEN_FILE_BONUS_EG as f64,
            ],
            rook_on_seventh: [ROOK_ON_SEVENTH_MG as f64, ROOK_ON_SEVENTH_EG as f64],
            pawn_piece_value: [PAWN_PIECE_VALUE_MG as f64, PAWN_PIECE_VALUE_EG as f64],
            knight_piece_value: [KNIGHT_PIECE_VALUE_MG as f64, KNIGHT_PIECE_VALUE_EG as f64],
            knight_value_with_pawns,
            bishop_piece_value: [BISHOP_PIECE_VALUE_MG as f64, BISHOP_PIECE_VALUE_EG as f64],
            bishop_pair: [BISHOP_PAIR_BONUS_MG as f64, BISHOP_PAIR_BONUS_EG as f64],
            rook_piece_value: [ROOK_PIECE_VALUE_MG as f64, ROOK_PIECE_VALUE_EG as f64],
            queen_piece_value: [QUEEN_PIECE_VALUE_MG as f64, QUEEN_PIECE_VALUE_EG as f64],
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
            pawn_supported: [0.; 2],
            pawn_attack_center: [0.; 2],
            pawn_passed: [[0.; 7]; 2],
            pawn_passed_notblocked: [[0.; 7]; 2],
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
            attack_weight: [0.; 8],
            safety_table: SafetyTable {
                safety_table: [0.; 100],
            },
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
            self.pawn_supported[i] += gradient.pawn_supported[i] / norm;
            self.pawn_attack_center[i] += gradient.pawn_attack_center[i] / norm;
            self.knight_supported[i] += gradient.knight_supported[i] / norm;
            self.rook_on_open[i] += gradient.rook_on_open[i] / norm;
            self.rook_on_seventh[i] += gradient.rook_on_seventh[i] / norm;
            self.pawn_piece_value[i] += gradient.pawn_piece_value[i] / norm;
            self.knight_piece_value[i] += gradient.knight_piece_value[i] / norm;
            self.bishop_piece_value[i] += gradient.bishop_piece_value[i] / norm;
            self.bishop_pair[i] += gradient.bishop_pair[i] / norm;
            self.rook_piece_value[i] += gradient.rook_piece_value[i] / norm;
            self.queen_piece_value[i] += gradient.queen_piece_value[i] / norm;
        }
        for i in 0..2 {
            apply_gradient_arr(&mut self.pawn_passed[i], &gradient.pawn_passed[i], norm);
            apply_gradient_arr(
                &mut self.pawn_passed_notblocked[i],
                &gradient.pawn_passed_notblocked[i],
                norm,
            );
        }
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
        }
        apply_gradient_arr(&mut self.attack_weight, &gradient.attack_weight, norm);
        apply_gradient_arr(
            &mut self.safety_table.safety_table,
            &gradient.safety_table.safety_table,
            norm,
        );
    }
}
