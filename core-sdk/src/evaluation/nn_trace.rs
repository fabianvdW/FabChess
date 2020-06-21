pub const FEATURES: usize = 676;
#[cfg(feature = "nn-eval")]
use ndarray::Array1;
pub struct NNTrace {
    #[cfg(feature = "nn-eval")]
    pub trace: Array1<f32>,
}
impl NNTrace {
    pub fn new() -> Self {
        NNTrace {
            #[cfg(feature = "nn-eval")]
            trace: Array1::zeros(FEATURES),
        }
    }
}
pub mod trace_pos {
    pub const TEMPO_BONUS: usize = 0;
    pub const TEMPO_BONUS_SIZE: usize = 1;

    pub const SHIELDING_PAWN_MISSING: usize = TEMPO_BONUS + TEMPO_BONUS_SIZE;
    pub const SHIELDING_PAWN_MISSING_SIZE: usize = 4;

    pub const SHIELDING_PAWN_ONOPEN_MISSING: usize =
        SHIELDING_PAWN_MISSING + SHIELDING_PAWN_MISSING_SIZE;
    pub const SHIELDING_PAWN_ONOPEN_MISSING_SIZE: usize = 4;

    pub const PAWN_DOUBLED: usize =
        SHIELDING_PAWN_ONOPEN_MISSING + SHIELDING_PAWN_ONOPEN_MISSING_SIZE;
    pub const PAWN_DOUBLED_SIZE: usize = 1;

    pub const PAWN_ISOLATED: usize = PAWN_DOUBLED + PAWN_DOUBLED_SIZE;
    pub const PAWN_ISOLATED_SIZE: usize = 1;

    pub const PAWN_BACKWARD: usize = PAWN_ISOLATED + PAWN_ISOLATED_SIZE;
    pub const PAWN_BACKWARD_SIZE: usize = 1;

    pub const PAWN_SUPPORTED: usize = PAWN_BACKWARD + PAWN_BACKWARD_SIZE;
    pub const PAWN_SUPPORTED_SIZE: usize = 64;

    pub const PAWN_ATTACK_CENTER: usize = PAWN_SUPPORTED + PAWN_SUPPORTED_SIZE;
    pub const PAWN_ATTACK_CENTER_SIZE: usize = 1;

    pub const PAWN_MOBILITY: usize = PAWN_ATTACK_CENTER + PAWN_ATTACK_CENTER_SIZE;
    pub const PAWN_MOBILITY_SIZE: usize = 1;

    pub const PAWN_PASSED: usize = PAWN_MOBILITY + PAWN_MOBILITY_SIZE;
    pub const PAWN_PASSED_SIZE: usize = 7;

    pub const PAWN_PASSED_NOTBLOCKED: usize = PAWN_PASSED + PAWN_PASSED_SIZE;
    pub const PAWN_PASSED_NOTBLOCKED_SIZE: usize = 7;

    pub const PAWN_PASSED_KINGDISTANCE: usize =
        PAWN_PASSED_NOTBLOCKED + PAWN_PASSED_NOTBLOCKED_SIZE;
    pub const PAWN_PASSED_KINGDISTANCE_SIZE: usize = 7;

    pub const PAWN_PASSED_ENEMYKINGDISTANCE: usize =
        PAWN_PASSED_KINGDISTANCE + PAWN_PASSED_KINGDISTANCE_SIZE;
    pub const PAWN_PASSED_ENEMYKINGDISTANCE_SIZE: usize = 7;

    pub const PAWN_PASSED_SUBKINGDISTANCE: usize =
        PAWN_PASSED_ENEMYKINGDISTANCE + PAWN_PASSED_ENEMYKINGDISTANCE_SIZE;
    pub const PAWN_PASSED_SUBKINGDISTANCE_SIZE: usize = 13;

