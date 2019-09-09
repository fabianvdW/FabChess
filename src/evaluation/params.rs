pub const TEMPO_BONUS_MG: i16 = 30;
pub const TEMPO_BONUS_EG: i16 = 46;
pub const SHIELDING_PAWN_MISSING_MG: [i16; 4] = [4, -38, -62, -87];
pub const SHIELDING_PAWN_MISSING_EG: [i16; 4] = [-9, 3, 3, 2];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG: [i16; 4] = [-39, -47, -103, -173];
pub const SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG: [i16; 4] = [-9, -3, 9, 2];
pub const PAWN_DOUBLED_VALUE_MG: i16 = 6;
pub const PAWN_DOUBLED_VALUE_EG: i16 = -18;
pub const PAWN_ISOLATED_VALUE_MG: i16 = -15;
pub const PAWN_ISOLATED_VALUE_EG: i16 = -14;
pub const PAWN_BACKWARD_VALUE_MG: i16 = -13;
pub const PAWN_BACKWARD_VALUE_EG: i16 = -17;
pub const PAWN_SUPPORTED_VALUE_MG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [7, 10, 24, 10, 20, -7, 38, 17],
    [4, 11, 11, 13, 13, 3, 7, 12],
    [-8, 2, 12, 5, 27, 31, -4, -16],
    [-13, -12, 49, 71, 73, 62, 15, -14],
    [15, 23, 24, 28, 28, 23, 20, 15],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PAWN_SUPPORTED_VALUE_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [11, 21, 24, 28, 37, 20, 16, 18],
    [-2, 10, 7, 29, 16, 2, 0, -7],
    [-5, 5, 27, 34, 35, 9, 8, -3],
    [12, 26, 48, 91, 95, 64, 28, 30],
    [16, 22, 26, 34, 37, 29, 19, 12],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PAWN_ATTACK_CENTER_MG: i16 = -18;
pub const PAWN_ATTACK_CENTER_EG: i16 = -15;
pub const PAWN_MOBILITY_MG: i16 = 6;
pub const PAWN_MOBILITY_EG: i16 = 17;
pub const PAWN_PASSED_VALUES_MG: [i16; 7] = [0, -10, -19, -5, 29, 43, 81];
pub const PAWN_PASSED_VALUES_EG: [i16; 7] = [0, -1, 17, 49, 89, 132, 229];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_MG: [i16; 7] = [0, 7, 0, -18, -27, -13, 82];
pub const PAWN_PASSED_NOT_BLOCKED_VALUES_EG: [i16; 7] = [0, 25, 16, 51, 109, 256, 319];
pub const ROOK_BEHIND_SUPPORT_PASSER_MG: i16 = 13;
pub const ROOK_BEHIND_SUPPORT_PASSER_EG: i16 = 0;
pub const ROOK_BEHIND_ENEMY_PASSER_MG: i16 = 6;
pub const ROOK_BEHIND_ENEMY_PASSER_EG: i16 = -110;
pub const PAWN_PASSED_WEAK_MG: i16 = -16;
pub const PAWN_PASSED_WEAK_EG: i16 = -39;
pub const KNIGHT_SUPPORTED_BY_PAWN_MG: i16 = 1;
pub const KNIGHT_SUPPORTED_BY_PAWN_EG: i16 = 6;
pub const KNIGHT_OUTPOST_MG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-7, 2, 17, -3, 8, 2, 1, 7],
    [34, 31, 49, 45, 58, 55, 20, 42],
    [16, 50, 60, 67, 75, 70, 73, 11],
    [11, 6, 39, 31, 36, 57, 23, 18],
    [15, 19, 19, 25, 35, 20, 18, 16],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const KNIGHT_OUTPOST_EG_TABLE: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [8, 8, 17, 8, 12, 3, 3, -2],
    [11, 22, 36, 39, 37, 34, 26, 17],
    [17, 28, 37, 48, 55, 51, 37, 17],
    [10, 9, 32, 56, 48, 41, 22, 11],
    [15, 22, 20, 25, 28, 18, 24, 15],
    [0, 1, 0, 0, 1, 0, 1, 0],
];
pub const ROOK_ON_OPEN_FILE_BONUS_MG: i16 = 45;
pub const ROOK_ON_OPEN_FILE_BONUS_EG: i16 = 33;
pub const ROOK_ON_SEVENTH_MG: i16 = 0;
pub const ROOK_ON_SEVENTH_EG: i16 = 42;
pub const PAWN_PIECE_VALUE_MG: i16 = 107;
pub const PAWN_PIECE_VALUE_EG: i16 = 188;
pub const KNIGHT_PIECE_VALUE_MG: i16 = 438;
pub const KNIGHT_PIECE_VALUE_EG: i16 = 713;
pub const KNIGHT_VALUE_WITH_PAWNS: [i16; 17] = [
    -41, -112, -48, -38, -31, -21, -6, 0, 17, 22, 30, 36, 43, 49, 48, 55, 53,
];
pub const BISHOP_PIECE_VALUE_MG: i16 = 488;
pub const BISHOP_PIECE_VALUE_EG: i16 = 697;
pub const BISHOP_PAIR_BONUS_MG: i16 = 32;
pub const BISHOP_PAIR_BONUS_EG: i16 = 114;
pub const ROOK_PIECE_VALUE_MG: i16 = 669;
pub const ROOK_PIECE_VALUE_EG: i16 = 1240;
pub const QUEEN_PIECE_VALUE_MG: i16 = 1541;
pub const QUEEN_PIECE_VALUE_EG: i16 = 2332;
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG: [i16; 5] = [-16, -16, -25, -4, -100];
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG: [i16; 5] = [56, 56, 38, 14, -100];
pub const KNIGHT_MOBILITY_BONUS_MG: [i16; 9] = [-46, -18, -9, -2, 1, 1, -1, -4, -3];
pub const KNIGHT_MOBILITY_BONUS_EG: [i16; 9] = [-69, 1, 6, 12, 23, 37, 40, 44, 40];
pub const BISHOP_MOBILITY_BONUS_MG: [i16; 14] =
    [-11, 3, 13, 21, 29, 39, 45, 49, 48, 47, 50, 48, 66, 92];
