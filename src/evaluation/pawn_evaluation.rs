use super::{bitboards,VERBOSE};
const PAWN_PIECE_VALUE_MG: f64 = 100.0;
const PAWN_PIECE_VALUE_EG: f64 = 150.0;
const PAWN_DOUBLED_VALUE_MG: f64 = -8.0;
const PAWN_DOUBLED_VALUE_EG: f64 = -37.5;
const PAWN_ISOLATED_VALUE_MG: f64 = -5.0;
const PAWN_ISOLATED_VALUE_EG: f64 = -15.0;
const PAWN_BACKWARD_VALUE_MG: f64 = -10.0;
const PAWN_BACKWARD_VALUE_EG: f64 = -25.0;
const PAWN_PASSED_VALUES_MG: [f64; 7] = [0.0, -20.0, -10.0, 10.0, 70.0, 120.0, 200.0];
const PAWN_PASSED_NOT_BLOCKED_VALUES_MG: [f64; 7] = [0.0, 0.0, 0.0, 25.0, 40.0, 130.0, 210.0];
const PAWN_PASSED_VALUES_EG: [f64; 7] = [0.0, -40.0, -20.0, 20.0, 140.0, 240.0, 400.0];
const PAWN_PASSED_NOT_BLOCKED_VALUES_EG: [f64; 7] = [0.0, 0.0, 0.0, 50.0, 80.0, 260.0, 420.0];

