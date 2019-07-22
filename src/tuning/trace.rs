use super::parameters::Parameters;
use crate::board_representation::game_state::{BLACK, WHITE};
use crate::evaluation::{EG, MG};
pub struct Trace {
    pub tempo_bonus: [isize; 2],
    pub shielding_pawn_missing: [[isize; 4]; 2],
    pub shielding_pawn_onopen_missing: [[isize; 4]; 2],
    pub pawn_doubled: [isize; 2],
    pub pawn_isolated: [isize; 2],
    pub pawn_backward: [isize; 2],
    pub pawn_supported: [isize; 2],
    pub pawn_attack_center: [isize; 2],
    pub pawn_passed: [[isize; 7]; 2],
    pub pawn_passed_notblocked: [[isize; 7]; 2],
    pub knight_supported: [isize; 2],
    pub knight_outpost_table: [[[isize; 8]; 8]; 2],
    pub rook_on_open: [isize; 2],
    pub rook_on_seventh: [isize; 2],
    pub pawns: [isize; 2],
    pub knights: [isize; 2],
    pub knight_value_with_pawns: usize,
    pub bishops: [isize; 2],
    pub bishop_bonus: [isize; 2],
    pub rooks: [isize; 2],
    pub queens: [isize; 2],
    pub diagonally_adjacent_squares_withpawns: [[isize; 5]; 2],
    pub knight_mobility: [[isize; 9]; 2],
    pub bishop_mobility: [[isize; 14]; 2],
    pub rook_mobility: [[isize; 15]; 2],
    pub queen_mobility: [[isize; 28]; 2],
    pub attackers: [usize; 2],
    pub attacker_value: [usize; 2],
    pub psqt_pawn: [[[isize; 8]; 8]; 2],
    pub psqt_knight: [[[isize; 8]; 8]; 2],
    pub psqt_bishop: [[[isize; 8]; 8]; 2],
    pub psqt_king: [[[isize; 8]; 8]; 2],
    pub phase: f64,
}
impl Trace {
    pub fn evaluate(&self, params: &Parameters) -> f64 {
        let mut res = (0., 0.);
        res.0 +=
            (self.tempo_bonus[WHITE] - self.tempo_bonus[BLACK]) as f64 * params.tempo_bonus[MG];
        res.1 +=
            (self.tempo_bonus[WHITE] - self.tempo_bonus[BLACK]) as f64 * params.tempo_bonus[EG];
        for i in 0..4 {
            res.0 += (self.shielding_pawn_missing[WHITE][i] - self.shielding_pawn_missing[BLACK][i])
                as f64
                * params.shielding_pawn_missing[MG][i];
            res.1 += (self.shielding_pawn_missing[WHITE][i] - self.shielding_pawn_missing[BLACK][i])
                as f64
                * params.shielding_pawn_missing[EG][i];
            res.0 += (self.shielding_pawn_onopen_missing[WHITE][i]
                - self.shielding_pawn_onopen_missing[BLACK][i]) as f64
                * params.shielding_pawn_onopen_missing[MG][i];
            res.1 += (self.shielding_pawn_onopen_missing[WHITE][i]
                - self.shielding_pawn_onopen_missing[BLACK][i]) as f64
                * params.shielding_pawn_onopen_missing[EG][i];
        }
        res.0 +=
            (self.pawn_doubled[WHITE] - self.pawn_doubled[BLACK]) as f64 * params.pawn_doubled[MG];
        res.1 +=
            (self.pawn_doubled[WHITE] - self.pawn_doubled[BLACK]) as f64 * params.pawn_doubled[EG];
        res.0 += (self.pawn_isolated[WHITE] - self.pawn_isolated[BLACK]) as f64
            * params.pawn_isolated[MG];
        res.1 += (self.pawn_isolated[WHITE] - self.pawn_isolated[BLACK]) as f64
            * params.pawn_isolated[EG];
        res.0 += (self.pawn_backward[WHITE] - self.pawn_backward[BLACK]) as f64
            * params.pawn_backward[MG];
        res.1 += (self.pawn_backward[WHITE] - self.pawn_backward[BLACK]) as f64
            * params.pawn_backward[EG];
        res.0 += (self.pawn_supported[WHITE] - self.pawn_supported[BLACK]) as f64
            * params.pawn_supported[MG];
        res.1 += (self.pawn_supported[WHITE] - self.pawn_supported[BLACK]) as f64
            * params.pawn_supported[EG];
        res.0 += (self.pawn_attack_center[WHITE] - self.pawn_attack_center[BLACK]) as f64
            * params.pawn_attack_center[MG];
        res.1 += (self.pawn_attack_center[WHITE] - self.pawn_attack_center[BLACK]) as f64
            * params.pawn_attack_center[EG];

        for i in 0..7 {
            res.0 += (self.pawn_passed[WHITE][i] - self.pawn_passed[BLACK][i]) as f64
                * params.pawn_passed[MG][i];
            res.1 += (self.pawn_passed[WHITE][i] - self.pawn_passed[BLACK][i]) as f64
                * params.pawn_passed[EG][i];
            res.0 += (self.pawn_passed_notblocked[WHITE][i] - self.pawn_passed_notblocked[BLACK][i])
                as f64
                * params.pawn_passed_notblocked[MG][i];
            res.1 += (self.pawn_passed_notblocked[WHITE][i] - self.pawn_passed_notblocked[BLACK][i])
                as f64
                * params.pawn_passed_notblocked[EG][i];
        }
        res.0 += (self.knight_supported[WHITE] - self.knight_supported[BLACK]) as f64
            * params.knight_supported[MG];
        res.1 += (self.knight_supported[WHITE] - self.knight_supported[BLACK]) as f64
            * params.knight_supported[EG];
        for i in 0..8 {
            for j in 0..8 {
                res.0 += (self.knight_outpost_table[WHITE][i][j]
                    - self.knight_outpost_table[BLACK][i][j]) as f64
                    * params.knight_outpost_table[MG][i][j];
                res.1 += (self.knight_outpost_table[WHITE][i][j]
                    - self.knight_outpost_table[BLACK][i][j]) as f64
                    * params.knight_outpost_table[EG][i][j];
                res.0 += (self.psqt_pawn[WHITE][i][j] - self.psqt_pawn[BLACK][i][j]) as f64
                    * params.psqt_pawn[MG][i][j];
                res.1 += (self.psqt_pawn[WHITE][i][j] - self.psqt_pawn[BLACK][i][j]) as f64
                    * params.psqt_pawn[EG][i][j];
                res.0 += (self.psqt_knight[WHITE][i][j] - self.psqt_knight[BLACK][i][j]) as f64
                    * params.psqt_knight[MG][i][j];
                res.1 += (self.psqt_knight[WHITE][i][j] - self.psqt_knight[BLACK][i][j]) as f64
                    * params.psqt_knight[EG][i][j];
                res.0 += (self.psqt_bishop[WHITE][i][j] - self.psqt_bishop[BLACK][i][j]) as f64
                    * params.psqt_bishop[MG][i][j];
                res.1 += (self.psqt_bishop[WHITE][i][j] - self.psqt_bishop[BLACK][i][j]) as f64
                    * params.psqt_bishop[EG][i][j];
                res.0 += (self.psqt_king[WHITE][i][j] - self.psqt_king[BLACK][i][j]) as f64
                    * params.psqt_king[MG][i][j];
                res.1 += (self.psqt_king[WHITE][i][j] - self.psqt_king[BLACK][i][j]) as f64
                    * params.psqt_king[EG][i][j];
            }
        }
        res.0 +=
            (self.rook_on_open[WHITE] - self.rook_on_open[BLACK]) as f64 * params.rook_on_open[MG];
        res.1 +=
            (self.rook_on_open[WHITE] - self.rook_on_open[BLACK]) as f64 * params.rook_on_open[EG];
        res.0 += (self.rook_on_seventh[WHITE] - self.rook_on_seventh[BLACK]) as f64
            * params.rook_on_seventh[MG];
        res.1 += (self.rook_on_seventh[WHITE] - self.rook_on_seventh[BLACK]) as f64
            * params.rook_on_seventh[EG];
        res.0 += (self.pawns[WHITE] - self.pawns[BLACK]) as f64 * params.pawn_piece_value[MG];
        res.1 += (self.pawns[WHITE] - self.pawns[BLACK]) as f64 * params.pawn_piece_value[EG];
        res.0 += (self.knights[WHITE] - self.knights[BLACK]) as f64
            * (params.knight_piece_value[MG]
                + params.knight_value_with_pawns[self.knight_value_with_pawns]);
        res.1 += (self.knights[WHITE] - self.knights[BLACK]) as f64
            * (params.knight_piece_value[EG]
                + params.knight_value_with_pawns[self.knight_value_with_pawns]);
        res.0 += (self.bishops[WHITE] - self.bishops[BLACK]) as f64 * params.bishop_piece_value[MG];
        res.1 += (self.bishops[WHITE] - self.bishops[BLACK]) as f64 * params.bishop_piece_value[EG];
        res.0 +=
            (self.bishop_bonus[WHITE] - self.bishop_bonus[BLACK]) as f64 * params.bishop_pair[MG];
        res.1 +=
            (self.bishop_bonus[WHITE] - self.bishop_bonus[BLACK]) as f64 * params.bishop_pair[EG];
        res.0 += (self.rooks[WHITE] - self.rooks[BLACK]) as f64 * params.rook_piece_value[MG];
        res.1 += (self.rooks[WHITE] - self.rooks[BLACK]) as f64 * params.rook_piece_value[EG];
        res.0 += (self.queens[WHITE] - self.queens[BLACK]) as f64 * params.queen_piece_value[MG];
        res.1 += (self.queens[WHITE] - self.queens[BLACK]) as f64 * params.queen_piece_value[EG];
        for i in 0..5 {
            res.0 += (self.diagonally_adjacent_squares_withpawns[WHITE][i]
                - self.diagonally_adjacent_squares_withpawns[BLACK][i]) as f64
                * params.diagonally_adjacent_squares_withpawns[MG][i];
            res.1 += (self.diagonally_adjacent_squares_withpawns[WHITE][i]
                - self.diagonally_adjacent_squares_withpawns[BLACK][i]) as f64
                * params.diagonally_adjacent_squares_withpawns[EG][i];
        }
        for i in 0..9 {
            res.0 += (self.knight_mobility[WHITE][i] - self.knight_mobility[BLACK][i]) as f64
                * params.knight_mobility[MG][i];
            res.1 += (self.knight_mobility[WHITE][i] - self.knight_mobility[BLACK][i]) as f64
                * params.knight_mobility[EG][i];
        }
        for i in 0..14 {
            res.0 += (self.bishop_mobility[WHITE][i] - self.bishop_mobility[BLACK][i]) as f64
                * params.bishop_mobility[MG][i];
            res.1 += (self.bishop_mobility[WHITE][i] - self.bishop_mobility[BLACK][i]) as f64
                * params.bishop_mobility[EG][i];
        }
        for i in 0..15 {
            res.0 += (self.rook_mobility[WHITE][i] - self.rook_mobility[BLACK][i]) as f64
                * params.rook_mobility[MG][i];
            res.1 += (self.rook_mobility[WHITE][i] - self.rook_mobility[BLACK][i]) as f64
                * params.rook_mobility[EG][i];
        }
        for i in 0..28 {
            res.0 += (self.queen_mobility[WHITE][i] - self.queen_mobility[BLACK][i]) as f64
                * params.queen_mobility[MG][i];
            res.1 += (self.queen_mobility[WHITE][i] - self.queen_mobility[BLACK][i]) as f64
                * params.queen_mobility[EG][i];
        }
        res.0 += (params.attack_weight[self.attackers[WHITE]]
            * params.safety_table.safety_table[self.attacker_value[WHITE]]
            - params.attack_weight[self.attackers[BLACK]]
                * params.safety_table.safety_table[self.attacker_value[BLACK]])
            / 100.0;
        res.1 += (params.attack_weight[self.attackers[WHITE]]
            * params.safety_table.safety_table[self.attacker_value[WHITE]]
            - params.attack_weight[self.attackers[BLACK]]
                * params.safety_table.safety_table[self.attacker_value[BLACK]])
            / 100.0;
        (res.0 * self.phase + res.1 * (128.0 - self.phase)) / 128.0
    }
    pub fn default() -> Self {
        Trace {
            tempo_bonus: [0; 2],
            shielding_pawn_missing: [[0; 4]; 2],
            shielding_pawn_onopen_missing: [[0; 4]; 2],
            pawn_doubled: [0; 2],
            pawn_isolated: [0; 2],
            pawn_backward: [0; 2],
            pawn_supported: [0; 2],
            pawn_attack_center: [0; 2],
            pawn_passed: [[0; 7]; 2],
            pawn_passed_notblocked: [[0; 7]; 2],
            knight_supported: [0; 2],
            knight_outpost_table: [[[0; 8]; 8]; 2],
            rook_on_open: [0; 2],
            rook_on_seventh: [0; 2],
            pawns: [0; 2],
            knights: [0; 2],
            knight_value_with_pawns: 0,
            bishops: [0; 2],
            bishop_bonus: [0; 2],
            rooks: [0; 2],
            queens: [0; 2],
            diagonally_adjacent_squares_withpawns: [[0; 5]; 2],
            knight_mobility: [[0; 9]; 2],
            bishop_mobility: [[0; 14]; 2],
            rook_mobility: [[0; 15]; 2],
            queen_mobility: [[0; 28]; 2],
            attackers: [0; 2],
            attacker_value: [0; 2],
            psqt_pawn: [[[0; 8]; 8]; 2],
            psqt_knight: [[[0; 8]; 8]; 2],
            psqt_bishop: [[[0; 8]; 8]; 2],
            psqt_king: [[[0; 8]; 8]; 2],
            phase: 0.,
        }
    }
}
