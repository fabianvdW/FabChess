use super::{bitboards, VERBOSE, Evaluation, MidGameDisplay, EndGameDisplay};

const PSQT_KING_MG: [[f64; 8]; 8] = [
    [40.0, 60.0, 20.0, 0.0, 0.0, 20.0, 60.0, 40.0],
    [40.0, 40.0, 0.0, 0.0, 0.0, 0.0, 20.0, 20.0],
    [-20.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -20.0],
    [-40.0, -60.0, -60.0, -80.0, -80.0, -60.0, -60.0, -40.0],
    [-60.0, -80.0, -80.0, -100.0, -100.0, -80.0, -80.0, -60.0],
    [-60.0, -80.0, -80.0, -100.0, -100.0, -80.0, -80.0, -60.0],
    [-60.0, -80.0, -80.0, -100.0, -100.0, -80.0, -80.0, -60.0],
    [-60.0, -80.0, -80.0, -100.0, -100.0, -80.0, -80.0, -60.0],
];
const PSQT_KING_EG: [[f64; 8]; 8] = [
    [-100.0, -60.0, -60.0, -60.0, -60.0, -60.0, -60.0, -100.0],
    [-60.0, -60.0, 0.0, 0.0, 0.0, 0.0, -60.0, -60.0],
    [-60.0, -20.0, 40.0, 60.0, 60.0, 40.0, -20.0, -60.0],
    [-60.0, -20.0, 60.0, 80.0, 80.0, 60.0, -20.0, -60.0],
    [-60.0, -20.0, 60.0, 80.0, 80.0, 60.0, -20.0, -60.0],
    [-60.0, -20.0, 40.0, 60.0, 60.0, 40.0, -20.0, -60.0],
    [-60.0, -40.0, -20.0, 0.0, 0.0, -20.0, -40.0, -60.0],
    [-100.0, -80.0, -60.0, -40.0, -40.0, -60.0, -80.0, -100.0]
];
const SHIELDING_PAWN_MISSING_MG: f64 = -20.0;
const SHIELDING_PAWN_MISSING_ON_OPEN_FILE: f64 = -40.0;

pub struct KingEvaluation {
    shielding_pawns_missing: u32,
    shielding_pawns_missing_on_open_file: u32,
}

impl Evaluation for KingEvaluation {
    fn eval_mg(&self) -> f64 {
        let mut res = 0.0;
        res += self.shielding_pawns_missing as f64 * SHIELDING_PAWN_MISSING_MG;
        res += self.shielding_pawns_missing_on_open_file as f64 * SHIELDING_PAWN_MISSING_ON_OPEN_FILE;
        res
    }
    fn eval_eg(&self) -> f64 {
        0.0
    }
}

impl MidGameDisplay for KingEvaluation {
    fn display(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tKing-MidGame");
        println!("\t\tShielding Pawns missing:              {} -> {}", self.shielding_pawns_missing, self.shielding_pawns_missing as f64 * SHIELDING_PAWN_MISSING_MG);
        println!("\t\tShielding Pawns on open file missing: {} -> {}", self.shielding_pawns_missing_on_open_file, self.shielding_pawns_missing_on_open_file as f64 * SHIELDING_PAWN_MISSING_ON_OPEN_FILE);
        println!("\tSum: {}", self.eval_mg());
        res_str
    }
}

impl EndGameDisplay for KingEvaluation {
    fn display(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("\tKing-EndGame");
        println!("\tSum: {}", self.eval_eg());
        res_str
    }
}

pub fn king_eval(king: u64, my_pawns: u64, enemy_pawns: u64, is_white: bool) -> KingEvaluation {
    //Missing psqt
    let king_index = king.trailing_zeros() as usize;
    let mut shield = if is_white { bitboards::SHIELDING_PAWNS_WHITE[king_index] } else { bitboards::SHIELDING_PAWNS_BLACK[king_index] };
    let mut shields_missing = 0;
    let mut shields_on_open_missing = 0;
    while shield != 0u64 {
        let idx = shield.trailing_zeros() as usize;
        //Block out whole file
        let file = bitboards::FILES[idx % 8];
        if my_pawns & shield & file == 0u64 {
            shields_missing += 1;
            if enemy_pawns & file == 0u64 {
                shields_on_open_missing += 1;
            }
        }
        shield &= !file;
    }
    KingEvaluation {
        shielding_pawns_missing: shields_missing,
        shielding_pawns_missing_on_open_file: shields_on_open_missing,
    }
}