    pub const ROOK_BEHIND_SUPPORT_PASSER: usize =
        PAWN_PASSED_SUBKINGDISTANCE + PAWN_PASSED_SUBKINGDISTANCE_SIZE;
    pub const ROOK_BEHIND_SUPPORT_PASSER_SIZE: usize = 1;

    pub const ROOK_BEHIND_ENEMY_PASSER: usize =
        ROOK_BEHIND_SUPPORT_PASSER + ROOK_BEHIND_SUPPORT_PASSER_SIZE;
    pub const ROOK_BEHIND_ENEMY_PASSER_SIZE: usize = 1;

    pub const PAWN_PASSED_WEAK: usize = ROOK_BEHIND_ENEMY_PASSER + ROOK_BEHIND_ENEMY_PASSER_SIZE;
    pub const PAWN_PASSED_WEAK_SIZE: usize = 1;

    pub const KNIGHT_SUPPORTED: usize = PAWN_PASSED_WEAK + PAWN_PASSED_WEAK_SIZE;
    pub const KNIGHT_SUPPORTED_SIZE: usize = 1;

    pub const KNIGHT_OUTPOST_TABLE: usize = KNIGHT_SUPPORTED + KNIGHT_SUPPORTED_SIZE;
    pub const KNIGHT_OUTPOST_TABLE_SIZE: usize = 64;

    pub const BISHOP_XRAY_KING: usize = KNIGHT_OUTPOST_TABLE + KNIGHT_OUTPOST_TABLE_SIZE;
    pub const BISHOP_XRAY_KING_SIZE: usize = 1;

    pub const ROOK_XRAY_KING: usize = BISHOP_XRAY_KING + BISHOP_XRAY_KING_SIZE;
    pub const ROOK_XRAY_KING_SIZE: usize = 1;

    pub const QUEEN_XRAY_KING: usize = ROOK_XRAY_KING + ROOK_XRAY_KING_SIZE;
    pub const QUEEN_XRAY_KING_SIZE: usize = 1;

    pub const ROOK_ON_OPEN: usize = QUEEN_XRAY_KING + QUEEN_XRAY_KING_SIZE;
    pub const ROOK_ON_OPEN_SIZE: usize = 1;

    pub const ROOK_ON_SEMI_OPEN: usize = ROOK_ON_OPEN + ROOK_ON_OPEN_SIZE;
    pub const ROOK_ON_SEMI_OPEN_SIZE: usize = 1;

    pub const QUEEN_ON_OPEN: usize = ROOK_ON_SEMI_OPEN + ROOK_ON_SEMI_OPEN_SIZE;
    pub const QUEEN_ON_OPEN_SIZE: usize = 1;

    pub const QUEEN_ON_SEMI_OPEN: usize = QUEEN_ON_OPEN + QUEEN_ON_OPEN_SIZE;
    pub const QUEEN_ON_SEMI_OPEN_SIZE: usize = 1;

    pub const ROOK_ON_SEVENTH: usize = QUEEN_ON_SEMI_OPEN + QUEEN_ON_SEMI_OPEN_SIZE;
    pub const ROOK_ON_SEVENTH_SIZE: usize = 1;

    pub const PAWNS: usize = ROOK_ON_SEVENTH + ROOK_ON_SEVENTH_SIZE;
    pub const PAWNS_SIZE: usize = 1;

    pub const KNIGHTS: usize = PAWNS + PAWNS_SIZE;
    pub const KNIGHTS_SIZE: usize = 1;

    pub const KNIGHT_VALUE_WITH_PAWNS: usize = KNIGHTS + KNIGHTS_SIZE;
    pub const KNIGHT_VALUE_WITH_PAWNS_SIZE: usize = 1;

    pub const BISHOPS: usize = KNIGHT_VALUE_WITH_PAWNS + KNIGHT_VALUE_WITH_PAWNS_SIZE;
    pub const BISHOPS_SIZE: usize = 1;

    pub const BISHOP_BONUS: usize = BISHOPS + BISHOPS_SIZE;
    pub const BISHOP_BONUS_SIZE: usize = 1;

