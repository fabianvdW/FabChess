pub const TEMPO_BONUS_MG: i16 = 29;
pub const TEMPO_BONUS_EG: i16 = 31;
pub const SHIELDING_PAWN_MISSING_MG: [i16; 4] = [12, -31, -64, -97];
pub const SHIELDING_PAWN_MISSING_EG: [i16; 4] = [-11, 1, 2, 7];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG: [i16; 4] = [-41, -42, -101, -177];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG: [i16; 4] = [1, 1, 7, -8];
pub const PAWN_DOUBLED_VALUE_MG: i16 = 11;
pub const PAWN_DOUBLED_VALUE_EG: i16 = -18;
pub const PAWN_ISOLATED_VALUE_MG: i16 = -23;
pub const PAWN_ISOLATED_VALUE_EG: i16 = -10;
pub const PAWN_BACKWARD_VALUE_MG: i16 = -15;
pub const PAWN_BACKWARD_VALUE_EG: i16 = -9;
pub const PAWN_SUPPORTED_VALUE_MG: i16 = 16;
pub const PAWN_SUPPORTED_VALUE_EG: i16 = 13;
pub const PAWN_ATTACK_CENTER_MG: i16 = -5;
pub const PAWN_ATTACK_CENTER_EG: i16 = -11;
pub const PAWN_PASSED_VALUES_MG: [i16; 7] = [0, -4, -12, -2, 27, 67, 96];
pub const PAWN_PASSED_VALUES_EG: [i16; 7] = [0, -4, 6, 17, 31, 61, 127];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_MG: [i16; 7] = [0, 11, 24, 23, 41, 81, 110];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_EG: [i16; 7] = [0, 19, 9, 30, 56, 133, 196];
pub const KNIGHT_SUPPORTED_BY_PAWN_MG: i16 = 7;
pub const KNIGHT_SUPPORTED_BY_PAWN_EG: i16 = 13;
pub const KNIGHT_OUTPOST_MG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0, 1],
    [8, 7, 11, 13, 12, 10, 7, 5],
    [5, 13, 11, 20, 19, 10, 12, 5],
    [10, 17, 16, 21, 21, 15, 17, 10],
    [15, 20, 20, 25, 25, 20, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const KNIGHT_OUTPOST_EG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, -1, 1, 1, 0, 0, -1],
    [7, 7, 11, 14, 15, 11, 8, 5],
    [5, 11, 14, 20, 19, 14, 11, 6],
    [10, 15, 17, 21, 21, 17, 16, 10],
    [15, 20, 19, 25, 25, 19, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const ROOK_ON_OPEN_FILE_BONUS_MG: i16 = 74;
pub const ROOK_ON_OPEN_FILE_BONUS_EG: i16 = 7;
pub const ROOK_ON_SEVENTH_MG: i16 = 14;
pub const ROOK_ON_SEVENTH_EG: i16 = 16;
pub const PAWN_PIECE_VALUE_MG: i16 = 130;
pub const PAWN_PIECE_VALUE_EG: i16 = 157;
pub const KNIGHT_PIECE_VALUE_MG: i16 = 528;
pub const KNIGHT_PIECE_VALUE_EG: i16 = 509;
pub const KNIGHT_VALUE_WITH_PAWNS: [i16; 17] = [
    -41, -47, -38, -27, -20, -16, -8, -10, -4, 0, 1, 6, 7, 16, 11, 21, 17,
];
pub const BISHOP_PIECE_VALUE_MG: i16 = 519;
pub const BISHOP_PIECE_VALUE_EG: i16 = 512;
pub const BISHOP_PAIR_BONUS_MG: i16 = 54;
pub const BISHOP_PAIR_BONUS_EG: i16 = 74;
pub const ROOK_PIECE_VALUE_MG: i16 = 682;
pub const ROOK_PIECE_VALUE_EG: i16 = 850;
pub const QUEEN_PIECE_VALUE_MG: i16 = 1506;
pub const QUEEN_PIECE_VALUE_EG: i16 = 1631;
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG: [i16; 5] = [8, 6, -7, -38, -100];
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG: [i16; 5] = [-3, 2, 0, -37, -100];
pub const KNIGHT_MOBILITY_BONUS_MG: [i16; 9] = [-73, -23, -3, -1, 20, 22, 20, 19, 26];
pub const KNIGHT_MOBILITY_BONUS_EG: [i16; 9] = [-75, -44, -17, -2, -1, 9, 12, 22, 17];
pub const BISHOP_MOBILITY_BONUS_MG: [i16; 14] =
    [-20, -3, 12, 23, 30, 36, 40, 43, 52, 52, 61, 72, 85, 90];
pub const BISHOP_MOBILITY_BONUS_EG: [i16; 14] =
    [-47, -25, -6, 3, 16, 26, 33, 39, 40, 37, 42, 36, 53, 53];
pub const ROOK_MOBILITY_BONUS_MG: [i16; 15] =
    [-33, -15, -9, 0, 1, 5, 12, 18, 18, 22, 27, 25, 29, 34, 33];
pub const ROOK_MOBILITY_BONUS_EG: [i16; 15] = [
    -85, -57, -31, -18, 0, 13, 23, 29, 36, 42, 49, 52, 58, 58, 53,
];
pub const QUEEN_MOBILITY_BONUS_MG: [i16; 28] = [
    -27, -20, -10, -9, 2, 13, 9, 8, 7, 12, 11, 9, 4, 12, 16, 14, 19, 27, 33, 32, 38, 42, 44, 47,
    48, 51, 53, 55,
];
pub const QUEEN_MOBILITY_BONUS_EG: [i16; 28] = [
    -40, -30, -20, -12, -8, -4, -2, 0, 0, 3, 4, 13, 16, 23, 29, 31, 35, 42, 49, 54, 57, 65, 66, 70,
    71, 78, 80, 85,
];
pub const ATTACK_WEIGHT: [i16; 8] = [0, 59, 126, 131, 104, 100, 100, 100];
pub const SAFETY_TABLE: [i16; 100] = [
    0, 0, 2, 8, 4, 4, 12, 12, 14, 23, 20, 24, 31, 37, 32, 38, 46, 48, 55, 60, 70, 73, 82, 84, 92,
    97, 104, 111, 122, 130, 141, 151, 169, 179, 193, 203, 214, 224, 237, 249, 260, 272, 283, 295,
    307, 319, 330, 342, 354, 366, 377, 389, 401, 412, 424, 436, 448, 459, 471, 483, 494, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
];
pub const KNIGHT_ATTACK_WORTH: i16 = 2;
pub const BISHOP_ATTACK_WORTH: i16 = 2;
pub const ROOK_ATTACK_WORTH: i16 = 3;
pub const QUEEN_ATTACK_WORTH: i16 = 5;
pub const PSQT_PAWN_MG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-20, 11, -10, -14, -13, -9, 9, -23],
    [-18, -11, -3, -9, -11, 0, -13, -18],
    [-28, -14, 14, 45, 42, 14, -16, -31],
    [-15, 11, 9, 48, 45, 3, 11, -16],
    [-5, 1, 6, 9, 9, 8, 1, -4],
    [-2, 11, 5, 5, 6, 6, 11, -3],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_PAWN_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-22, -17, -7, -13, -12, -6, -14, -23],
    [-25, -20, -8, -4, -2, -10, -21, -26],
    [-14, -13, -12, -15, -13, -9, -12, -15],
    [5, 4, -9, -19, -23, -6, 3, 3],
    [24, 22, 4, -12, -13, 11, 21, 27],
    [26, 22, 10, 7, 10, 11, 16, 25],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_KNIGHT_MG: [[i16; 8]; 8] = [
    [-41, -19, -31, -28, -27, -32, -15, -40],
    [-39, -21, -2, 16, 12, -2, -21, -37],
    [-38, 0, 13, 17, 11, 12, -2, -41],
    [-27, 3, 23, 18, 15, 16, 7, -27],
    [-26, 19, 42, 39, 39, 21, 19, -26],
    [-31, 20, 28, 40, 39, 27, 19, -30],
    [-41, -20, 6, 5, 4, 4, -21, -42],
    [-52, -41, -31, -30, -30, -32, -40, -53],
];
pub const PSQT_KNIGHT_EG: [[i16; 8]; 8] = [
    [-51, -42, -29, -26, -25, -31, -42, -50],
    [-42, -24, -9, -4, -5, -15, -28, -40],
    [-30, -15, -10, 13, 10, -6, -9, -31],
    [-25, 0, 14, 25, 20, 15, 1, -27],
    [-25, 3, 16, 26, 25, 20, 1, -23],
    [-32, -8, 10, 12, 13, 8, -12, -33],
    [-43, -23, -10, -1, -1, -11, -25, -42],
    [-54, -42, -32, -30, -30, -31, -42, -54],
];
pub const PSQT_BISHOP_MG: [[i16; 8]; 8] = [
    [-48, -9, -11, -28, -26, -8, -9, -49],
    [-28, 37, 15, 9, 10, 16, 38, -28],
    [-8, 13, 22, 5, 1, 22, 12, -11],
    [-10, 10, 0, 27, 23, 1, 0, -10],
    [-11, 1, 19, 27, 26, 18, -9, -10],
    [-15, 10, 16, 11, 9, 16, 11, -16],
    [-34, 9, 14, -1, -1, 14, 11, -34],
    [-49, -10, -11, -30, -31, -12, -10, -49],
];
pub const PSQT_BISHOP_EG: [[i16; 8]; 8] = [
    [-49, -29, -18, -14, -14, -23, -28, -46],
    [-31, -12, -11, -7, -5, -11, -16, -30],
    [-20, -2, 2, 8, 12, 5, -1, -19],
    [-18, -3, 2, 15, 7, 1, -3, -20],
    [-15, -1, 2, 14, 12, 4, 0, -17],
    [-19, 0, 0, 7, 5, 2, 0, -16],
    [-28, -10, -8, -1, 0, -11, -10, -32],
    [-48, -29, -32, -20, -22, -30, -31, -49],
];
pub const PSQT_KING_MG: [[i16; 8]; 8] = [
    [42, 80, 8, 18, 16, 7, 81, 40],
    [41, 40, -7, -23, -22, -8, 31, 32],
    [-21, -39, -41, -43, -43, -42, -40, -21],
    [-41, -60, -61, -81, -81, -61, -61, -41],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
];
pub const PSQT_KING_EG: [[i16; 8]; 8] = [
    [-78, -41, -34, -40, -40, -36, -43, -78],
    [-38, -12, -1, 3, 2, 2, -10, -36],
    [-33, 0, 14, 22, 22, 12, 2, -32],
    [-35, -5, 24, 31, 31, 24, -1, -35],
    [-27, 7, 34, 37, 36, 32, 8, -25],
    [-27, 5, 28, 26, 27, 29, 6, -22],
    [-30, -14, -5, 2, 2, -4, -16, -29],
    [-51, -40, -30, -22, -21, -29, -40, -51],
];