const PSQT_PAWN_MG: [[f64; 8]; 8] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [-5.0, -5.0, -5.0, -5.0, -5.0, -5.0, -5.0, -5.0],
    [-7.0, 3.0, 6.0, 10.0, 10.0, 6.0, 3.0, -7.0],
    [-14.0, -7.0, 15.0, 20.0, 20.0, 15.0, -7.0, -14.0],
    [-10.0, -2.0, 1.0, 12.0, 12.0, 1.0, -2.0, -10.0],
    [-7.0, -1.0, 0.0, 5.0, 5.0, 0.0, -1.0, -7.0],
    [-3.0, 10.0, 5.0, 5.0, 5.0, 5.0, 10.0, -3.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
];
const PSQT_PAWN_EG: [[f64; 8]; 8] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [-20.0, -20.0, -20.0, -20.0, -20.0, -20.0, -20.0, -20.0],
    [-10.0, -10.0, -10.0, -10.0, -10.0, -10.0, -10.0, -10.0],
    [-5.0, -5.0, -5.0, -5.0, -5.0, -5.0, -5.0, -5.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
    [10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
];

pub fn pawn_eval_white(w_pawns: u64, w_pawns_front_span: u64, w_pawn_attack_span: u64, b_pawns_all_front_spans: u64, enemy_pieces: u64, black_pawn_attacks: u64) -> (f64, f64) {
    let file_fill = bitboards::file_fill(w_pawns);
    //Evaluation parameters
    let amount_of_pawns = w_pawns.count_ones();
    let doubled_pawns = pawns_behind_own(w_pawns, w_pawns_front_span);
    let isolated_pawns = isolated_pawns(w_pawns, file_fill);
    let backwards_pawns = w_backwards(w_pawns, w_pawn_attack_span, black_pawn_attacks);
    //Doubled Pawns aren't doubled passed
    let (passed_pawns, passed_not_blocked) = w_passed_pawns(w_pawns & !bitboards::w_rear_span(w_pawns), b_pawns_all_front_spans, enemy_pieces);
    (pawns_mg_linear_combination(w_pawns, amount_of_pawns, doubled_pawns, isolated_pawns, backwards_pawns, passed_pawns, passed_not_blocked, true),
     pawns_eg_linear_combination(w_pawns, amount_of_pawns, doubled_pawns, isolated_pawns, backwards_pawns, passed_pawns, passed_not_blocked, true))
}

pub fn pawn_eval_black(b_pawns: u64, b_pawns_front_span: u64, b_pawns_attack_span: u64, w_pawns_all_front_spans: u64, enemy_pieces: u64, white_pawn_attacks: u64) -> (f64, f64) {
    let file_fill = bitboards::file_fill(b_pawns);
    let amount_of_pawns = b_pawns.count_ones();
    let doubled_pawns = pawns_behind_own(b_pawns, b_pawns_front_span);
    let isolated_pawns = isolated_pawns(b_pawns, file_fill);
    let backwards_pawns = b_backwards(b_pawns, b_pawns_attack_span, white_pawn_attacks);
    //Doubled Pawns aren't doubled passed
    let (passed_pawns, passed_not_blocked) = b_passed_pawns(b_pawns & !bitboards::b_rear_span(b_pawns), w_pawns_all_front_spans, enemy_pieces);
    (pawns_mg_linear_combination(b_pawns, amount_of_pawns, doubled_pawns, isolated_pawns, backwards_pawns, passed_pawns, passed_not_blocked, false),
     pawns_eg_linear_combination(b_pawns, amount_of_pawns, doubled_pawns, isolated_pawns, backwards_pawns, passed_pawns, passed_not_blocked, false))
}

pub fn pawns_mg_linear_combination(mut pawns: u64, amount_of_pawns: u32, doubled_pawns: u32, isolated_pawns: u32, backwards_pawns: u32, mut passed_pawns: u64, mut passed_not_blocked: u64, is_white: bool) -> f64 {
    let mut res: f64 = amount_of_pawns as f64 * PAWN_PIECE_VALUE_MG + doubled_pawns as f64 * PAWN_DOUBLED_VALUE_MG + isolated_pawns as f64 * PAWN_ISOLATED_VALUE_MG +
        backwards_pawns as f64 * PAWN_BACKWARD_VALUE_MG;
    let passed_pawns_amt = passed_pawns.count_ones();
    let passed_pawns_nb_amt = passed_not_blocked.count_ones();
    let mut passer_score = 0.0;
    while passed_pawns != 0u64 {
        let idx = passed_pawns.trailing_zeros() as usize;
        passer_score += PAWN_PASSED_VALUES_MG[if is_white { idx / 8 } else { 7 - idx / 8 }];
        passed_pawns ^= 1u64 << idx;
    }
    let mut passer_not_blocked = 0.0;
    while passed_not_blocked != 0u64 {
        let idx = passed_not_blocked.trailing_zeros() as usize;
        passer_not_blocked += PAWN_PASSED_NOT_BLOCKED_VALUES_MG[if is_white { idx / 8 } else { 7 - idx / 8 }];
        passed_not_blocked ^= 1u64 << idx;
    }
    res += passer_score;
    res += passer_not_blocked;
    //PSQT
    let mut psqt = 0.0;
    while pawns != 0u64 {
        let mut idx = pawns.trailing_zeros() as usize;
        pawns ^= 1u64 << idx;
        if !is_white {
            idx = 63 - idx;
        }
        psqt += PSQT_PAWN_MG[idx / 8][idx % 8];
    }
    res += psqt;
    if VERBOSE {
        println!("------------------------------------------------");
        println!("\tPawns MidGame --{}", if is_white { "white" } else { "black" });
        println!("\t\tAmount of Pawns:         \t{} -> {}", amount_of_pawns, amount_of_pawns as f64 * PAWN_PIECE_VALUE_MG);
        println!("\t\tDoubled Pawns:           \t{} -> {}", doubled_pawns, doubled_pawns as f64 * PAWN_DOUBLED_VALUE_MG);
        println!("\t\tIsolated Pawns:          \t{} -> {}", isolated_pawns, isolated_pawns as f64 * PAWN_ISOLATED_VALUE_MG);
        println!("\t\tBackwards Pawns:         \t{} -> {}", backwards_pawns, backwards_pawns as f64 * PAWN_BACKWARD_VALUE_MG);
        println!("\t\tPassed Pawns:            \t{} -> {}", passed_pawns_amt, passer_score);
        println!("\t\tNot Blocked Passed Pawns:\t{} -> {}", passed_pawns_nb_amt, passer_not_blocked);
        println!("\t\tPSQT-Value:              \t{}", psqt);
        println!("\tSum: {}", res);
        println!("------------------------------------------------");
    }
    res
}

pub fn pawns_eg_linear_combination(mut pawns: u64, amount_of_pawns: u32, doubled_pawns: u32, isolated_pawns: u32, backwards_pawns: u32, mut passed_pawns: u64, mut passed_not_blocked: u64, is_white: bool) -> f64 {
    let mut res: f64 = amount_of_pawns as f64 * PAWN_PIECE_VALUE_EG + doubled_pawns as f64 * PAWN_DOUBLED_VALUE_EG + isolated_pawns as f64 * PAWN_ISOLATED_VALUE_EG +
        backwards_pawns as f64 * PAWN_BACKWARD_VALUE_EG;
    let passed_pawns_amt = passed_pawns.count_ones();
    let passed_pawns_nb_amt = passed_not_blocked.count_ones();
    let mut passer_score = 0.0;
    let mut passer_not_blocked = 0.0;
    while passed_pawns != 0u64 {
        let idx = passed_pawns.trailing_zeros() as usize;
        passer_score += PAWN_PASSED_VALUES_EG[if is_white { idx / 8 } else { 7 - idx / 8 }];
        passed_pawns ^= 1u64 << idx;
    }
    while passed_not_blocked != 0u64 {
        let idx = passed_not_blocked.trailing_zeros() as usize;
        passer_not_blocked += PAWN_PASSED_NOT_BLOCKED_VALUES_EG[if is_white { idx / 8 } else { 7 - idx / 8 }];
        passed_not_blocked ^= 1u64 << idx;
    }
    res += passer_score;
    res += passer_not_blocked;
    let mut psqt = 0.0;
    while pawns != 0u64 {
        let mut idx = pawns.trailing_zeros() as usize;
        pawns ^= 1u64 << idx;
        if !is_white {
            idx = 63 - idx;
        }
        psqt += PSQT_PAWN_EG[idx / 8][idx % 8];
    }
    res += psqt;
    if VERBOSE {
        println!("------------------------------------------------");
        println!("\tPawns EndGame --{}", if is_white { "white" } else { "black" });
        println!("\t\tAmount of Pawns:         \t{} -> {}", amount_of_pawns, amount_of_pawns as f64 * PAWN_PIECE_VALUE_EG);
        println!("\t\tDoubled Pawns:           \t{} -> {}", doubled_pawns, doubled_pawns as f64 * PAWN_DOUBLED_VALUE_EG);
        println!("\t\tIsolated Pawns:          \t{} -> {}", isolated_pawns, isolated_pawns as f64 * PAWN_ISOLATED_VALUE_EG);
        println!("\t\tBackwards Pawns:         \t{} -> {}", backwards_pawns, backwards_pawns as f64 * PAWN_BACKWARD_VALUE_EG);
        println!("\t\tPassed Pawns:            \t{} -> {}", passed_pawns_amt, passer_score);
        println!("\t\tNot Blocked Passed Pawns:\t{} -> {}", passed_pawns_nb_amt, passer_not_blocked);
        println!("\t\tPSQT-Value:              \t{}", psqt);
        println!("\tSum: {}", res);
        println!("------------------------------------------------");
    }
    res
}

pub fn w_backwards(w_pawns: u64, w_pawn_attack_span: u64,black_pawn_attacks:u64) -> u32 {
    let stops = w_pawns << 8;
    (stops & black_pawn_attacks & !w_pawn_attack_span).count_ones()
}

pub fn b_backwards(b_pawns: u64, b_pawn_attack_span: u64,white_pawn_attacks:u64) -> u32 {
    let stops = b_pawns >> 8;
    (stops & white_pawn_attacks & !b_pawn_attack_span).count_ones()
}

pub fn pawns_behind_own(pawns: u64, front_span: u64) -> u32 {
    (pawns & front_span).count_ones()
}

pub fn isolated_pawns(pawns: u64, file_fill: u64) -> u32 {
    (pawns & !bitboards::west_one(file_fill) & !bitboards::east_one(file_fill)).count_ones()
}

pub fn w_passed_pawns(w_pawns: u64, b_pawns_all_front_spans: u64, enemy_pieces: u64) -> (u64, u64) {
    let mut passed_board = w_pawns & !b_pawns_all_front_spans;
    let passed_board_cl = passed_board.clone();
    let mut passed_not_blocked = 0u64;
    while passed_board != 0u64 {
        let idx = passed_board.trailing_zeros() as usize;
        let piece = 1u64 << idx;
        if bitboards::w_front_span(piece) & enemy_pieces == 0u64 {
            passed_not_blocked |= piece;
        }
        passed_board ^= piece;
    }
    (passed_board_cl, passed_not_blocked)
}

pub fn b_passed_pawns(b_pawns: u64, w_pawns_all_front_spans: u64, enemy_pieces: u64) -> (u64, u64) {
    let mut passed_board = b_pawns & !w_pawns_all_front_spans;
    let passed_board_cl = passed_board.clone();
    let mut passed_not_blocked = 0u64;
    while passed_board != 0u64 {
        let idx = passed_board.trailing_zeros() as usize;
        let piece = 1u64 << idx;
        if bitboards::b_front_span(piece) & enemy_pieces == 0u64 {
            passed_not_blocked |= piece;
        }
        passed_board ^= piece;
    }
    (passed_board_cl, passed_not_blocked)
}