    pub const ROOKS: usize = BISHOP_BONUS + BISHOP_BONUS_SIZE;
    pub const ROOKS_SIZE: usize = 1;

    pub const QUEENS: usize = ROOKS + ROOKS_SIZE;
    pub const QUEENS_SIZE: usize = 1;

    pub const DIAGONALLY_ADJACENT_SQUARES_WITHPAWNS: usize = QUEENS + QUEENS_SIZE;
    pub const DIAGONALLY_ADJACENT_SQUARES_WITHPAWNS_SIZE: usize = 5;

    pub const KNIGHT_MOBILITY: usize =
        DIAGONALLY_ADJACENT_SQUARES_WITHPAWNS + DIAGONALLY_ADJACENT_SQUARES_WITHPAWNS_SIZE;
    pub const KNIGHT_MOBILITY_SIZE: usize = 9;

    pub const BISHOP_MOBILITY: usize = KNIGHT_MOBILITY + KNIGHT_MOBILITY_SIZE;
    pub const BISHOP_MOBILITY_SIZE: usize = 14;

    pub const ROOK_MOBILITY: usize = BISHOP_MOBILITY + BISHOP_MOBILITY_SIZE;
    pub const ROOK_MOBILITY_SIZE: usize = 15;

    pub const QUEEN_MOBILITY: usize = ROOK_MOBILITY + ROOK_MOBILITY_SIZE;
    pub const QUEEN_MOBILITY_SIZE: usize = 28;

    pub const ATTACKERS: usize = QUEEN_MOBILITY + QUEEN_MOBILITY_SIZE;
    pub const ATTACKERS_SIZE: usize = 2;

    pub const KNIGHT_ATTACKED_SQ: usize = ATTACKERS + ATTACKERS_SIZE;
    pub const KNIGHT_ATTACKED_SQ_SIZE: usize = 2;

    pub const BISHOP_ATTACKED_SQ: usize = KNIGHT_ATTACKED_SQ + KNIGHT_ATTACKED_SQ_SIZE;
    pub const BISHOP_ATTACKED_SQ_SIZE: usize = 2;

    pub const ROOK_ATTACKED_SQ: usize = BISHOP_ATTACKED_SQ + BISHOP_ATTACKED_SQ_SIZE;
    pub const ROOK_ATTACKED_SQ_SIZE: usize = 2;

    pub const QUEEN_ATTACKED_SQ: usize = ROOK_ATTACKED_SQ + ROOK_ATTACKED_SQ_SIZE;
    pub const QUEEN_ATTACKED_SQ_SIZE: usize = 2;

    pub const KNIGHT_SAFE_CHECK: usize = QUEEN_ATTACKED_SQ + QUEEN_ATTACKED_SQ_SIZE;
    pub const KNIGHT_SAFE_CHECK_SIZE: usize = 2;

    pub const BISHOP_SAFE_CHECK: usize = KNIGHT_SAFE_CHECK + KNIGHT_SAFE_CHECK_SIZE;
    pub const BISHOP_SAFE_CHECK_SIZE: usize = 2;

    pub const ROOK_SAFE_CHECK: usize = BISHOP_SAFE_CHECK + BISHOP_SAFE_CHECK_SIZE;
    pub const ROOK_SAFE_CHECK_SIZE: usize = 2;

    pub const QUEEN_SAFE_CHECK: usize = ROOK_SAFE_CHECK + ROOK_SAFE_CHECK_SIZE;
    pub const QUEEN_SAFE_CHECK_SIZE: usize = 2;

    pub const PSQT: usize = QUEEN_SAFE_CHECK + QUEEN_SAFE_CHECK_SIZE;
    pub const PSQT_SIZE: usize = 6 * 64;

    pub const PHASE: usize = PSQT + PSQT_SIZE;
    pub const PHASE_SIZE: usize = 1;
}