pub const BISHOP_MOBILITY_BONUS_EG: [i16; 14] =
    [-30, -22, -8, 3, 31, 51, 56, 64, 71, 68, 60, 60, 47, 47];
pub const ROOK_MOBILITY_BONUS_MG: [i16; 15] =
    [-25, -10, -5, 4, -1, 7, 16, 25, 22, 22, 24, 21, 19, 17, 18];
pub const ROOK_MOBILITY_BONUS_EG: [i16; 15] =
    [-30, 2, 7, 10, 24, 37, 34, 39, 54, 63, 69, 75, 76, 78, 73];
pub const QUEEN_MOBILITY_BONUS_MG: [i16; 28] = [
    -20, 4, -5, 4, 6, 12, 18, 16, 18, 20, 21, 16, 18, 16, 14, 12, 12, 10, 13, 14, 25, 25, 29, 45,
    50, 59, 56, 55,
];
pub const QUEEN_MOBILITY_BONUS_EG: [i16; 28] = [
    -40, -37, -35, -39, -44, -49, -43, -10, 3, 10, 25, 52, 55, 72, 85, 100, 109, 117, 127, 124,
    130, 136, 125, 118, 89, 98, 89, 88,
];
pub const ATTACK_WEIGHT_MG: [i16; 8] = [0, 60, 119, 187, 225, 114, 100, 100];
pub const SAFETY_TABLE_MG: [i16; 100] = [
    0, 0, 1, 2, 3, 8, 15, 13, 12, 15, 19, 25, 31, 36, 37, 40, 45, 52, 58, 64, 70, 76, 83, 86, 90,
    98, 106, 114, 123, 132, 140, 151, 169, 180, 191, 202, 213, 225, 237, 248, 260, 272, 283, 295,
    307, 319, 330, 342, 354, 366, 377, 389, 401, 412, 424, 436, 448, 459, 471, 483, 494, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
];
pub const ATTACK_WEIGHT_EG: [i16; 8] = [0, 43, 79, 144, 202, 110, 100, 100];
pub const SAFETY_TABLE_EG: [i16; 100] = [
    0, -1, 1, 2, 4, 6, 8, 9, 12, 16, 19, 23, 27, 30, 35, 39, 44, 50, 56, 62, 68, 75, 82, 85, 89,
    97, 105, 113, 122, 131, 140, 150, 169, 180, 191, 202, 213, 225, 237, 248, 260, 272, 283, 295,
    307, 319, 330, 342, 354, 366, 377, 389, 401, 412, 424, 436, 448, 459, 471, 483, 494, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
];
pub const KNIGHT_ATTACK_WORTH_MG: i16 = 7;
pub const KNIGHT_ATTACK_WORTH_EG: i16 = 3;
pub const BISHOP_ATTACK_WORTH_MG: i16 = 7;
pub const BISHOP_ATTACK_WORTH_EG: i16 = 1;
pub const ROOK_ATTACK_WORTH_MG: i16 = 6;
pub const ROOK_ATTACK_WORTH_EG: i16 = 3;
pub const QUEEN_ATTACK_WORTH_MG: i16 = 6;
pub const QUEEN_ATTACK_WORTH_EG: i16 = 10;
pub const PSQT_PAWN_MG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-43, -44, -35, -44, -26, -25, 6, -34],
    [-37, -41, -12, -1, 3, -5, -25, -25],
    [-35, -31, 9, 29, 33, 28, -2, -13],
    [-28, -11, -4, 10, 31, 19, 18, 4],
    [-9, 6, 38, 35, 62, 113, 54, 20],
    [3, 1, 26, 32, 35, 8, -18, -60],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_PAWN_EG: [[i16; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-21, -30, -31, -37, -14, -18, -33, -36],
    [-9, -18, -15, -15, -11, 0, -16, -17],
    [7, -9, -7, -19, -10, -2, -7, -8],
    [36, 6, -21, -43, -40, -19, 1, 12],
    [61, 48, 4, -48, -58, -4, 42, 38],
    [90, 57, 45, -9, -15, 6, 23, 32],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
pub const PSQT_KNIGHT_MG: [[i16; 8]; 8] = [
    [-67, -49, -43, -26, -28, -17, -45, -64],
    [-56, -43, -23, -3, -8, -1, -17, -23],
    [-53, -11, 4, 15, 26, 13, 19, -27],
    [-24, 1, 21, 12, 31, 28, 15, -18],
    [-8, 0, 30, 59, 25, 59, -2, 5],
    [-21, 29, 41, 46, 105, 81, 44, -24],
    [-64, -25, 23, 30, -14, 84, -23, -47],
    [-167, -48, -39, -41, -31, -51, -40, -139],
];
pub const PSQT_KNIGHT_EG: [[i16; 8]; 8] = [
    [-58, -51, -24, -18, -18, -30, -45, -57],
    [-30, -28, -6, -5, -8, -9, -13, -11],
    [-18, 8, 14, 37, 27, 18, 2, -12],
    [-12, 12, 32, 41, 42, 22, 9, -12],
    [-2, 8, 30, 34, 24, 31, -1, -19],
    [-22, 3, 38, 27, 11, 15, -3, -30],
    [-48, -9, -3, -2, -31, -8, -14, -57],
    [-149, -53, -41, -35, -28, -51, -45, -139],
];
pub const PSQT_BISHOP_MG: [[i16; 8]; 8] = [
    [1, 23, -9, -4, -6, -24, 8, 0],
    [-1, 9, 18, -9, 3, 17, 27, 10],
    [-2, 20, 14, 5, 5, 17, 24, 14],
    [-6, -13, -9, 23, 19, 2, -2, 5],
    [-25, -6, 9, 18, 30, 9, -3, -44],
    [-13, 7, 17, 36, 9, 45, 21, 16],
    [-45, -15, -14, -6, -11, -15, -28, -45],
    [-54, -25, -18, -49, -46, -27, -18, -70],
];
pub const PSQT_BISHOP_EG: [[i16; 8]; 8] = [
    [-35, -35, -28, -18, -6, -9, -27, -31],
    [-28, -20, -27, -5, -6, -17, -3, -44],
    [-17, 27, 20, 18, 27, 26, 10, -3],
    [-3, 1, 21, 22, 15, 13, -4, -23],
    [-2, 12, 4, 27, 12, 4, -11, -11],
    [-3, 1, 8, -7, -13, 8, -10, -18],
    [-24, -8, -7, -9, -16, -20, -25, -43],
    [-28, -10, -20, -22, -33, -26, -27, -46],
];
pub const PSQT_KING_MG: [[i16; 8]; 8] = [
    [69, 109, 95, -26, 36, -17, 79, 87],
    [87, 65, 32, -16, -21, 13, 80, 92],
    [-37, -20, -67, -88, -91, -71, -13, -38],
    [-63, -83, -73, -102, -105, -84, -90, -77],
    [-69, -86, -84, -104, -103, -87, -90, -71],
    [-60, -80, -80, -100, -100, -80, -80, -61],
    [-60, -80, -80, -100, -100, -80, -80, -60],
    [-60, -80, -80, -100, -100, -80, -80, -60],
];
pub const PSQT_KING_EG: [[i16; 8]; 8] = [
    [-123, -96, -80, -76, -106, -70, -90, -150],
    [-69, -38, -22, -19, -16, -24, -47, -87],
    [-56, -11, 9, 21, 22, 6, -13, -53],
    [-48, 4, 40, 55, 46, 26, -1, -65],
    [-39, 32, 59, 77, 74, 59, 32, -39],
    [-24, 60, 68, 78, 76, 72, 66, -23],
    [-32, 35, 51, 30, 38, 51, 43, -27],
    [-77, -49, -41, -21, -19, -23, -45, -64],
];
