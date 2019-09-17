pub const TEMPO_BONUS_MG: i16 = 10;
pub const TEMPO_BONUS_EG: i16 = 15;
pub const SHIELDING_PAWN_MISSING_MG: [i16; 4] = [2, -39, -62, -84];
pub const SHIELDING_PAWN_MISSING_EG: [i16; 4] = [-9, 3, 3, 3];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG: [i16; 4] = [-40, -48, -102, -172];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG: [i16; 4] = [-9, -4, 9, 2];
pub const PAWN_DOUBLED_VALUE_MG: i16 = 6;
pub const PAWN_DOUBLED_VALUE_EG: i16 = -18;
pub const PAWN_ISOLATED_VALUE_MG: i16 = -15;
pub const PAWN_ISOLATED_VALUE_EG: i16 = -14;
pub const PAWN_BACKWARD_VALUE_MG: i16 = -14;
pub const PAWN_BACKWARD_VALUE_EG: i16 = -18;
pub const PAWN_SUPPORTED_VALUE_MG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [7, 10, 24, 10, 20, -6, 38, 18],
    [4, 11, 12, 13, 14, 3, 7, 12],
    [-8, 2, 12, 5, 27, 31, -4, -16],
    [-13, -12, 49, 71, 73, 62, 15, -14],
    [15, 23, 24, 28, 28, 23, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PAWN_SUPPORTED_VALUE_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [11, 21, 24, 28, 38, 21, 16, 18],
    [-2, 10, 7, 29, 16, 2, 0, -7],
    [-5, 5, 27, 34, 35, 9, 8, -3],
    [12, 26, 48, 92, 96, 64, 28, 30],
    [16, 22, 26, 34, 37, 29, 19, 12],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PAWN_ATTACK_CENTER_MG: i16 = -19;
pub const PAWN_ATTACK_CENTER_EG: i16 = -16;
pub const PAWN_MOBILITY_MG: i16 = 6;
pub const PAWN_MOBILITY_EG: i16 = 17;
pub const PAWN_PASSED_VALUES_MG: [i16; 7] = [0, -11, -20, -5, 29, 44, 81];
pub const PAWN_PASSED_VALUES_EG: [i16; 7] = [0, -1, 17, 49, 90, 133, 230];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_MG: [i16; 7] = [0, 5, -1, -19, -29, -12, 82];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_EG: [i16; 7] = [0, 24, 16, 51, 110, 259, 323];
pub const ROOK_BEHIND_SUPPORT_PASSER_MG: i16 = 13;
pub const ROOK_BEHIND_SUPPORT_PASSER_EG: i16 = 1;
pub const ROOK_BEHIND_ENEMY_PASSER_MG: i16 = 7;
pub const ROOK_BEHIND_ENEMY_PASSER_EG: i16 = -114;
pub const PAWN_PASSED_WEAK_MG: i16 = -16;
pub const PAWN_PASSED_WEAK_EG: i16 = -38;
pub const KNIGHT_SUPPORTED_BY_PAWN_MG: i16 = 1;
pub const KNIGHT_SUPPORTED_BY_PAWN_EG: i16 = 6;
pub const KNIGHT_OUTPOST_MG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-7, 2, 17, -3, 8, 2, 1, 7],
    [34, 31, 49, 45, 58, 55, 20, 42],
    [16, 50, 60, 67, 75, 70, 74, 11],
    [11, 6, 39, 32, 36, 57, 23, 18],
    [15, 19, 19, 25, 35, 20, 18, 16],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const KNIGHT_OUTPOST_EG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [8, 8, 17, 8, 12, 3, 3, -2],
    [11, 22, 36, 39, 37, 34, 26, 17],
    [17, 28, 37, 48, 55, 51, 37, 17],
    [10, 9, 32, 57, 48, 41, 22, 11],
    [15, 22, 20, 25, 28, 18, 24, 15],
    [0, 1, 0, 0, 1, 0, 1, 0],
];
pub const ROOK_ON_OPEN_FILE_BONUS_MG: i16 = 45;
pub const ROOK_ON_OPEN_FILE_BONUS_EG: i16 = 33;
pub const ROOK_ON_SEVENTH_MG: i16 = 1;
pub const ROOK_ON_SEVENTH_EG: i16 = 43;
pub const PAWN_PIECE_VALUE_MG: i16 = 108;
pub const PAWN_PIECE_VALUE_EG: i16 = 189;
pub const KNIGHT_PIECE_VALUE_MG: i16 = 438;
pub const KNIGHT_PIECE_VALUE_EG: i16 = 718;
pub const KNIGHT_VALUE_WITH_PAWNS: [i16; 17] = [
    -41, -113, -48, -38, -31, -21, -5, 0, 18, 22, 31, 37, 44, 50, 49, 56, 54,
];
pub const BISHOP_PIECE_VALUE_MG: i16 = 489;
pub const BISHOP_PIECE_VALUE_EG: i16 = 701;
pub const BISHOP_PAIR_BONUS_MG: i16 = 32;
pub const BISHOP_PAIR_BONUS_EG: i16 = 114;
pub const ROOK_PIECE_VALUE_MG: i16 = 670;
pub const ROOK_PIECE_VALUE_EG: i16 = 1247;
pub const QUEEN_PIECE_VALUE_MG: i16 = 1542;
pub const QUEEN_PIECE_VALUE_EG: i16 = 2342;
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG: [i16; 5] = [-16, -16, -24, -4, -100];
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG: [i16; 5] = [57, 57, 40, 15, -100];
pub const KNIGHT_MOBILITY_BONUS_MG: [i16; 9] = [-46, -18, -8, -2, 1, 1, -1, -4, -5];
pub const KNIGHT_MOBILITY_BONUS_EG: [i16; 9] = [-69, 1, 6, 12, 24, 38, 41, 45, 41];
pub const BISHOP_MOBILITY_BONUS_MG: [i16; 14] =
    [-10, 4, 13, 21, 29, 38, 45, 49, 47, 46, 49, 48, 66, 92];
