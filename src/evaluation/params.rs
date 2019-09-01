pub const TEMPO_BONUS_MG: i16 = 30;
pub const TEMPO_BONUS_EG: i16 = 32;
pub const SHIELDING_PAWN_MISSING_MG: [i16; 4] = [10, -31, -62, -97];
pub const SHIELDING_PAWN_MISSING_EG: [i16; 4] = [-3, 1, 1, -1];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG: [i16; 4] = [-38, -45, -99, -178];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG: [i16; 4] = [-9, -3, 8, 4];
pub const PAWN_DOUBLED_VALUE_MG: i16 = 8;
pub const PAWN_DOUBLED_VALUE_EG: i16 = -16;
pub const PAWN_ISOLATED_VALUE_MG: i16 = -15;
pub const PAWN_ISOLATED_VALUE_EG: i16 = -10;
pub const PAWN_BACKWARD_VALUE_MG: i16 = -14;
pub const PAWN_BACKWARD_VALUE_EG: i16 = -12;
pub const PAWN_SUPPORTED_VALUE_MG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [14, 20, 12, 14, 11, 11, 22, 10],
    [14, 10, 10, 15, 9, 5, 5, 6],
    [-10, 5, 24, 8, 18, 3, -4, -16],
    [-4, 8, 45, 63, 61, 52, 8, -8],
    [15, 21, 22, 26, 26, 21, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PAWN_SUPPORTED_VALUE_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [12, 14, 14, 28, 26, 17, 14, 13],
    [-3, 2, 3, 16, 18, 4, 6, -4],
    [-8, 2, 14, 30, 25, 16, 7, -2],
    [7, 21, 41, 76, 73, 49, 21, 25],
    [15, 20, 24, 30, 30, 26, 19, 13],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PAWN_ATTACK_CENTER_MG: i16 = -15;
pub const PAWN_ATTACK_CENTER_EG: i16 = -12;
pub const PAWN_MOBILITY_MG: i16 = 6;
pub const PAWN_MOBILITY_EG: i16 = 9;
pub const PAWN_PASSED_VALUES_MG: [i16; 7] = [0, -6, -22, -4, 25, 41, 70];
pub const PAWN_PASSED_VALUES_EG: [i16; 7] = [0, -4, 8, 22, 44, 53, 118];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_MG: [i16; 7] = [0, 9, 3, -14, -21, -20, 83];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_EG: [i16; 7] = [0, 10, 8, 32, 64, 154, 203];
pub const ROOK_BEHIND_SUPPORT_PASSER_MG: i16 = 8;
pub const ROOK_BEHIND_SUPPORT_PASSER_EG: i16 = 12;
pub const ROOK_BEHIND_ENEMY_PASSER_MG: i16 = 1;
pub const ROOK_BEHIND_ENEMY_PASSER_EG: i16 = -88;
pub const KNIGHT_SUPPORTED_BY_PAWN_MG: i16 = 2;
pub const KNIGHT_SUPPORTED_BY_PAWN_EG: i16 = 2;
pub const KNIGHT_OUTPOST_MG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-6, 1, 3, -1, 6, 11, 1, 3],
    [29, 20, 45, 38, 50, 42, 14, 35],
    [9, 59, 48, 84, 60, 46, 37, 12],
    [11, 18, 32, 25, 40, 43, 14, 16],
    [15, 19, 21, 25, 31, 20, 20, 16],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const KNIGHT_OUTPOST_EG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [6, 7, 9, 7, 9, 7, 2, -2],
    [18, 17, 34, 29, 34, 27, 18, 9],
    [12, 28, 32, 43, 45, 37, 27, 15],
    [10, 14, 32, 36, 44, 30, 17, 11],
    [15, 20, 20, 25, 27, 18, 24, 15],
    [0, 0, 0, 0, 1, 0, 1, 0],
];
pub const ROOK_ON_OPEN_FILE_BONUS_MG: i16 = 47;
pub const ROOK_ON_OPEN_FILE_BONUS_EG: i16 = 22;
pub const ROOK_ON_SEVENTH_MG: i16 = 2;
pub const ROOK_ON_SEVENTH_EG: i16 = 23;
pub const PAWN_PIECE_VALUE_MG: i16 = 102;
pub const PAWN_PIECE_VALUE_EG: i16 = 146;
pub const KNIGHT_PIECE_VALUE_MG: i16 = 431;
pub const KNIGHT_PIECE_VALUE_EG: i16 = 563;
pub const KNIGHT_VALUE_WITH_PAWNS: [i16; 17] = [
    -41, -80, -46, -44, -40, -35, -21, -17, -2, 2, 8, 14, 19, 25, 23, 32, 29,
];
pub const BISHOP_PIECE_VALUE_MG: i16 = 464;
pub const BISHOP_PIECE_VALUE_EG: i16 = 556;
pub const BISHOP_PAIR_BONUS_MG: i16 = 30;
pub const BISHOP_PAIR_BONUS_EG: i16 = 82;
pub const ROOK_PIECE_VALUE_MG: i16 = 621;
pub const ROOK_PIECE_VALUE_EG: i16 = 938;
pub const QUEEN_PIECE_VALUE_MG: i16 = 1445;
pub const QUEEN_PIECE_VALUE_EG: i16 = 1814;
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG: [i16; 5] = [-19, -22, -34, -10, -100];
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG: [i16; 5] = [7, 9, 0, -1, -100];
pub const KNIGHT_MOBILITY_BONUS_MG: [i16; 9] = [-53, -18, -6, -1, 2, 2, -1, -6, -8];
pub const KNIGHT_MOBILITY_BONUS_EG: [i16; 9] = [-69, -7, -3, 0, 5, 14, 15, 16, 11];
pub const BISHOP_MOBILITY_BONUS_MG: [i16; 14] =
    [-10, 4, 13, 20, 27, 36, 42, 46, 44, 43, 45, 46, 71, 90];
