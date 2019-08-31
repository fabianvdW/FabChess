pub const TEMPO_BONUS_MG: i16 = 29;
pub const TEMPO_BONUS_EG: i16 = 32;
pub const SHIELDING_PAWN_MISSING_MG: [i16; 4] = [12, -30, -63, -99];
pub const SHIELDING_PAWN_MISSING_EG: [i16; 4] = [-1, 2, 0, -2];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG: [i16; 4] = [-38, -44, -100, -179];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG: [i16; 4] = [-8, -2, 7, 4];
pub const PAWN_DOUBLED_VALUE_MG: i16 = 7;
pub const PAWN_DOUBLED_VALUE_EG: i16 = -16;
pub const PAWN_ISOLATED_VALUE_MG: i16 = -15;
pub const PAWN_ISOLATED_VALUE_EG: i16 = -10;
pub const PAWN_BACKWARD_VALUE_MG: i16 = -14;
pub const PAWN_BACKWARD_VALUE_EG: i16 = -12;
pub const PAWN_SUPPORTED_VALUE_MG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [14, 20, 13, 13, 10, 11, 22, 9],
    [13, 9, 9, 15, 9, 4, 5, 5],
    [-9, 4, 23, 8, 18, 3, -5, -15],
    [0, 11, 41, 56, 54, 47, 11, -3],
    [15, 21, 22, 26, 26, 21, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PAWN_SUPPORTED_VALUE_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [12, 13, 13, 28, 25, 16, 14, 13],
    [-3, 2, 3, 15, 17, 3, 6, -4],
    [-8, 2, 14, 29, 24, 15, 6, -2],
    [7, 20, 37, 66, 63, 43, 20, 21],
    [15, 20, 23, 28, 28, 24, 19, 13],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PAWN_ATTACK_CENTER_MG: i16 = -13;
pub const PAWN_ATTACK_CENTER_EG: i16 = -13;
pub const PAWN_MOBILITY_MG: i16 = 6;
pub const PAWN_MOBILITY_EG: i16 = 9;
pub const PAWN_PASSED_VALUES_MG: [i16; 7] = [0, -8, -22, -4, 26, 46, 76];
pub const PAWN_PASSED_VALUES_EG: [i16; 7] = [0, -4, 9, 23, 44, 50, 115];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_MG: [i16; 7] = [0, 9, 2, -14, -20, -10, 90];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_EG: [i16; 7] = [0, 10, 6, 30, 61, 146, 193];
pub const ROOK_BEHIND_SUPPORT_PASSER_MG: i16 = 8;
pub const ROOK_BEHIND_SUPPORT_PASSER_EG: i16 = 11;
pub const ROOK_BEHIND_ENEMY_PASSER_MG: i16 = -4;
pub const ROOK_BEHIND_ENEMY_PASSER_EG: i16 = -73;
pub const KNIGHT_SUPPORTED_BY_PAWN_MG: i16 = 3;
pub const KNIGHT_SUPPORTED_BY_PAWN_EG: i16 = 3;
pub const KNIGHT_OUTPOST_MG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-5, 1, 3, 0, 5, 11, 1, 3],
    [26, 17, 40, 38, 50, 37, 12, 30],
    [9, 52, 43, 79, 57, 42, 33, 11],
    [11, 18, 29, 27, 39, 38, 15, 14],
    [15, 19, 21, 25, 29, 20, 20, 16],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const KNIGHT_OUTPOST_EG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [4, 5, 9, 5, 7, 7, 2, -2],
    [16, 15, 31, 27, 34, 25, 16, 9],
    [11, 26, 30, 42, 44, 34, 24, 13],
    [10, 14, 30, 33, 39, 28, 17, 11],
    [15, 20, 20, 25, 26, 18, 23, 15],
    [0, 0, 0, 0, 1, 0, 1, 0],
];
pub const ROOK_ON_OPEN_FILE_BONUS_MG: i16 = 47;
pub const ROOK_ON_OPEN_FILE_BONUS_EG: i16 = 22;
pub const ROOK_ON_SEVENTH_MG: i16 = 1;
pub const ROOK_ON_SEVENTH_EG: i16 = 25;
pub const PAWN_PIECE_VALUE_MG: i16 = 101;
pub const PAWN_PIECE_VALUE_EG: i16 = 146;
pub const KNIGHT_PIECE_VALUE_MG: i16 = 440;
pub const KNIGHT_PIECE_VALUE_EG: i16 = 561;
pub const KNIGHT_VALUE_WITH_PAWNS: [i16; 17] = [
    -41, -71, -44, -42, -38, -33, -22, -17, -2, 1, 7, 13, 18, 24, 22, 31, 28,
];
pub const BISHOP_PIECE_VALUE_MG: i16 = 470;
pub const BISHOP_PIECE_VALUE_EG: i16 = 553;
pub const BISHOP_PAIR_BONUS_MG: i16 = 31;
pub const BISHOP_PAIR_BONUS_EG: i16 = 82;
pub const ROOK_PIECE_VALUE_MG: i16 = 629;
pub const ROOK_PIECE_VALUE_EG: i16 = 927;
pub const QUEEN_PIECE_VALUE_MG: i16 = 1470;
pub const QUEEN_PIECE_VALUE_EG: i16 = 1787;
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG: [i16; 5] = [-16, -20, -31, -11, -100];
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG: [i16; 5] = [7, 9, 1, -6, -100];
pub const KNIGHT_MOBILITY_BONUS_MG: [i16; 9] = [-57, -17, -5, 0, 4, 4, 0, -5, -6];
pub const KNIGHT_MOBILITY_BONUS_EG: [i16; 9] = [-71, -11, -6, -1, 5, 15, 16, 18, 13];
pub const BISHOP_MOBILITY_BONUS_MG: [i16; 14] =
    [-10, 4, 14, 21, 28, 36, 43, 46, 44, 44, 44, 44, 74, 90];
