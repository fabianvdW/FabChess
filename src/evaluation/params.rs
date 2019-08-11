pub const TEMPO_BONUS_MG: i16 = 29;
pub const TEMPO_BONUS_EG: i16 = 29;
pub const SHIELDING_PAWN_MISSING_MG: [i16; 4] = [13, -28, -63, -101];
pub const SHIELDING_PAWN_MISSING_EG: [i16; 4] = [-2, -2, 0, 2];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG: [i16; 4] = [-40, -41, -102, -178];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG: [i16; 4] = [-10, -5, 6, 10];
pub const PAWN_DOUBLED_VALUE_MG: i16 = 5;
pub const PAWN_DOUBLED_VALUE_EG: i16 = -22;
pub const PAWN_ISOLATED_VALUE_MG: i16 = -18;
pub const PAWN_ISOLATED_VALUE_EG: i16 = -12;
pub const PAWN_BACKWARD_VALUE_MG: i16 = -16;
pub const PAWN_BACKWARD_VALUE_EG: i16 = -13;
pub const PAWN_SUPPORTED_VALUE_MG: i16 = 15;
pub const PAWN_SUPPORTED_VALUE_EG: i16 = 15;
pub const PAWN_ATTACK_CENTER_MG: i16 = -5;
pub const PAWN_ATTACK_CENTER_EG: i16 = -10;
pub const PAWN_PASSED_VALUES_MG: [i16; 7] = [0, -4, -21, -8, 21, 69, 93];
pub const PAWN_PASSED_VALUES_EG: [i16; 7] = [0, -5, 8, 24, 41, 56, 114];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_MG: [i16; 7] = [0, 11, 16, 11, 23, 70, 108];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_EG: [i16; 7] = [0, 15, 7, 29, 56, 128, 188];
pub const KNIGHT_SUPPORTED_BY_PAWN_MG: i16 = 4;
pub const KNIGHT_SUPPORTED_BY_PAWN_EG: i16 = 8;
pub const KNIGHT_OUTPOST_MG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-1, 0, 0, 1, 0, 1, 0, 1],
    [11, 8, 14, 18, 21, 13, 7, 8],
    [6, 17, 15, 31, 25, 15, 14, 6],
    [10, 18, 17, 23, 24, 17, 17, 10],
    [15, 20, 20, 25, 25, 20, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const KNIGHT_OUTPOST_EG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 2, 1, 0, -1],
    [8, 8, 13, 16, 20, 12, 9, 6],
    [6, 13, 17, 24, 23, 17, 12, 7],
    [10, 15, 18, 22, 23, 19, 16, 10],
    [15, 20, 19, 25, 25, 19, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const ROOK_ON_OPEN_FILE_BONUS_MG: i16 = 51;
pub const ROOK_ON_OPEN_FILE_BONUS_EG: i16 = 15;
pub const ROOK_ON_SEVENTH_MG: i16 = 10;
pub const ROOK_ON_SEVENTH_EG: i16 = 25;
pub const PAWN_PIECE_VALUE_MG: i16 = 116;
pub const PAWN_PIECE_VALUE_EG: i16 = 163;
pub const KNIGHT_PIECE_VALUE_MG: i16 = 490;
pub const KNIGHT_PIECE_VALUE_EG: i16 = 524;
pub const KNIGHT_VALUE_WITH_PAWNS: [i16; 17] = [
    -41, -49, -38, -28, -22, -19, -13, -14, -5, -5, 0, 4, 9, 14, 13, 20, 18,
];
pub const BISHOP_PIECE_VALUE_MG: i16 = 501;
pub const BISHOP_PIECE_VALUE_EG: i16 = 519;
pub const BISHOP_PAIR_BONUS_MG: i16 = 36;
pub const BISHOP_PAIR_BONUS_EG: i16 = 74;
pub const ROOK_PIECE_VALUE_MG: i16 = 695;
pub const ROOK_PIECE_VALUE_EG: i16 = 859;
pub const QUEEN_PIECE_VALUE_MG: i16 = 1510;
pub const QUEEN_PIECE_VALUE_EG: i16 = 1655;
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG: [i16; 5] = [1, -3, -13, -34, -100];
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG: [i16; 5] = [3, 4, -4, -34, -100];
pub const KNIGHT_MOBILITY_BONUS_MG: [i16; 9] = [-72, -21, -7, 3, 12, 14, 12, 10, 17];
pub const KNIGHT_MOBILITY_BONUS_EG: [i16; 9] = [-75, -39, -10, 0, 3, 11, 14, 18, 14];
pub const BISHOP_MOBILITY_BONUS_MG: [i16; 14] =
    [-17, 1, 11, 19, 28, 37, 45, 48, 47, 46, 52, 64, 84, 90];
pub const BISHOP_MOBILITY_BONUS_EG: [i16; 14] =
    [-43, -22, -9, -1, 17, 31, 34, 40, 43, 41, 37, 36, 51, 52];
pub const ROOK_MOBILITY_BONUS_MG: [i16; 15] =
    [-26, -5, -3, 6, -1, 6, 14, 22, 20, 21, 23, 20, 21, 29, 30];
pub const ROOK_MOBILITY_BONUS_EG: [i16; 15] =
    [-69, -31, -17, -9, 5, 16, 14, 20, 31, 38, 43, 47, 48, 50, 45];
pub const QUEEN_MOBILITY_BONUS_MG: [i16; 28] = [
    -27, -18, -13, -8, -4, 2, 8, 8, 11, 14, 17, 14, 15, 16, 16, 17, 20, 23, 30, 29, 37, 41, 43, 47,
    48, 51, 53, 55,
];
pub const QUEEN_MOBILITY_BONUS_EG: [i16; 28] = [
    -40, -30, -21, -13, -10, -8, -7, -3, -1, 1, 5, 17, 21, 27, 33, 38, 41, 45, 52, 55, 59, 66, 67,
    70, 71, 78, 80, 85,
];
pub const ATTACK_WEIGHT: [i16; 8] = [0, 68, 150, 164, 111, 101, 100, 100];
pub const SAFETY_TABLE: [i16; 100] = [
    0, 0, 20, 7, 23, 16, 21, 18, 13, 30, 28, 29, 34, 38, 38, 42, 47, 52, 61, 64, 77, 75, 84, 85,
    95, 99, 104, 112, 123, 132, 142, 151, 168, 179, 194, 203, 215, 224, 237, 249, 260, 272, 283,
    295, 307, 319, 330, 342, 354, 366, 377, 389, 401, 412, 424, 436, 448, 459, 471, 483, 494, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
];
pub const KNIGHT_ATTACK_WORTH: i16 = 2;
pub const BISHOP_ATTACK_WORTH: i16 = 2;
pub const ROOK_ATTACK_WORTH: i16 = 3;
pub const QUEEN_ATTACK_WORTH: i16 = 5;
pub const PSQT_PAWN_MG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-23, -3, -11, -12, -17, -14, -5, -31],
    [-23, -15, -5, 6, 2, -8, -18, -27],
    [-21, -10, 17, 37, 32, 12, -14, -24],
    [-17, 11, 21, 33, 45, 12, 7, -13],
    [-6, 4, 14, 12, 16, 21, 4, -3],
    [-3, 10, 5, 6, 6, 6, 10, -5],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_PAWN_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-22, -15, -10, -9, -11, -11, -15, -21],
    [-23, -20, -11, -6, -6, -9, -18, -24],
    [-17, -14, -9, -12, -11, -8, -12, -16],
    [5, 5, -7, -22, -21, -7, 3, 6],
    [25, 27, 9, -18, -16, 16, 26, 28],
    [24, 21, 10, 4, 6, 9, 16, 24],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_KNIGHT_MG: [[i16; 8]; 8] = [
    [-42, -34, -29, -26, -25, -27, -29, -41],
    [-38, -22, -5, -2, 0, -7, -23, -30],
    [-43, -3, 2, 16, 14, 2, -11, -45],
    [-21, 2, 21, 19, 22, 16, 8, -22],
    [-20, 13, 47, 39, 38, 31, 16, -21],
    [-30, 23, 30, 45, 46, 30, 21, -29],
    [-42, -20, 9, 5, 4, 7, -21, -42],
    [-55, -41, -31, -30, -30, -32, -40, -56],
];
pub const PSQT_KNIGHT_EG: [[i16; 8]; 8] = [
    [-51, -40, -28, -24, -23, -29, -40, -50],
    [-40, -24, -10, -10, -11, -16, -28, -36],
    [-32, -16, -12, 14, 13, -8, -12, -32],
    [-24, 0, 13, 23, 23, 13, 2, -25],
    [-21, 3, 20, 23, 22, 23, 1, -20],
    [-31, -7, 14, 13, 15, 11, -10, -32],
    [-44, -22, -8, -2, -2, -10, -24, -43],
    [-57, -42, -33, -30, -30, -31, -42, -58],
];
pub const PSQT_BISHOP_MG: [[i16; 8]; 8] = [
    [-44, -1, -10, -25, -24, -15, -6, -46],
    [-18, 27, 16, 6, -7, 13, 22, -18],
    [-4, 11, 17, 3, 8, 13, 14, -11],
    [-10, 7, -8, 25, 25, -5, -4, -8],
    [-12, 0, 19, 29, 26, 17, -5, -11],
    [-8, 11, 17, 12, 10, 18, 14, -6],
    [-35, 6, 12, -1, -2, 12, 8, -34],
    [-49, -10, -11, -30, -31, -12, -10, -50],
];
pub const PSQT_BISHOP_EG: [[i16; 8]; 8] = [
    [-47, -28, -16, -12, -12, -18, -27, -44],
    [-30, -10, -14, -10, -10, -13, -17, -27],
    [-19, -3, 2, 7, 12, 4, -1, -21],
    [-18, -5, 0, 12, 8, 2, -4, -19],
    [-14, 0, 3, 13, 12, 2, 1, -15],
    [-14, 1, 2, 5, 2, 4, 1, -11],
    [-28, -12, -10, -2, -1, -12, -12, -32],
    [-47, -28, -32, -20, -23, -29, -30, -49],
];
pub const PSQT_KING_MG: [[i16; 8]; 8] = [
    [47, 58, 28, 3, 13, 24, 58, 58],
    [50, 46, -11, -30, -29, -8, 38, 50],
    [-22, -39, -43, -46, -46, -44, -40, -22],
    [-42, -61, -61, -82, -82, -62, -62, -42],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
];
pub const PSQT_KING_EG: [[i16; 8]; 8] = [
    [-80, -46, -36, -47, -47, -32, -48, -82],
    [-39, -15, -4, -4, -2, -2, -12, -37],
    [-36, -3, 9, 21, 19, 9, -1, -35],
    [-36, -3, 25, 35, 35, 25, 1, -37],
    [-27, 12, 38, 42, 42, 35, 12, -25],
    [-26, 10, 31, 30, 31, 32, 11, -22],
    [-30, -11, -2, 3, 3, -1, -13, -29],
    [-52, -40, -30, -22, -21, -29, -40, -51],
];
