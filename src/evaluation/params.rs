pub const TEMPO_BONUS_MG: i16 = 10;
pub const TEMPO_BONUS_EG: i16 = 1;
pub const SHIELDING_PAWN_MISSING_MG: [i16; 4] = [21, -22, -53, -126];
pub const SHIELDING_PAWN_MISSING_EG: [i16; 4] = [-3, -9, -7, 19];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG: [i16; 4] = [-45, -41, -99, -176];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG: [i16; 4] = [3, -3, 7, -7];
pub const PAWN_DOUBLED_VALUE_MG: i16 = -13;
pub const PAWN_DOUBLED_VALUE_EG: i16 = -16;
pub const PAWN_ISOLATED_VALUE_MG: i16 = -9;
pub const PAWN_ISOLATED_VALUE_EG: i16 = -12;
pub const PAWN_BACKWARD_VALUE_MG: i16 = -14;
pub const PAWN_BACKWARD_VALUE_EG: i16 = -14;
pub const PAWN_SUPPORTED_VALUE_MG: i16 = 18;
pub const PAWN_SUPPORTED_VALUE_EG: i16 = 13;
pub const PAWN_ATTACK_CENTER_MG: i16 = -14;
pub const PAWN_ATTACK_CENTER_EG: i16 = -12;
pub const PAWN_PASSED_VALUES_MG: [i16; 7] = [0, 13, -5, -5, 37, 104, 110];
pub const PAWN_PASSED_VALUES_EG: [i16; 7] = [0, -2, 11, 30, 47, 73, 149];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_MG: [i16; 7] = [0, 20, 24, 5, 16, 77, 113];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_EG: [i16; 7] = [0, 20, 16, 35, 65, 126, 223];
pub const KNIGHT_SUPPORTED_BY_PAWN_MG: i16 = 0;
pub const KNIGHT_SUPPORTED_BY_PAWN_EG: i16 = -5;
pub const KNIGHT_OUTPOST_MG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-3, 1, -9, -3, -3, -16, -2, 1],
    [20, 5, 22, 33, 23, 15, 6, 19],
    [7, 18, 15, 35, 34, 9, 10, 8],
    [10, 17, 20, 35, 30, 20, 19, 10],
    [15, 20, 22, 25, 29, 19, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const KNIGHT_OUTPOST_EG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-2, -1, -7, 0, 0, -8, -1, 3],
    [11, 5, 23, 30, 20, 15, 6, 5],
    [8, -2, 25, 37, 41, 17, 15, 8],
    [10, 15, 19, 39, 25, 21, 16, 13],
    [15, 20, 20, 24, 27, 20, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const ROOK_ON_OPEN_FILE_BONUS_MG: i16 = 61;
pub const ROOK_ON_OPEN_FILE_BONUS_EG: i16 = 20;
pub const ROOK_ON_SEVENTH_MG: i16 = 44;
pub const ROOK_ON_SEVENTH_EG: i16 = 38;
pub const PAWN_PIECE_VALUE_MG: i16 = 109;
pub const PAWN_PIECE_VALUE_EG: i16 = 178;
pub const KNIGHT_PIECE_VALUE_MG: i16 = 470;
pub const KNIGHT_PIECE_VALUE_EG: i16 = 453;
pub const KNIGHT_VALUE_WITH_PAWNS: [i16; 17] = [
    -309, -59, -9, -6, 3, 7, 9, 11, 16, 13, 10, 22, 8, 0, 12, 7, 21,
];
pub const BISHOP_PIECE_VALUE_MG: i16 = 485;
pub const BISHOP_PIECE_VALUE_EG: i16 = 463;
pub const BISHOP_PAIR_BONUS_MG: i16 = 26;
pub const BISHOP_PAIR_BONUS_EG: i16 = 109;
pub const ROOK_PIECE_VALUE_MG: i16 = 668;
pub const ROOK_PIECE_VALUE_EG: i16 = 781;
pub const QUEEN_PIECE_VALUE_MG: i16 = 1504;
pub const QUEEN_PIECE_VALUE_EG: i16 = 1565;
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG: [i16; 5] = [-7, -14, -8, -36, -100];
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG: [i16; 5] = [-16, -9, -24, -37, -100];
pub const KNIGHT_MOBILITY_BONUS_MG: [i16; 9] = [-73, -32, -5, 6, 13, 13, 7, 7, 15];
pub const KNIGHT_MOBILITY_BONUS_EG: [i16; 9] = [-76, -60, -37, -16, -4, 10, 19, 20, 9];
pub const BISHOP_MOBILITY_BONUS_MG: [i16; 14] =
    [-13, -7, 17, 21, 26, 33, 39, 42, 41, 44, 46, 74, 84, 93];
pub const BISHOP_MOBILITY_BONUS_EG: [i16; 14] =
    [-71, -40, -29, -6, 18, 41, 44, 47, 61, 54, 57, 33, 58, -14];
pub const ROOK_MOBILITY_BONUS_MG: [i16; 15] =
    [-60, -11, -7, 3, 0, 6, 10, 17, 14, 20, 32, 27, 24, 35, 41];
pub const ROOK_MOBILITY_BONUS_EG: [i16; 15] = [
    -122, -86, -50, -41, -17, 3, 20, 28, 43, 53, 60, 65, 69, 74, 57,
];
pub const QUEEN_MOBILITY_BONUS_MG: [i16; 28] = [
    -24, -13, -21, -18, -13, -5, -2, 0, 3, 9, 10, 9, 15, 18, 19, 22, 26, 35, 37, 40, 40, 46, 48,
    49, 48, 51, 53, 55,
];
pub const QUEEN_MOBILITY_BONUS_EG: [i16; 28] = [
    -40, -29, -21, -15, -18, -14, -17, -16, -9, -6, 3, 7, 17, 23, 33, 39, 39, 53, 54, 59, 59, 54,
    65, 61, 69, 74, 80, 84,
];
pub const ATTACK_WEIGHT: [i16; 8] = [0, 37, 152, 203, 117, 100, 100, 100];
pub const SAFETY_TABLE: [i16; 100] = [
    0, 0, 17, -1, 51, 38, 23, 28, 35, 28, 25, 8, 28, 26, 46, 32, 38, 62, 53, 68, 69, 74, 86, 80,
    86, 98, 105, 114, 122, 132, 137, 152, 171, 178, 192, 201, 213, 224, 237, 249, 260, 272, 283,
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
    [-26, 3, -17, -19, -20, -20, 5, -38],
    [-23, -15, -10, 7, 6, -11, -15, -26],
    [-16, 5, 22, 29, 29, 11, 4, -27],
    [-20, 4, 10, 30, 21, 9, -8, -7],
    [-11, 6, 5, 38, 14, 10, 6, 12],
    [1, 14, 6, 8, 8, 7, 7, 2],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_PAWN_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-22, -23, -5, -9, -18, -8, -15, -13],
    [-23, -25, -7, -1, 0, -12, -24, -23],
    [-19, -12, -12, -17, -13, -9, -9, -17],
    [7, 6, -17, -22, -20, -17, 8, -2],
    [25, 26, 10, -1, -6, 11, 15, 25],
    [6, 34, 13, 8, 19, 14, 28, 25],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_KNIGHT_MG: [[i16; 8]; 8] = [
    [-43, 1, -33, -29, -27, -27, -5, -42],
    [-37, -30, -7, -8, -10, -24, -29, -36],
    [-47, -10, -3, 24, 19, 1, -26, -62],
    [-14, -3, 25, 11, 19, 28, 11, -20],
    [-11, -14, 55, 18, 14, 25, -4, -15],
    [-30, 22, 41, 68, 57, 42, 24, -29],
    [-40, -18, 10, 8, 13, 2, -17, -44],
    [-53, -40, -30, -29, -29, -31, -40, -54],
];
pub const PSQT_KNIGHT_EG: [[i16; 8]; 8] = [
    [-54, -55, -38, -33, -38, -33, -52, -51],
    [-39, -29, -21, -18, -20, -23, -33, -41],
    [-51, -26, -21, 6, 1, -13, -29, -35],
    [-21, 0, 20, 21, 11, 17, -4, -28],
    [-12, -14, 25, 22, 17, 25, 3, -16],
    [-26, 1, 22, 27, 20, 26, 1, -25],
    [-35, -26, -7, 5, 10, -10, -19, -40],
    [-53, -39, -25, -26, -28, -31, -39, -52],
];
pub const PSQT_BISHOP_MG: [[i16; 8]; 8] = [
    [-53, -16, -13, -29, -32, -17, -13, -46],
    [-18, 34, 1, 2, -6, 8, 28, -19],
    [-8, 17, 22, 7, 2, 15, 19, -13],
    [10, 2, -7, 25, 26, -10, -1, -22],
    [-10, -24, 16, 36, 35, 15, -21, -6],
    [-3, 15, 20, 18, 14, 20, 16, 8],
    [-28, 7, 16, -5, -2, 11, 9, -33],
    [-48, -10, -11, -31, -30, -10, -9, -49],
];
pub const PSQT_BISHOP_EG: [[i16; 8]; 8] = [
    [-51, -41, -41, -32, -18, -50, -39, -40],
    [-33, -18, -15, -10, -6, -12, -16, -32],
    [-26, -6, 8, 20, 13, 4, 9, -23],
    [-14, -9, 17, 6, 13, 12, -7, -29],
    [-19, 9, 5, 15, 10, 1, 0, -18],
    [-17, 7, 0, 1, -3, 10, 1, -9],
    [-25, -9, -9, 1, 3, -13, -13, -28],
    [-48, -25, -28, -18, -21, -26, -32, -48],
];
pub const PSQT_KING_MG: [[i16; 8]; 8] = [
    [44, 78, -30, 48, 54, -21, 74, 62],
    [40, 38, -5, -14, -25, -5, 23, 26],
    [-28, -45, -45, -46, -38, -41, -40, -24],
    [-41, -61, -62, -80, -79, -61, -61, -42],
    [-60, -80, -80, -100, -99, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -79, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
];
pub const PSQT_KING_EG: [[i16; 8]; 8] = [
    [-86, -48, -46, -51, -51, -56, -40, -85],
    [-39, -25, -9, -12, -12, -2, -21, -36],
    [-43, -9, 4, 12, 17, 10, -4, -33],
    [-32, 9, 18, 0, 1, 25, 18, -36],
    [-22, 25, 31, -28, -23, 35, 25, -21],
    [-18, 31, 45, 24, 19, 52, 33, -13],
    [-27, 3, 20, 38, 36, 21, -2, -30],
    [-49, -35, -22, -9, -8, -22, -35, -52],
];