pub const BISHOP_MOBILITY_BONUS_EG: [i16; 14] =
    [-30, -22, -9, 3, 31, 51, 57, 65, 72, 68, 61, 61, 47, 47];
pub const ROOK_MOBILITY_BONUS_MG: [i16; 15] =
    [-24, -9, -4, 4, 0, 8, 16, 24, 21, 22, 24, 20, 18, 16, 18];
pub const ROOK_MOBILITY_BONUS_EG: [i16; 15] =
    [-30, 2, 8, 11, 24, 37, 34, 39, 54, 63, 70, 75, 76, 79, 74];
pub const QUEEN_MOBILITY_BONUS_MG: [i16; 28] = [
    -20, 4, -5, 5, 7, 13, 20, 17, 19, 21, 21, 16, 17, 16, 13, 11, 11, 9, 12, 13, 24, 24, 28, 45,
    50, 59, 56, 55,
];
pub const QUEEN_MOBILITY_BONUS_EG: [i16; 28] = [
    -40, -37, -35, -39, -44, -49, -43, -9, 4, 11, 26, 54, 56, 73, 86, 100, 109, 117, 127, 124, 130,
    136, 125, 118, 89, 98, 89, 88,
];
pub const ATTACK_WEIGHT_MG: [i16; 8] = [0, 80, 131, 171, 223, 114, 100, 100];
pub const SAFETY_TABLE_MG: [i16; 100] = [
    0, 0, 1, 2, 3, 11, 28, 14, 12, 15, 20, 27, 38, 41, 38, 41, 46, 53, 60, 67, 72, 76, 84, 87, 91,
    99, 107, 115, 124, 133, 141, 152, 170, 181, 191, 202, 213, 225, 237, 248, 260, 272, 283, 295,
    307, 319, 330, 342, 354, 366, 377, 389, 401, 412, 424, 436, 448, 459, 471, 483, 494, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
];
pub const ATTACK_WEIGHT_EG: [i16; 8] = [0, 59, 82, 138, 201, 110, 100, 100];
pub const SAFETY_TABLE_EG: [i16; 100] = [
    0, 0, 2, 3, 5, 7, 8, 10, 12, 16, 19, 23, 27, 30, 35, 39, 44, 50, 56, 62, 68, 75, 82, 85, 89,
    97, 105, 113, 122, 131, 140, 150, 169, 180, 191, 202, 213, 225, 237, 248, 260, 272, 283, 295,
    307, 319, 330, 342, 354, 366, 377, 389, 401, 412, 424, 436, 448, 459, 471, 483, 494, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
];
pub const KNIGHT_ATTACK_WORTH_MG: i16 = 6;
pub const KNIGHT_ATTACK_WORTH_EG: i16 = 1;
pub const BISHOP_ATTACK_WORTH_MG: i16 = 7;
pub const BISHOP_ATTACK_WORTH_EG: i16 = 0;
pub const ROOK_ATTACK_WORTH_MG: i16 = 7;
pub const ROOK_ATTACK_WORTH_EG: i16 = 2;
pub const QUEEN_ATTACK_WORTH_MG: i16 = 7;
pub const QUEEN_ATTACK_WORTH_EG: i16 = 4;
pub const KNIGHT_SAFE_CHECK_MG: i16 = 14;
pub const KNIGHT_SAFE_CHECK_EG: i16 = 7;
pub const BISHOP_SAFE_CHECK_MG: i16 = 6;
pub const BISHOP_SAFE_CHECK_EG: i16 = 13;
pub const ROOK_SAFE_CHECK_MG: i16 = 7;
pub const ROOK_SAFE_CHECK_EG: i16 = 13;
pub const QUEEN_SAFE_CHECK_MG: i16 = 6;
pub const QUEEN_SAFE_CHECK_EG: i16 = 29;
pub const PSQT_PAWN_MG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-43, -43, -35, -43, -26, -25, 4, -35],
    [-38, -40, -12, -1, 3, -5, -26, -26],
    [-34, -30, 10, 28, 33, 28, -2, -14],
    [-28, -10, -4, 10, 32, 19, 18, 3],
    [-9, 6, 38, 35, 62, 113, 55, 20],
    [3, 1, 26, 32, 35, 8, -18, -61],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_PAWN_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-21, -30, -31, -37, -14, -19, -33, -36],
    [-9, -18, -16, -15, -11, -1, -17, -17],
    [8, -9, -7, -19, -10, -2, -7, -8],
    [37, 7, -21, -42, -39, -18, 1, 12],
    [61, 48, 4, -48, -58, -4, 42, 38],
    [90, 57, 45, -9, -15, 6, 23, 32],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_KNIGHT_MG: [[i16; 8]; 8] = [
    [-67, -49, -43, -26, -28, -17, -46, -64],
    [-56, -43, -23, -3, -8, -1, -17, -23],
    [-53, -11, 4, 15, 26, 13, 18, -27],
    [-24, 1, 21, 12, 31, 28, 15, -18],
    [-8, 0, 30, 59, 25, 59, -1, 6],
    [-21, 29, 41, 47, 106, 81, 45, -24],
    [-64, -25, 23, 31, -14, 85, -23, -47],
    [-169, -48, -39, -41, -31, -51, -40, -140],
];
pub const PSQT_KNIGHT_EG: [[i16; 8]; 8] = [
    [-58, -51, -24, -18, -18, -30, -45, -57],
    [-30, -28, -6, -6, -8, -9, -13, -11],
    [-18, 8, 13, 37, 27, 18, 2, -12],
    [-12, 12, 32, 41, 42, 22, 9, -12],
    [-2, 8, 30, 35, 25, 31, 0, -19],
    [-22, 3, 38, 29, 12, 15, -3, -30],
    [-48, -9, -3, -1, -31, -8, -14, -57],
    [-150, -53, -41, -35, -28, -51, -45, -140],
];
pub const PSQT_BISHOP_MG: [[i16; 8]; 8] = [
    [1, 23, -9, -4, -6, -24, 8, 0],
    [-1, 10, 18, -9, 3, 16, 26, 10],
    [-2, 20, 13, 5, 4, 16, 23, 13],
    [-6, -13, -10, 23, 19, 1, -2, 5],
    [-25, -6, 9, 19, 31, 9, -3, -43],
    [-13, 7, 17, 37, 10, 45, 21, 18],
    [-45, -15, -14, -6, -11, -15, -28, -45],
    [-54, -25, -18, -49, -46, -27, -18, -70],
];
pub const PSQT_BISHOP_EG: [[i16; 8]; 8] = [
    [-35, -35, -28, -18, -6, -10, -27, -31],
    [-28, -20, -27, -6, -7, -17, -4, -44],
    [-17, 28, 20, 18, 26, 26, 10, -3],
    [-3, 1, 22, 23, 16, 13, -4, -23],
    [-2, 12, 5, 28, 13, 5, -11, -11],
    [-3, 1, 8, -6, -12, 8, -10, -18],
    [-24, -8, -7, -9, -16, -20, -25, -43],
    [-28, -10, -20, -22, -33, -26, -27, -46],
];
pub const PSQT_KING_MG: [[i16; 8]; 8] = [
    [69, 108, 94, -26, 35, -15, 78, 85],
    [87, 65, 33, -15, -20, 14, 80, 91],
    [-37, -19, -67, -88, -91, -70, -12, -38],
    [-63, -83, -73, -102, -105, -84, -90, -78],
    [-69, -86, -84, -104, -103, -87, -90, -71],
    [-60, -80, -80, -100, -100, -80, -80, -61],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
];
pub const PSQT_KING_EG: [[i16; 8]; 8] = [
    [-124, -97, -80, -76, -106, -69, -90, -152],
    [-70, -38, -22, -19, -16, -23, -48, -89],
    [-56, -11, 9, 21, 22, 6, -13, -53],
    [-48, 4, 40, 55, 46, 26, -1, -65],
    [-39, 32, 59, 77, 75, 60, 32, -39],
    [-24, 60, 68, 79, 77, 73, 66, -23],
    [-32, 35, 51, 30, 38, 51, 43, -27],
    [-77, -49, -41, -21, -19, -23, -45, -64],
];