pub const BISHOP_MOBILITY_BONUS_EG: [i16; 14] =
    [-33, -19, -8, 2, 22, 36, 39, 44, 49, 47, 42, 43, 41, 44];
pub const ROOK_MOBILITY_BONUS_MG: [i16; 15] =
    [-28, -12, -8, 2, -4, 3, 12, 21, 19, 19, 21, 17, 17, 14, 19];
pub const ROOK_MOBILITY_BONUS_EG: [i16; 15] =
    [-35, -14, -9, -4, 8, 18, 15, 21, 29, 37, 42, 46, 47, 48, 45];
pub const QUEEN_MOBILITY_BONUS_MG: [i16; 28] = [
    -23, -4, -13, -4, -1, 7, 14, 12, 13, 16, 17, 13, 15, 14, 13, 11, 12, 11, 15, 16, 28, 31, 35,
    45, 47, 54, 54, 55,
];
pub const QUEEN_MOBILITY_BONUS_EG: [i16; 28] = [
    -40, -33, -27, -26, -29, -37, -37, -16, -5, -2, 7, 26, 28, 39, 49, 59, 65, 72, 78, 77, 82, 86,
    84, 85, 73, 83, 83, 86,
];
pub const ATTACK_WEIGHT: [i16; 8] = [0, 12, 79, 166, 181, 109, 100, 100];
pub const SAFETY_TABLE: [i16; 100] = [
    0, 0, 92, -1, 57, 87, 66, 41, 11, 56, 59, 67, 54, 47, 79, 64, 48, 81, 108, 105, 107, 103, 100,
    87, 130, 114, 102, 132, 129, 152, 152, 155, 161, 184, 205, 205, 221, 223, 237, 248, 261, 272,
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
    [-33, -19, -24, -25, -32, -25, -19, -40],
    [-27, -30, -6, 4, 3, -6, -33, -29],
    [-24, -16, 17, 31, 30, 15, -17, -23],
    [-16, 5, 9, 24, 31, 13, 2, -10],
    [-6, 22, 54, 36, 57, 79, 14, 7],
    [-9, 5, 11, 17, 15, 6, -4, -23],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_PAWN_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-23, -21, -18, -17, -21, -18, -21, -22],
    [-14, -16, -9, -12, -11, -9, -14, -16],
    [-5, -8, -4, -12, -12, -4, -8, -4],
    [15, 5, -15, -35, -31, -16, 1, 16],
    [40, 39, 7, -38, -36, 3, 37, 38],
    [29, 25, 21, -10, -9, 6, 20, 33],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_KNIGHT_MG: [[i16; 8]; 8] = [
    [-54, -54, -26, -31, -31, -17, -48, -50],
    [-44, -29, -17, -9, -6, -21, -41, -28],
    [-44, 3, 6, 18, 18, 7, -7, -48],
    [-22, -5, 22, 18, 18, 19, 14, -24],
    [6, -5, 49, 30, 32, 47, 2, -4],
    [-20, 34, 50, 66, 78, 55, 32, -26],
    [-54, -24, 34, 4, -1, 31, -23, -47],
    [-93, -45, -35, -34, -31, -39, -40, -103],
];
pub const PSQT_KNIGHT_EG: [[i16; 8]; 8] = [
    [-55, -45, -27, -21, -17, -26, -43, -53],
    [-29, -24, -12, -9, -11, -12, -27, -22],
    [-29, -5, 5, 20, 21, 8, -3, -21],
    [-21, 3, 18, 25, 27, 16, 5, -17],
    [-9, 1, 22, 14, 16, 18, -2, -15],
    [-25, -3, 27, 12, 14, 21, -1, -29],
    [-50, -21, -3, -16, -16, -11, -16, -49],
    [-101, -49, -42, -34, -30, -36, -41, -103],
];
pub const PSQT_BISHOP_MG: [[i16; 8]; 8] = [
    [-9, 19, -9, -14, -9, -20, 12, -23],
    [-7, 23, 14, 4, -13, 14, 13, 1],
    [8, 13, 18, -2, 9, 9, 23, -11],
    [-13, 1, -14, 21, 21, -4, -19, -1],
    [-26, -12, 12, 24, 21, 5, -9, -29],
    [9, 14, 22, 19, 15, 29, 27, 10],
    [-41, -14, -1, -3, -12, -12, -16, -41],
    [-52, -15, -15, -38, -36, -17, -14, -59],
];
pub const PSQT_BISHOP_EG: [[i16; 8]; 8] = [
    [-34, -30, -20, -8, -8, -17, -23, -30],
    [-36, -6, -22, -9, -6, -17, -11, -27],
    [-13, 7, 10, 11, 12, 14, 7, -22],
    [-15, -9, 6, 10, 9, 6, -4, -14],
    [-11, 0, 2, 9, 10, -4, -4, -9],
    [-8, 2, 5, -7, -13, 5, -5, -8],
    [-24, -19, -18, -15, -10, -14, -17, -32],
    [-37, -22, -31, -20, -33, -22, -24, -47],
];
pub const PSQT_KING_MG: [[i16; 8]; 8] = [
    [66, 69, 46, 17, 25, 41, 68, 79],
    [78, 58, -7, -40, -39, 4, 53, 83],
    [-34, -38, -60, -71, -72, -65, -33, -34],
    [-54, -72, -68, -91, -91, -71, -71, -53],
    [-64, -84, -81, -101, -102, -84, -83, -64],
    [-60, -80, -80, -100, -100, -80, -80, -61],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
];
pub const PSQT_KING_EG: [[i16; 8]; 8] = [
    [-101, -64, -60, -72, -73, -53, -64, -107],
    [-58, -31, -16, -13, -10, -16, -29, -59],
    [-44, -10, 3, 16, 12, 3, -9, -44],
    [-44, 0, 24, 36, 37, 22, 4, -49],
    [-31, 28, 47, 55, 59, 43, 26, -29],
    [-22, 48, 52, 57, 55, 53, 42, -19],
    [-30, 17, 26, 16, 20, 26, 17, -25],
    [-61, -43, -35, -21, -21, -25, -43, -56],
];
