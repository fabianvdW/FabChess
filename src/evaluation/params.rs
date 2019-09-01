pub const TEMPO_BONUS_MG: i16 = 29;
pub const TEMPO_BONUS_EG: i16 = 31;
pub const SHIELDING_PAWN_MISSING_MG: [i16; 4] = [13, -29, -63, -101];
pub const SHIELDING_PAWN_MISSING_EG: [i16; 4] = [-2, 2, 0, -2];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG: [i16; 4] = [-36, -43, -102, -180];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG: [i16; 4] = [-7, -2, 7, 3];
pub const PAWN_DOUBLED_VALUE_MG: i16 = 8;
pub const PAWN_DOUBLED_VALUE_EG: i16 = -16;
pub const PAWN_ISOLATED_VALUE_MG: i16 = -15;
pub const PAWN_ISOLATED_VALUE_EG: i16 = -10;
pub const PAWN_BACKWARD_VALUE_MG: i16 = -14;
pub const PAWN_BACKWARD_VALUE_EG: i16 = -12;
pub const PAWN_SUPPORTED_VALUE_MG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [14, 20, 12, 14, 10, 11, 22, 9],
    [14, 10, 10, 15, 9, 5, 5, 6],
    [-10, 5, 24, 8, 18, 3, -5, -15],
    [-2, 9, 43, 60, 58, 50, 9, -6],
    [15, 21, 22, 26, 26, 21, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PAWN_SUPPORTED_VALUE_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [12, 14, 14, 28, 26, 17, 15, 13],
    [-3, 2, 3, 16, 18, 4, 6, -4],
    [-8, 2, 14, 30, 25, 16, 6, -2],
    [7, 20, 39, 72, 69, 47, 20, 23],
    [15, 20, 24, 29, 29, 25, 19, 13],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PAWN_ATTACK_CENTER_MG: i16 = -14;
pub const PAWN_ATTACK_CENTER_EG: i16 = -12;
pub const PAWN_MOBILITY_MG: i16 = 6;
pub const PAWN_MOBILITY_EG: i16 = 9;
pub const PAWN_PASSED_VALUES_MG: [i16; 7] = [0, -6, -22, -4, 25, 42, 72];
pub const PAWN_PASSED_VALUES_EG: [i16; 7] = [0, -4, 9, 22, 44, 52, 117];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_MG: [i16; 7] = [0, 10, 2, -15, -21, -17, 86];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_EG: [i16; 7] = [0, 10, 7, 31, 63, 151, 199];
pub const ROOK_BEHIND_SUPPORT_PASSER_MG: i16 = 7;
pub const ROOK_BEHIND_SUPPORT_PASSER_EG: i16 = 11;
pub const ROOK_BEHIND_ENEMY_PASSER_MG: i16 = -2;
pub const ROOK_BEHIND_ENEMY_PASSER_EG: i16 = -86;
pub const KNIGHT_SUPPORTED_BY_PAWN_MG: i16 = 2;
pub const KNIGHT_SUPPORTED_BY_PAWN_EG: i16 = 2;
pub const KNIGHT_OUTPOST_MG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-6, 1, 3, -1, 6, 11, 1, 3],
    [28, 19, 43, 38, 50, 40, 13, 33],
    [9, 56, 46, 82, 59, 44, 35, 12],
    [11, 18, 31, 26, 40, 41, 14, 15],
    [15, 19, 21, 25, 30, 20, 20, 16],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const KNIGHT_OUTPOST_EG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 6, 9, 6, 8, 7, 2, -2],
    [17, 16, 33, 28, 34, 26, 17, 9],
    [12, 27, 31, 43, 45, 36, 26, 14],
    [10, 14, 31, 35, 42, 29, 17, 11],
    [15, 20, 20, 25, 27, 18, 24, 15],
    [0, 0, 0, 0, 1, 0, 1, 0],
];
pub const ROOK_ON_OPEN_FILE_BONUS_MG: i16 = 47;
pub const ROOK_ON_OPEN_FILE_BONUS_EG: i16 = 23;
pub const ROOK_ON_SEVENTH_MG: i16 = 2;
pub const ROOK_ON_SEVENTH_EG: i16 = 24;
pub const PAWN_PIECE_VALUE_MG: i16 = 101;
pub const PAWN_PIECE_VALUE_EG: i16 = 147;
pub const KNIGHT_PIECE_VALUE_MG: i16 = 433;
pub const KNIGHT_PIECE_VALUE_EG: i16 = 563;
pub const KNIGHT_VALUE_WITH_PAWNS: [i16; 17] = [
    -41, -76, -45, -43, -39, -34, -22, -17, -3, 1, 7, 14, 19, 25, 23, 32, 28,
];
pub const BISHOP_PIECE_VALUE_MG: i16 = 465;
pub const BISHOP_PIECE_VALUE_EG: i16 = 555;
pub const BISHOP_PAIR_BONUS_MG: i16 = 31;
pub const BISHOP_PAIR_BONUS_EG: i16 = 82;
pub const ROOK_PIECE_VALUE_MG: i16 = 623;
pub const ROOK_PIECE_VALUE_EG: i16 = 934;
pub const QUEEN_PIECE_VALUE_MG: i16 = 1456;
pub const QUEEN_PIECE_VALUE_EG: i16 = 1803;
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG: [i16; 5] = [-18, -22, -33, -10, -100];
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG: [i16; 5] = [7, 9, 0, -3, -100];
pub const KNIGHT_MOBILITY_BONUS_MG: [i16; 9] = [-55, -18, -6, -1, 2, 3, 0, -6, -7];
pub const KNIGHT_MOBILITY_BONUS_EG: [i16; 9] = [-70, -9, -5, -1, 5, 15, 16, 17, 13];
pub const BISHOP_MOBILITY_BONUS_MG: [i16; 14] =
    [-11, 2, 12, 20, 27, 36, 42, 46, 45, 44, 46, 47, 72, 90];
