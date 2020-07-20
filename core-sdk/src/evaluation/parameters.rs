use crate::board_representation::game_state::PIECE_TYPES;
use crate::evaluation::psqt_evaluation::BLACK_INDEX;
use crate::evaluation::{EG, MG};
use std::fmt::{Debug, Display, Formatter, Result};
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
    pub pawn_passed_kingdistance: [[f64; 7]; 2],
    pub pawn_passed_enemykingdistance: [[f64; 7]; 2],
    pub pawn_passed_subdistance: [[f64; 13]; 2],
    pub rook_behind_support_passer: [f64; 2],
    pub rook_behind_enemy_passer: [f64; 2],
    pub pawn_passed_weak: [f64; 2],
    pub knight_supported: [f64; 2],
    pub knight_outpost_table: [[[f64; 8]; 8]; 2],
    pub bishop_xray_king: [f64; 2],
    pub rook_xray_king: [f64; 2],
    pub queen_xray_king: [f64; 2],
    pub rook_on_open: [f64; 2],
    pub rook_on_semi_open: [f64; 2],
    pub queen_on_open: [f64; 2],
    pub queen_on_semi_open: [f64; 2],
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
    pub knight_check_value: [f64; 2],
    pub bishop_check_value: [f64; 2],
    pub rook_check_value: [f64; 2],
    pub queen_check_value: [f64; 2],
    pub psqt: [[[[f64; 8]; 8]; 2]; 6],
}
pub fn vectorized_psqt_to_string(psqt: &[[[[f64; 8]; 8]; 2]; 6]) -> String {
    let mut res_str = String::new();
    res_str.push_str("[");
    for pt in PIECE_TYPES.iter() {
        res_str.push_str("[");
        res_str.push_str(&psqts_to_evalstring(
            &psqt[*pt as usize][MG],
            &psqt[*pt as usize][EG],
        ));
        res_str.push_str(",");
        let mut color_swapped = [[[0.; 8]; 8]; 2];
        for ph in 0..2 {
            for i in 0..8 {
                for j in 0..8 {
                    let b_index = BLACK_INDEX[i * 8 + j];
                    color_swapped[ph][i][j] =
                        psqt[*pt as usize][ph][b_index / 8][b_index % 8] * -1.;
                }
            }
        }
        res_str.push_str(&psqts_to_evalstring(&color_swapped[MG], &color_swapped[EG]));
        res_str.push_str("],");
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
pub fn arrays_to_evalstring(array1: &[f64], array2: &[f64]) -> String {
    let mut res_str = String::new();
    res_str.push_str("[");
    for (index, x) in array1.iter().enumerate() {
        res_str.push_str(&format!(
            "EvaluationScore({}, {}), ",
            x.round() as isize,
            array2[index].round() as isize
        ));
    }
    res_str.push_str("]");
    res_str
}
pub fn psqts_to_evalstring(psqt1: &[[f64; 8]; 8], psqt2: &[[f64; 8]; 8]) -> String {
    let mut res_str = String::new();
    res_str.push_str("[");
    for (index, x) in psqt1.iter().enumerate() {
        res_str.push_str(&format!("{}, ", arrays_to_evalstring(x, &psqt2[index])));
    }
    res_str.push_str("]");
    res_str
}
impl Display for Parameters {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str = String::new();
        res_str.push_str("use super::EvaluationScore;");
        res_str.push_str(&format!(
            "pub const TEMPO_BONUS: EvaluationScore = EvaluationScore({}, {});\n",
            self.tempo_bonus[MG].round() as isize,
            self.tempo_bonus[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const SHIELDING_PAWN_MISSING: [EvaluationScore;4] = {};\n",
            arrays_to_evalstring(
                &self.shielding_pawn_missing[MG],
                &self.shielding_pawn_missing[EG]
            )
        ));
        res_str.push_str(&format!(
            "pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE: [EvaluationScore;4] = {};\n",
            arrays_to_evalstring(
                &self.shielding_pawn_onopen_missing[MG],
                &self.shielding_pawn_onopen_missing[EG]
            )
        ));
        res_str.push_str(&format!(
            "pub const PAWN_DOUBLED_VALUE: EvaluationScore = EvaluationScore({}, {});\n",
            self.pawn_doubled[MG].round() as isize,
            self.pawn_doubled[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_ISOLATED_VALUE: EvaluationScore = EvaluationScore({}, {});\n",
            self.pawn_isolated[MG].round() as isize,
            self.pawn_isolated[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_BACKWARD_VALUE: EvaluationScore = EvaluationScore({}, {});\n",
            self.pawn_backward[MG].round() as isize,
            self.pawn_backward[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_SUPPORTED_VALUE: [[EvaluationScore;8];8] = {};\n",
            psqts_to_evalstring(&self.pawn_supported[MG], &self.pawn_supported[EG])
        ));
        res_str.push_str(&format!(
            "pub const PAWN_ATTACK_CENTER: EvaluationScore = EvaluationScore({}, {});\n",
            self.pawn_attack_center[MG].round() as isize,
            self.pawn_attack_center[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_MOBILITY: EvaluationScore = EvaluationScore({}, {});\n",
            self.pawn_mobility[MG].round() as isize,
            self.pawn_mobility[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_PASSED_VALUES: [EvaluationScore;7] = {};\n",
            arrays_to_evalstring(&self.pawn_passed[MG], &self.pawn_passed[EG])
        ));
        res_str.push_str(&format!(
            "pub const PAWN_PASSED_NOT_BLOCKED_VALUES: [EvaluationScore;7] = {};\n",
            arrays_to_evalstring(
                &self.pawn_passed_notblocked[MG],
                &self.pawn_passed_notblocked[EG]
            )
        ));
        res_str.push_str(&format!(
            "pub const PASSED_KING_DISTANCE: [EvaluationScore;7] = {};\n",
            arrays_to_evalstring(
                &self.pawn_passed_kingdistance[MG],
                &self.pawn_passed_kingdistance[EG]
            )
        ));
        res_str.push_str(&format!(
            "pub const PASSED_ENEMY_KING_DISTANCE: [EvaluationScore;7] = {};\n",
            arrays_to_evalstring(
                &self.pawn_passed_enemykingdistance[MG],
                &self.pawn_passed_enemykingdistance[EG]
            )
        ));
        res_str.push_str(&format!(
            "pub const PASSED_SUBTRACT_DISTANCE: [EvaluationScore;13] = {};\n",
            arrays_to_evalstring(
                &self.pawn_passed_subdistance[MG],
                &self.pawn_passed_subdistance[EG]
            )
        ));
        res_str.push_str(&format!(
            "pub const ROOK_BEHIND_SUPPORT_PASSER: EvaluationScore = EvaluationScore({}, {});\n",
            self.rook_behind_support_passer[MG].round() as isize,
            self.rook_behind_support_passer[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_BEHIND_ENEMY_PASSER: EvaluationScore = EvaluationScore({}, {});\n",
            self.rook_behind_enemy_passer[MG].round() as isize,
            self.rook_behind_enemy_passer[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_PASSED_WEAK: EvaluationScore = EvaluationScore({}, {});\n",
            self.pawn_passed_weak[MG].round() as isize,
            self.pawn_passed_weak[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_SUPPORTED_BY_PAWN: EvaluationScore = EvaluationScore({}, {});\n",
            self.knight_supported[MG].round() as isize,
            self.knight_supported[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_OUTPOST_TABLE: [[EvaluationScore;8];8] = {};\n",
            psqts_to_evalstring(
                &self.knight_outpost_table[MG],
                &self.knight_outpost_table[EG]
            )
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_XRAY_KING: EvaluationScore = EvaluationScore({}, {});\n",
            self.bishop_xray_king[MG].round() as isize,
            self.bishop_xray_king[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_XRAY_KING: EvaluationScore = EvaluationScore({}, {});\n",
            self.rook_xray_king[MG].round() as isize,
            self.rook_xray_king[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_XRAY_KING: EvaluationScore = EvaluationScore({}, {});\n",
            self.queen_xray_king[MG].round() as isize,
            self.queen_xray_king[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_ON_OPEN_FILE_BONUS: EvaluationScore = EvaluationScore({}, {});\n",
            self.rook_on_open[MG].round() as isize,
            self.rook_on_open[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_ON_SEMI_OPEN_FILE_BONUS: EvaluationScore = EvaluationScore({}, {});\n",
            self.rook_on_semi_open[MG].round() as isize,
            self.rook_on_semi_open[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_ON_OPEN_FILE_BONUS: EvaluationScore = EvaluationScore({}, {});\n",
            self.queen_on_open[MG].round() as isize,
            self.queen_on_open[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_ON_SEMI_OPEN_FILE_BONUS: EvaluationScore = EvaluationScore({}, {});\n",
            self.queen_on_semi_open[MG].round() as isize,
            self.queen_on_semi_open[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_ON_SEVENTH: EvaluationScore = EvaluationScore({}, {});\n",
            self.rook_on_seventh[MG].round() as isize,
            self.rook_on_seventh[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PAWN_PIECE_VALUE: EvaluationScore = EvaluationScore({}, {});\n",
            self.pawn_piece_value[MG].round() as isize,
            self.pawn_piece_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_PIECE_VALUE: EvaluationScore = EvaluationScore({}, {});\n",
            self.knight_piece_value[MG].round() as isize,
            self.knight_piece_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_VALUE_WITH_PAWNS: [i16;17] = {};\n",
            array_to_string(&self.knight_value_with_pawns)
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_PIECE_VALUE: EvaluationScore = EvaluationScore({}, {});\n",
            self.bishop_piece_value[MG].round() as isize,
            self.bishop_piece_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_PAIR_BONUS: EvaluationScore = EvaluationScore({}, {});\n",
            self.bishop_pair[MG].round() as isize,
            self.bishop_pair[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_PIECE_VALUE: EvaluationScore = EvaluationScore({}, {});\n",
            self.rook_piece_value[MG].round() as isize,
            self.rook_piece_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_PIECE_VALUE: EvaluationScore = EvaluationScore({}, {});\n",
            self.queen_piece_value[MG].round() as isize,
            self.queen_piece_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS: [EvaluationScore;5] = {};\n",
            arrays_to_evalstring(
                &self.diagonally_adjacent_squares_withpawns[MG],
                &self.diagonally_adjacent_squares_withpawns[EG]
            )
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_MOBILITY_BONUS: [EvaluationScore;9] = {};\n",
            arrays_to_evalstring(&self.knight_mobility[MG], &self.knight_mobility[EG])
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_MOBILITY_BONUS: [EvaluationScore;14] = {};\n",
            arrays_to_evalstring(&self.bishop_mobility[MG], &self.bishop_mobility[EG])
        ));
        res_str.push_str(&format!(
            "pub const ROOK_MOBILITY_BONUS: [EvaluationScore;15] = {};\n",
            arrays_to_evalstring(&self.rook_mobility[MG], &self.rook_mobility[EG])
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_MOBILITY_BONUS: [EvaluationScore;28] = {};\n",
            arrays_to_evalstring(&self.queen_mobility[MG], &self.queen_mobility[EG])
        ));
        res_str.push_str(&format!(
            "pub const ATTACK_WEIGHT: [EvaluationScore;8] = {};\n",
            arrays_to_evalstring(&self.attack_weight[MG], &self.attack_weight[EG])
        ));
        res_str.push_str(&format!(
            "pub const SAFETY_TABLE: [EvaluationScore;100] = {};\n",
            arrays_to_evalstring(
                &self.safety_table[MG].safety_table,
                &self.safety_table[EG].safety_table
            )
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_ATTACK_WORTH: EvaluationScore = EvaluationScore({}, {});\n",
            self.knight_attack_value[MG].round() as isize,
            self.knight_attack_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_ATTACK_WORTH: EvaluationScore = EvaluationScore({}, {});\n",
            self.bishop_attack_value[MG].round() as isize,
            self.bishop_attack_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_ATTACK_WORTH: EvaluationScore = EvaluationScore({}, {});\n",
            self.rook_attack_value[MG].round() as isize,
            self.rook_attack_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_ATTACK_WORTH: EvaluationScore = EvaluationScore({}, {});\n",
            self.queen_attack_value[MG].round() as isize,
            self.queen_attack_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const KNIGHT_SAFE_CHECK: EvaluationScore = EvaluationScore({}, {});\n",
            self.knight_check_value[MG].round() as isize,
            self.knight_check_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const BISHOP_SAFE_CHECK: EvaluationScore = EvaluationScore({}, {});\n",
            self.bishop_check_value[MG].round() as isize,
            self.bishop_check_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const ROOK_SAFE_CHECK: EvaluationScore = EvaluationScore({}, {});\n",
            self.rook_check_value[MG].round() as isize,
            self.rook_check_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const QUEEN_SAFE_CHECK: EvaluationScore = EvaluationScore({}, {});\n",
            self.queen_check_value[MG].round() as isize,
            self.queen_check_value[EG].round() as isize
        ));
        res_str.push_str(&format!(
            "pub const PSQT: [[[[EvaluationScore;8];8];2];6] = {};\n",
            vectorized_psqt_to_string(&self.psqt)
        ));
        write!(formatter, "{}", res_str)
    }
}
impl Parameters {
    pub fn write_to_file(&self, file: &str) {
        fs::write(file, &format!("{}", self)).expect("Unable to write file");
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
            pawn_passed_kingdistance: [[0.; 7]; 2],
            pawn_passed_enemykingdistance: [[0.; 7]; 2],
            pawn_passed_subdistance: [[0.; 13]; 2],
            rook_behind_support_passer: [0.; 2],
            rook_behind_enemy_passer: [0.; 2],
            pawn_passed_weak: [0.; 2],
            knight_supported: [0.; 2],
            knight_outpost_table: [[[0.; 8]; 8]; 2],
            bishop_xray_king: [0.; 2],
            rook_xray_king: [0.; 2],
            queen_xray_king: [0.; 2],
            rook_on_open: [0.; 2],
            rook_on_semi_open: [0.; 2],
            queen_on_open: [0.; 2],
            queen_on_semi_open: [0.; 2],
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
            knight_check_value: [0.; 2],
            bishop_check_value: [0.; 2],
            rook_check_value: [0.; 2],
            queen_check_value: [0.; 2],
            psqt: [[[[0.; 8]; 8]; 2]; 6],
        }
    }
    pub fn calculate_norm(&self) -> f64 {
        //Norm gradient
        let mut norm: f64 = 0.;
        for i in 0..2 {
            norm += self.tempo_bonus[i].powf(2.);
            for j in 0..4 {
                norm += self.shielding_pawn_missing[i][j].powf(2.);
                norm += self.shielding_pawn_onopen_missing[i][j].powf(2.);
            }
            norm += self.pawn_doubled[i].powf(2.);
            norm += self.pawn_isolated[i].powf(2.);
            norm += self.pawn_backward[i].powf(2.);
            norm += self.pawn_attack_center[i].powf(2.);
            norm += self.pawn_mobility[i].powf(2.);
            for j in 0..7 {
                norm += self.pawn_passed[i][j].powf(2.);
                norm += self.pawn_passed_notblocked[i][j].powf(2.);
                norm += self.pawn_passed_kingdistance[i][j].powf(2.);
                norm += self.pawn_passed_enemykingdistance[i][j].powf(2.);
            }
            for j in 0..13 {
                norm += self.pawn_passed_subdistance[i][j].powf(2.);
            }
            norm += self.rook_behind_support_passer[i].powf(2.);
            norm += self.rook_behind_enemy_passer[i].powf(2.);
            norm += self.pawn_passed_weak[i].powf(2.);
            norm += self.knight_supported[i].powf(2.);
            for j in 0..8 {
                for k in 0..8 {
                    norm += self.pawn_supported[i][j][k].powf(2.);
                    norm += self.knight_outpost_table[i][j][k].powf(2.);
                    for pt in PIECE_TYPES.iter() {
                        norm += self.psqt[*pt as usize][i][j][k].powf(2.);
                    }
                }
            }
            norm += self.bishop_xray_king[i].powf(2.);
            norm += self.rook_xray_king[i].powf(2.);
            norm += self.queen_xray_king[i].powf(2.);
            norm += self.rook_on_open[i].powf(2.);
            norm += self.rook_on_semi_open[i].powf(2.);
            norm += self.queen_on_open[i].powf(2.);
            norm += self.queen_on_semi_open[i].powf(2.);
            norm += self.rook_on_seventh[i].powf(2.);
            norm += self.pawn_piece_value[i].powf(2.);
            norm += self.knight_piece_value[i].powf(2.);
            norm += self.bishop_piece_value[i].powf(2.);
            norm += self.bishop_pair[i].powf(2.);
            norm += self.rook_piece_value[i].powf(2.);
            norm += self.queen_piece_value[i].powf(2.);
            for j in 0..5 {
                norm += self.diagonally_adjacent_squares_withpawns[i][j].powf(2.);
            }
            for j in 0..9 {
                norm += self.knight_mobility[i][j].powf(2.);
            }
            for j in 0..14 {
                norm += self.bishop_mobility[i][j].powf(2.);
            }
            for j in 0..15 {
                norm += self.rook_mobility[i][j].powf(2.);
            }
            for j in 0..28 {
                norm += self.queen_mobility[i][j].powf(2.);
            }
        }
        for i in 0..17 {
            norm += self.knight_value_with_pawns[i].powf(2.);
        }
        for i in 0..2 {
            norm += self.knight_attack_value[i].powf(2.);
            norm += self.bishop_attack_value[i].powf(2.);
            norm += self.rook_attack_value[i].powf(2.);
            norm += self.queen_attack_value[i].powf(2.);
            norm += self.knight_check_value[i].powf(2.);
            norm += self.bishop_check_value[i].powf(2.);
            norm += self.rook_check_value[i].powf(2.);
            norm += self.queen_check_value[i].powf(2.);
            for j in 0..8 {
                norm += self.attack_weight[i][j].powf(2.);
            }
        }
        for i in 0..2 {
            for j in 0..100 {
                norm += self.safety_table[i].safety_table[j].powf(2.);
            }
        }
        norm = norm.sqrt();
        norm
    }

    pub fn apply_gradient(&mut self, gradient: &Parameters, lr: f64) {
        let norm = gradient.calculate_norm() / lr;
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
            self.bishop_xray_king[i] += gradient.bishop_xray_king[i] / norm;
            self.rook_xray_king[i] += gradient.rook_xray_king[i] / norm;
            self.queen_xray_king[i] += gradient.queen_xray_king[i] / norm;
            self.rook_on_open[i] += gradient.rook_on_open[i] / norm;
            self.rook_on_semi_open[i] += gradient.rook_on_semi_open[i] / norm;
            self.queen_on_open[i] += gradient.queen_on_open[i] / norm;
            self.queen_on_semi_open[i] += gradient.queen_on_semi_open[i] / norm;
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
            self.knight_check_value[i] += gradient.knight_check_value[i] / norm;
            self.bishop_check_value[i] += gradient.bishop_check_value[i] / norm;
            self.rook_check_value[i] += gradient.rook_check_value[i] / norm;
            self.queen_check_value[i] += gradient.queen_check_value[i] / norm;
        }
        for i in 0..2 {
            apply_gradient_arr(&mut self.pawn_passed[i], &gradient.pawn_passed[i], norm);
            apply_gradient_arr(
                &mut self.pawn_passed_notblocked[i],
                &gradient.pawn_passed_notblocked[i],
                norm,
            );
            apply_gradient_arr(
                &mut self.pawn_passed_kingdistance[i],
                &gradient.pawn_passed_kingdistance[i],
                norm,
            );
            apply_gradient_arr(
                &mut self.pawn_passed_enemykingdistance[i],
                &gradient.pawn_passed_enemykingdistance[i],
                norm,
            );
            apply_gradient_arr(
                &mut self.pawn_passed_subdistance[i],
                &gradient.pawn_passed_subdistance[i],
                norm,
            );
        }
        apply_gradient_psqt(&mut self.pawn_supported, &gradient.pawn_supported, norm);
        apply_gradient_psqt(
            &mut self.knight_outpost_table,
            &gradient.knight_outpost_table,
            norm,
        );
        for pt in PIECE_TYPES.iter() {
            apply_gradient_psqt(
                &mut self.psqt[*pt as usize],
                &gradient.psqt[*pt as usize],
                norm,
            );
        }

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