pub const BISHOP_MOBILITY_BONUS_EG: [i16; 14] =
    [-30, -17, -5, 4, 24, 37, 39, 44, 48, 46, 40, 40, 39, 42];
pub const ROOK_MOBILITY_BONUS_MG: [i16; 15] =
    [-28, -14, -9, 1, -5, 3, 11, 20, 19, 19, 22, 17, 17, 14, 17];
pub const ROOK_MOBILITY_BONUS_EG: [i16; 15] =
    [-30, -9, -5, -1, 10, 20, 16, 22, 30, 37, 41, 44, 45, 45, 43];
pub const QUEEN_MOBILITY_BONUS_MG: [i16; 28] = [
    -23, -3, -13, -2, 0, 8, 14, 12, 13, 15, 17, 12, 14, 13, 12, 10, 10, 9, 12, 12, 23, 26, 31, 44,
    47, 55, 54, 55,
];
pub const QUEEN_MOBILITY_BONUS_EG: [i16; 28] = [
    -40, -35, -30, -31, -35, -42, -39, -14, -3, 1, 10, 28, 29, 40, 50, 60, 66, 73, 81, 79, 86, 90,
    89, 90, 75, 86, 85, 86,
];
pub const ATTACK_WEIGHT: [i16; 8] = [0, 20, 87, 185, 195, 110, 100, 100];
pub const SAFETY_TABLE: [i16; 100] = [
    0, 0, 104, -4, 85, 86, 91, 56, 36, 66, 68, 68, 73, 54, 88, 72, 58, 84, 119, 115, 113, 111, 107,
    94, 135, 122, 102, 134, 132, 155, 155, 155, 162, 185, 207, 206, 223, 223, 238, 247, 261, 272,
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
    [-33, -18, -25, -25, -32, -26, -20, -41],
    [-28, -31, -6, 3, 3, -7, -34, -30],
    [-26, -17, 17, 31, 31, 15, -17, -25],
    [-16, 4, 7, 24, 31, 12, 1, -10],
    [-5, 26, 58, 40, 63, 84, 16, 8],
    [-11, 3, 13, 21, 19, 6, -9, -30],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_PAWN_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-22, -21, -18, -17, -22, -18, -22, -22],
    [-13, -15, -8, -12, -11, -9, -13, -15],
    [-4, -6, -4, -12, -13, -4, -7, -3],
    [17, 6, -14, -34, -30, -15, 2, 17],
    [41, 38, 3, -44, -42, -2, 36, 38],
    [33, 27, 23, -15, -13, 5, 21, 38],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_KNIGHT_MG: [[i16; 8]; 8] = [
    [-59, -52, -29, -33, -34, -19, -46, -55],
    [-45, -28, -18, -9, -6, -21, -42, -28],
    [-44, 5, 5, 17, 16, 6, -5, -48],
    [-22, -4, 24, 21, 22, 20, 17, -24],
    [7, -7, 48, 31, 34, 48, 1, -3],
    [-17, 36, 51, 67, 81, 58, 34, -26],
    [-56, -24, 41, 6, -1, 39, -23, -47],
    [-105, -47, -37, -36, -31, -41, -40, -119],
];
pub const PSQT_KNIGHT_EG: [[i16; 8]; 8] = [
    [-57, -46, -28, -21, -17, -26, -43, -55],
    [-27, -22, -10, -8, -10, -10, -25, -20],
    [-26, -1, 9, 22, 23, 12, 1, -17],
    [-19, 5, 19, 27, 30, 18, 6, -14],
    [-9, 0, 23, 13, 14, 17, -4, -15],
    [-24, -3, 26, 10, 12, 19, 0, -28],
    [-51, -19, -2, -17, -18, -11, -13, -49],
    [-115, -52, -45, -35, -30, -38, -41, -117],
];
pub const PSQT_BISHOP_MG: [[i16; 8]; 8] = [
    [-4, 21, -8, -15, -9, -19, 15, -19],
    [-6, 18, 16, 5, -13, 13, 11, 2],
    [9, 13, 18, -2, 9, 8, 21, -10],
    [-12, 2, -13, 19, 18, -2, -19, 2],
    [-28, -9, 13, 22, 20, 4, -4, -32],
    [8, 14, 22, 23, 18, 31, 29, 11],
    [-41, -17, -2, -3, -13, -15, -21, -42],
    [-52, -17, -16, -41, -38, -19, -16, -62],
];
pub const PSQT_BISHOP_EG: [[i16; 8]; 8] = [
    [-34, -31, -19, -10, -10, -16, -23, -28],
    [-36, -8, -22, -9, -6, -17, -12, -28],
    [-11, 7, 10, 10, 11, 14, 7, -19],
    [-14, -8, 6, 9, 8, 6, -3, -13],
    [-10, 1, 1, 7, 9, -5, -3, -7],
    [-10, 1, 5, -7, -14, 4, -7, -9],
    [-24, -18, -17, -17, -10, -12, -15, -33],
    [-34, -20, -31, -20, -35, -20, -22, -47],
];
pub const PSQT_KING_MG: [[i16; 8]; 8] = [
    [72, 74, 52, 21, 29, 48, 72, 84],
    [83, 66, -4, -40, -39, 6, 59, 88],
    [-36, -36, -66, -80, -82, -71, -30, -36],
    [-58, -77, -70, -94, -95, -74, -74, -57],
    [-66, -86, -82, -101, -103, -85, -84, -66],
    [-60, -80, -80, -100, -100, -80, -80, -61],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
];
pub const PSQT_KING_EG: [[i16; 8]; 8] = [
    [-107, -68, -62, -76, -77, -56, -68, -114],
    [-63, -34, -17, -13, -11, -17, -31, -63],
    [-46, -11, 3, 16, 12, 3, -9, -46],
    [-45, -1, 24, 36, 37, 22, 3, -51],
    [-32, 28, 47, 56, 60, 43, 26, -31],
    [-22, 53, 55, 62, 59, 56, 46, -19],
    [-30, 25, 34, 21, 25, 33, 25, -25],
    [-65, -44, -37, -21, -21, -24, -45, -58],
];