pub const BISHOP_MOBILITY_BONUS_EG: [i16; 14] =
    [-32, -19, -7, 2, 23, 37, 39, 44, 49, 47, 42, 42, 40, 43];
pub const ROOK_MOBILITY_BONUS_MG: [i16; 15] =
    [-30, -14, -10, 0, -5, 2, 11, 20, 19, 20, 23, 18, 18, 15, 18];
pub const ROOK_MOBILITY_BONUS_EG: [i16; 15] =
    [-33, -12, -8, -3, 9, 19, 15, 21, 30, 37, 42, 46, 47, 48, 45];
pub const QUEEN_MOBILITY_BONUS_MG: [i16; 28] = [
    -23, -4, -14, -4, -1, 7, 13, 11, 12, 15, 17, 13, 15, 14, 13, 11, 12, 11, 14, 15, 26, 28, 33,
    45, 47, 55, 54, 55,
];
pub const QUEEN_MOBILITY_BONUS_EG: [i16; 28] = [
    -40, -34, -29, -29, -33, -41, -39, -16, -5, -1, 8, 27, 29, 40, 50, 60, 66, 73, 80, 79, 85, 89,
    87, 88, 74, 85, 84, 86,
];
pub const ATTACK_WEIGHT: [i16; 8] = [0, 15, 89, 192, 188, 109, 100, 100];
pub const SAFETY_TABLE: [i16; 100] = [
    0, 0, 99, 0, 73, 87, 79, 55, 21, 60, 61, 68, 64, 49, 90, 68, 51, 84, 119, 117, 111, 107, 101,
    87, 133, 117, 101, 132, 130, 153, 153, 156, 161, 184, 206, 206, 222, 223, 237, 247, 261, 272,
    283, 295, 307, 320, 330, 342, 355, 366, 377, 389, 401, 412, 424, 436, 448, 459, 471, 483, 494,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500,
];
pub const KNIGHT_ATTACK_WORTH: i16 = 2;
pub const BISHOP_ATTACK_WORTH: i16 = 2;
pub const ROOK_ATTACK_WORTH: i16 = 3;
pub const QUEEN_ATTACK_WORTH: i16 = 5;
pub const PSQT_PAWN_MG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-33, -18, -24, -24, -32, -26, -19, -40],
    [-27, -31, -6, 4, 3, -6, -34, -29],
    [-25, -16, 17, 31, 31, 14, -17, -24],
    [-16, 4, 8, 23, 30, 12, 1, -10],
    [-6, 24, 56, 38, 60, 82, 15, 7],
    [-10, 4, 12, 19, 17, 6, -7, -27],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_PAWN_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-22, -20, -18, -17, -21, -18, -21, -22],
    [-13, -15, -8, -12, -11, -8, -13, -16],
    [-4, -7, -4, -12, -13, -4, -7, -3],
    [16, 5, -15, -35, -31, -16, 2, 17],
    [41, 38, 4, -41, -40, 0, 36, 38],
    [31, 26, 22, -13, -11, 5, 20, 36],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_KNIGHT_MG: [[i16; 8]; 8] = [
    [-57, -53, -29, -33, -33, -19, -47, -53],
    [-45, -29, -18, -9, -6, -21, -42, -29],
    [-45, 3, 5, 16, 16, 6, -6, -48],
    [-22, -4, 23, 20, 21, 19, 16, -24],
    [7, -6, 49, 33, 35, 48, 2, -3],
    [-18, 35, 51, 67, 80, 57, 33, -26],
    [-55, -24, 38, 5, -1, 36, -23, -47],
    [-100, -46, -36, -35, -31, -40, -40, -112],
];
pub const PSQT_KNIGHT_EG: [[i16; 8]; 8] = [
    [-56, -46, -28, -21, -17, -26, -44, -54],
    [-28, -23, -11, -10, -12, -11, -26, -21],
    [-28, -3, 7, 20, 22, 10, -1, -19],
    [-20, 4, 19, 26, 29, 17, 6, -15],
    [-9, 1, 23, 15, 16, 18, -3, -15],
    [-24, -3, 27, 11, 14, 20, 0, -28],
    [-51, -20, -2, -16, -17, -11, -14, -49],
    [-109, -51, -44, -34, -30, -37, -41, -111],
];
pub const PSQT_BISHOP_MG: [[i16; 8]; 8] = [
    [-6, 20, -9, -15, -9, -20, 14, -21],
    [-7, 19, 15, 4, -13, 13, 11, 1],
    [10, 13, 18, -2, 9, 8, 21, -10],
    [-12, 2, -14, 19, 19, -3, -19, 1],
    [-27, -10, 13, 23, 21, 5, -6, -31],
    [9, 14, 22, 22, 17, 30, 28, 11],
    [-41, -16, -2, -3, -13, -14, -19, -42],
    [-52, -16, -16, -40, -37, -18, -15, -61],
];
pub const PSQT_BISHOP_EG: [[i16; 8]; 8] = [
    [-34, -31, -20, -9, -9, -16, -23, -29],
    [-36, -8, -22, -9, -6, -17, -12, -28],
    [-11, 7, 10, 10, 12, 14, 7, -20],
    [-14, -8, 6, 9, 8, 6, -3, -13],
    [-10, 1, 1, 8, 10, -4, -3, -8],
    [-9, 1, 5, -7, -13, 4, -6, -9],
    [-24, -19, -17, -16, -10, -13, -16, -33],
    [-35, -21, -31, -20, -34, -21, -23, -47],
];
pub const PSQT_KING_MG: [[i16; 8]; 8] = [
    [72, 73, 51, 20, 29, 47, 72, 85],
    [82, 62, -8, -43, -42, 3, 56, 87],
    [-35, -37, -64, -77, -79, -69, -32, -35],
    [-56, -75, -69, -93, -93, -73, -73, -55],
    [-65, -85, -82, -101, -103, -85, -84, -65],
    [-60, -80, -80, -100, -100, -80, -80, -61],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
];
pub const PSQT_KING_EG: [[i16; 8]; 8] = [
    [-102, -65, -61, -75, -76, -54, -66, -108],
    [-59, -32, -18, -14, -12, -17, -30, -59],
    [-44, -11, 2, 15, 11, 2, -9, -44],
    [-44, -1, 23, 35, 36, 22, 3, -50],
    [-32, 28, 47, 55, 60, 43, 26, -30],
    [-22, 51, 54, 60, 57, 55, 45, -19],
    [-30, 22, 31, 19, 23, 30, 22, -25],
    [-63, -44, -36, -21, -21, -24, -44, -57],
];
