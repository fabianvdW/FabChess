extern crate core;
extern crate rand;

pub mod loading;

pub use crate::loading::{FileFormatSupported, LabelledGameState, Statistics};
use core_sdk::board_representation::game_state::{BLACK, WHITE};
pub use core_sdk::evaluation::parameters::{normal_parameters::*, special_parameters::*, *};
use core_sdk::evaluation::trace::CollapsedTrace;
use rand::{seq::SliceRandom, thread_rng};

pub const POSITION_FILE: &str = "D:/Users/fabia/Schach/TuningData/E12.41-1M-D12-Resolved.epd";
pub const PARAM_FILE: &str = "D:/Users/fabia/Schach/TuningData/E12.41-1M-D12-Resolved";
//Override for all others if true
pub const TUNE_ALL: bool = true;

pub const TUNE_TEMPO_BONUS: bool = true;
pub const TUNE_SHIELDING_PAWNS: bool = true;
pub const TUNE_PAWNS: bool = true;
//Category passed pawns
pub const TUNE_PASSED: bool = true;
pub const TUNE_PASSED_PAWN: bool = true;
pub const TUNE_PASSED_PAWN_NOT_BLOCKED: bool = true;

pub const TUNE_KNIGHTS: bool = true;
pub const TUNE_FILES: bool = true;

pub const TUNE_PIECE_VALUES: bool = true;
pub const TUNE_MOBILITY: bool = true;

pub const TUNE_ATTACK: bool = true;
pub const TUNE_ATTACK_INDEX: bool = true;
pub const TUNE_PSQT: bool = true;

pub const TUNABLE_PARAM: [bool; NORMAL_PARAMS] = init_tunable_param();

pub const OPTIMIZE_K: bool = false;
pub const BATCH_SIZE: usize = 10000000;
pub const START_LEARNING_RATE: f32 = 2.;
pub const L1_REGULARIZATION: f32 = 0.;
pub const L2_REGULARIZATION: f32 = 0.;

pub const fn init_tunable_param() -> [bool; NORMAL_PARAMS] {
    let mut res = [false; NORMAL_PARAMS];
    if TUNE_ALL {
        res = [true; NORMAL_PARAMS];
    } else {
        if TUNE_SHIELDING_PAWNS {
            let mut i = 0;
            while i < SIZE_SHIELDING_PAWN_MISSING {
                res[IDX_SHIELDING_PAWN_MISSING + i] = true;
                i += 1;
            }
            i = 0;
            while i < SIZE_SHIELDING_PAWN_ONOPEN_MISSING {
                res[IDX_SHIELDING_PAWN_ONOPEN_MISSING + i] = true;
                i += 1;
            }
        }
        if TUNE_PAWNS {
            res[IDX_PAWN_DOUBLED] = true;
            res[IDX_PAWN_ISOLATED] = true;
            res[IDX_PAWN_BACKWARD] = true;
            res[IDX_PAWN_ATTACK_CENTER] = true;
            res[IDX_PAWN_MOBILITY] = true;
            let mut i = 0;
            while i < SIZE_PAWN_SUPPORTED {
                res[IDX_PAWN_SUPPORTED + i] = true;
                i += 1;
            }
        }
        if TUNE_PASSED {
            let mut i = 0;
            while i < SIZE_PAWN_PASSED {
                res[IDX_PAWN_PASSED + i] = true;
                i += 1;
            }
            i = 0;
            while i < SIZE_PAWN_PASSED_NOTBLOCKED {
                res[IDX_PAWN_PASSED_NOTBLOCKED + i] = true;
                i += 1;
            }
            i = 0;
            while i < SIZE_PAWN_PASSED_KINGDISTANCE {
                res[IDX_PAWN_PASSED_KINGDISTANCE + i] = true;
                i += 1;
            }
            i = 0;
            while i < SIZE_PAWN_PASSED_ENEMYKINGDISTANCE {
                res[IDX_PAWN_PASSED_ENEMYKINGDISTANCE + i] = true;
                i += 1;
            }
            i = 0;
            while i < SIZE_PAWN_PASSED_SUBDISTANCE {
                res[IDX_PAWN_PASSED_SUBDISTANCE + i] = true;
                i += 1;
            }
            res[IDX_ROOK_BEHIND_SUPPORT_PASSER] = true;
            res[IDX_ROOK_BEHIND_ENEMY_PASSER] = true;
            res[IDX_PAWN_PASSED_WEAK] = true;
        }
        if TUNE_KNIGHTS {
            res[IDX_KNIGHT_SUPPORTED] = true;
            let mut i = 0;
            while i < SIZE_KNIGHT_OUTPOST_TABLE {
                res[IDX_KNIGHT_OUTPOST_TABLE + i] = true;
                i += 1;
            }
        }
        if TUNE_FILES {
            res[IDX_ROOK_ON_OPEN] = true;
            res[IDX_ROOK_ON_SEMI_OPEN] = true;
            res[IDX_QUEEN_ON_OPEN] = true;
            res[IDX_QUEEN_ON_SEMI_OPEN] = true;
            res[IDX_ROOK_ON_SEVENTH] = true;
        }
        if TUNE_PIECE_VALUES {
            res[IDX_PAWN_PIECE_VALUE] = true;
            res[IDX_KNIGHT_PIECE_VALUE] = true;
            res[IDX_BISHOP_PIECE_VALUE] = true;
            res[IDX_BISHOP_PAIR] = true;
            res[IDX_ROOK_PIECE_VALUE] = true;
            res[IDX_QUEEN_PIECE_VALUE] = true;
            let mut i = 0;
            while i < SIZE_DIAGONALLY_ADJ_SQ_WPAWNS {
                res[IDX_DIAGONALLY_ADJ_SQ_WPAWNS + i] = true;
                i += 1;
            }
        }
        if TUNE_MOBILITY {
            let mut i = 0;
            while i < SIZE_KNIGHT_MOBILITY {
                res[IDX_KNIGHT_MOBILITY + i] = true;
                i += 1;
            }
            i = 0;
            while i < SIZE_BISHOP_MOBILITY {
                res[IDX_BISHOP_MOBILITY + i] = true;
                i += 1;
            }
            i = 0;
            while i < SIZE_ROOK_MOBILITY {
                res[IDX_ROOK_MOBILITY + i] = true;
                i += 1;
            }
            i = 0;
            while i < SIZE_QUEEN_MOBILITY {
                res[IDX_QUEEN_MOBILITY + i] = true;
                i += 1;
            }
        }
        if TUNE_PSQT {
            let mut i = 0;
            while i < SIZE_PSQT {
                res[IDX_PSQT + i] = true;
                i += 1;
            }
        }
    }
    res[IDX_TEMPO_BONUS] = TUNE_TEMPO_BONUS;
    res
}
pub struct TexelState {
    pub label: f32,
    pub eval: f32,
    pub trace: CollapsedTrace,
}

pub struct Tuner {
    pub k: f32,
    pub positions: Vec<TexelState>,
    pub params: Parameters,
}

pub fn update_evaluations(tuner: &mut Tuner) {
    for pos in tuner.positions.iter_mut() {
        pos.eval = pos.trace.evaluate(&tuner.params);
    }
}

pub fn shuffle_positions(tuner: &mut Tuner) {
    tuner.positions.shuffle(&mut thread_rng());
}

pub fn add_regularization(gradient: &mut Parameters, parameters: &Parameters, portion: f32) {
    for i in 0..gradient.normal[0].len() {
        gradient.normal[0][i] -= portion * regularization(parameters.normal[0][i]);
        gradient.normal[1][i] -= portion * regularization(parameters.normal[1][i]);
    }
    for i in 0..gradient.special.len() {
        gradient.special[i] -= portion * regularization(parameters.special[i]);
    }
}
pub fn regularization(term: f32) -> f32 {
    L1_REGULARIZATION * term.signum() + 2. * L2_REGULARIZATION * term
}

pub fn calculate_gradient(tuner: &mut Tuner, from: usize, to: usize) -> Parameters {
    let mut gradient = Parameters::zero();
    for pos in tuner.positions[from..to].iter_mut() {
        //Step 1. Update evaluation
        pos.eval = pos.trace.evaluate(&tuner.params);
    }
    //let g = tuner.k * 10f32.ln() / 400.0;
    let portion = 2. / (to - from) as f32;
    for pos in tuner.positions[from..to].iter() {
        //Step 2. Calculate first half of gradient
        let s = sigmoid(tuner.k, pos.eval);
        let start_of_gradient = (pos.label - s) * s * (1. - s);
        let devaldmg = pos.trace.phase / 128.0;
        let devaldeg = (1. - pos.trace.phase / 128.0) / 1.5;
        for entry in pos.trace.entries.iter() {
            if TUNABLE_PARAM[entry.0 as usize] {
                gradient.normal[0][entry.0 as usize] +=
                    start_of_gradient * devaldmg * f32::from(entry.1);
                gradient.normal[1][entry.0 as usize] +=
                    start_of_gradient * devaldeg * f32::from(entry.1);
            }
        }

        //Piece values
        if TUNE_PIECE_VALUES || TUNE_ALL {
            let knights = f32::from(pos.trace.knights);
            gradient.special[IDX_KNIGHT_VALUE_WITH_PAWN + pos.trace.pawns_on_board as usize] +=
                start_of_gradient * knights;
        }
        //Safety
        if TUNE_ATTACK {
            for i in 0..2 {
                let devaldg = if i == 0 { devaldmg } else { devaldeg };
                let attack_knight_white = f32::from(pos.trace.knight_attacked_sq[WHITE])
                    * tuner.params.special[IDX_KNIGHT_ATTACK_VALUE + i];
                let attack_bishop_white = f32::from(pos.trace.bishop_attacked_sq[WHITE])
                    * tuner.params.special[IDX_BISHOP_ATTACK_VALUE + i];
                let attack_rook_white = f32::from(pos.trace.rook_attacked_sq[WHITE])
                    * tuner.params.special[IDX_ROOK_ATTACK_VALUE + i];
                let attack_queen_white = f32::from(pos.trace.queen_attacked_sq[WHITE])
                    * tuner.params.special[IDX_QUEEN_ATTACK_VALUE + i];
                let knight_check_white = f32::from(pos.trace.knight_safe_check[WHITE])
                    * tuner.params.special[IDX_KNIGHT_CHECK_VALUE + i];
                let bishop_check_white = f32::from(pos.trace.bishop_safe_check[WHITE])
                    * tuner.params.special[IDX_BISHOP_CHECK_VALUE + i];
                let rook_check_white = f32::from(pos.trace.rook_safe_check[WHITE])
                    * tuner.params.special[IDX_ROOK_CHECK_VALUE + i];
                let queen_check_white = f32::from(pos.trace.queen_safe_check[WHITE])
                    * tuner.params.special[IDX_QUEEN_CHECK_VALUE + i];
                let attacker_value_white = (attack_knight_white
                    + attack_bishop_white
                    + attack_rook_white
                    + attack_queen_white
                    + knight_check_white
                    + bishop_check_white
                    + rook_check_white
                    + queen_check_white)
                    .max(0.)
                    .min(99.);
                let attack_knight_black = f32::from(pos.trace.knight_attacked_sq[BLACK])
                    * tuner.params.special[IDX_KNIGHT_ATTACK_VALUE + i];
                let attack_bishop_black = f32::from(pos.trace.bishop_attacked_sq[BLACK])
                    * tuner.params.special[IDX_BISHOP_ATTACK_VALUE + i];
                let attack_rook_black = f32::from(pos.trace.rook_attacked_sq[BLACK])
                    * tuner.params.special[IDX_ROOK_ATTACK_VALUE + i];
                let attack_queen_black = f32::from(pos.trace.queen_attacked_sq[BLACK])
                    * tuner.params.special[IDX_QUEEN_ATTACK_VALUE + i];
                let knight_check_black = f32::from(pos.trace.knight_safe_check[BLACK])
                    * tuner.params.special[IDX_KNIGHT_CHECK_VALUE + i];
                let bishop_check_black = f32::from(pos.trace.bishop_safe_check[BLACK])
                    * tuner.params.special[IDX_BISHOP_CHECK_VALUE + i];
                let rook_check_black = f32::from(pos.trace.rook_safe_check[BLACK])
                    * tuner.params.special[IDX_ROOK_CHECK_VALUE + i];
                let queen_check_black = f32::from(pos.trace.queen_safe_check[BLACK])
                    * tuner.params.special[IDX_QUEEN_CHECK_VALUE + i];
                let attacker_value_black = (attack_knight_black
                    + attack_bishop_black
                    + attack_rook_black
                    + attack_queen_black
                    + knight_check_black
                    + bishop_check_black
                    + rook_check_black
                    + queen_check_black)
                    .max(0.)
                    .min(99.);
                gradient.special
                    [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i] +=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.special
                            [IDX_SAFETY_TABLE + 2 * attacker_value_white as usize + i];
                gradient.special[IDX_SAFETY_TABLE + 2 * attacker_value_white as usize + i] +=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.special
                            [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i];
                gradient.special
                    [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[BLACK] as usize + i] -=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.special
                            [IDX_SAFETY_TABLE + 2 * attacker_value_black as usize + i];
                gradient.special[IDX_SAFETY_TABLE + 2 * attacker_value_black as usize + i] +=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.special
                            [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[BLACK] as usize + i];
                //Attack constants
                if TUNE_ATTACK_INDEX {
                    //Knight
                    {
                        let c = tuner.params.special[IDX_KNIGHT_ATTACK_VALUE + i];
                        gradient.special[IDX_KNIGHT_ATTACK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - attack_knight_white,
                                pos.trace.knight_attacked_sq[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.special[IDX_KNIGHT_ATTACK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[BLACK] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_black - attack_knight_black,
                                pos.trace.knight_attacked_sq[BLACK],
                                c,
                            )
                            / 100.0;
                    }
                    //Bishop
                    {
                        let c = tuner.params.special[IDX_BISHOP_ATTACK_VALUE + i];
                        gradient.special[IDX_BISHOP_ATTACK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - attack_bishop_white,
                                pos.trace.bishop_attacked_sq[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.special[IDX_BISHOP_ATTACK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[BLACK] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_black - attack_bishop_black,
                                pos.trace.bishop_attacked_sq[BLACK],
                                c,
                            )
                            / 100.0;
                    }
                    //Rook
                    {
                        let c = tuner.params.special[IDX_ROOK_ATTACK_VALUE + i];
                        gradient.special[IDX_ROOK_ATTACK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - attack_rook_white,
                                pos.trace.rook_attacked_sq[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.special[IDX_ROOK_ATTACK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[BLACK] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_black - attack_rook_black,
                                pos.trace.rook_attacked_sq[BLACK],
                                c,
                            )
                            / 100.0;
                    }
                    //Queen
                    {
                        let c = tuner.params.special[IDX_QUEEN_ATTACK_VALUE + i];
                        gradient.special[IDX_QUEEN_ATTACK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - attack_queen_white,
                                pos.trace.queen_attacked_sq[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.special[IDX_QUEEN_ATTACK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[BLACK] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_black - attack_queen_black,
                                pos.trace.queen_attacked_sq[BLACK],
                                c,
                            )
                            / 100.0;
                    }
                    //Knight check
                    {
                        let c = tuner.params.special[IDX_KNIGHT_CHECK_VALUE + i];
                        gradient.special[IDX_KNIGHT_CHECK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - knight_check_white,
                                pos.trace.knight_safe_check[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.special[IDX_KNIGHT_CHECK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[BLACK] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_black - knight_check_black,
                                pos.trace.knight_safe_check[BLACK],
                                c,
                            )
                            / 100.0;
                    }
                    //Bishop check
                    {
                        let c = tuner.params.special[IDX_BISHOP_CHECK_VALUE + i];
                        gradient.special[IDX_BISHOP_CHECK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - bishop_check_white,
                                pos.trace.bishop_safe_check[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.special[IDX_BISHOP_CHECK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[BLACK] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_black - bishop_check_black,
                                pos.trace.bishop_safe_check[BLACK],
                                c,
                            )
                            / 100.0;
                    }
                    //Rook check
                    {
                        let c = tuner.params.special[IDX_ROOK_CHECK_VALUE + i];
                        gradient.special[IDX_ROOK_CHECK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - rook_check_white,
                                pos.trace.rook_safe_check[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.special[IDX_ROOK_CHECK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[BLACK] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_black - rook_check_black,
                                pos.trace.rook_safe_check[BLACK],
                                c,
                            )
                            / 100.0;
                    }
                    //Queen check
                    {
                        let c = tuner.params.special[IDX_QUEEN_CHECK_VALUE + i];
                        gradient.special[IDX_QUEEN_CHECK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - queen_check_white,
                                pos.trace.queen_safe_check[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.special[IDX_QUEEN_CHECK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.special
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[BLACK] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_black - queen_check_black,
                                pos.trace.queen_safe_check[BLACK],
                                c,
                            )
                            / 100.0;
                    }
                }
            }
        }
    }
    gradient.scale(portion);
    add_regularization(&mut gradient, &tuner.params, portion);
    gradient
}

pub fn dsafetytabledconstant(
    tuner: &Tuner,
    phase: usize,
    other: f32,
    relevant_feature: u8,
    current_constant: f32,
) -> f32 {
    let safety_table_inc = tuner.params.special[IDX_SAFETY_TABLE
        + 2 * ((other + f32::from(relevant_feature) * (current_constant + 1.)) as usize)
            .max(0)
            .min(99)
        + phase];
    let safety_table_dec = tuner.params.special[IDX_SAFETY_TABLE
        + 2 * ((other + f32::from(relevant_feature) * (current_constant - 1.)) as usize)
            .max(0)
            .min(99)
        + phase];

    (safety_table_inc - safety_table_dec) / 2.
}

pub fn texel_tuning(tuner: &mut Tuner) {
    let mut best_error = average_evaluation_error(&tuner);
    println!("Error in epoch 0: {}", best_error);
    let mut epoch = 0;
    let mut lr = START_LEARNING_RATE;
    let mut adagrad = Parameters::zero();

    loop {
        epoch += 1;
        println!("Starting epoch {}!", epoch);
        shuffle_positions(tuner);
        let mut ada_add = Parameters::zero();
        for batch in 0..=(tuner.positions.len() - 1) / BATCH_SIZE {
            let from = batch * BATCH_SIZE;
            let mut to = (batch + 1) * BATCH_SIZE;
            if to > tuner.positions.len() {
                to = tuner.positions.len();
            }
            let mut gradient = calculate_gradient(tuner, from, to);
            ada_add.add(&gradient, 1.);

            let mut ada_lr = adagrad.clone();
            ada_lr.add_scalar(1e-6);
            ada_lr.sqrt();
            gradient.mul_inverse_other(&ada_lr);
            tuner.params.add(&gradient, lr);
        }
        ada_add.square();
        adagrad.add(&ada_add, 1.);

        update_evaluations(tuner);
        let error = average_evaluation_error(tuner);
        println!("Error in epoch {}: {}", epoch, error);
        if error < best_error {
            best_error = error;
            tuner
                .params
                .write_to_file(&format!("{}tunebest.txt", PARAM_FILE));
            println!("Saved new best params in tunebest.txt");
        } else {
            lr /= 1.25;
        }
        //Save progress
        if (epoch + 1) % 10 == 0 {
            tuner
                .params
                .write_to_file(&format!("{}tune{}.txt", PARAM_FILE, epoch + 1));
            println!("Saved general progress params in tune.txt");
        }
    }
}

pub fn average_evaluation_error(tuner: &Tuner) -> f32 {
    let mut res = 0.;
    for pos in &tuner.positions {
        res += (pos.label - sigmoid(tuner.k, pos.eval)).powf(2.0);
    }
    res / tuner.positions.len() as f32
}

pub fn minimize_evaluation_error_fork(tuner: &mut Tuner) -> f32 {
    let mut best_k = tuner.k;
    let mut best_error = average_evaluation_error(&tuner);
    println!("Error in epoch 0: {}", best_error);
    let mut epoch = 0;
    let mut lr = 0.3;
    loop {
        epoch += 1;
        //Shuffle positions
        shuffle_positions(tuner);
        //Calculate dE/dk
        for batch in 0..=(tuner.positions.len() - 1) / BATCH_SIZE {
            let from = batch * BATCH_SIZE;
            let mut to = (batch + 1) * BATCH_SIZE;
            if to > tuner.positions.len() {
                to = tuner.positions.len();
            }
            let mut dedk = 0.;
            for pos in &tuner.positions[from..to] {
                let eval = pos.eval;
                dedk += (pos.label - sigmoid(tuner.k, eval)) * dsigmoiddk(tuner.k, eval);
            }
            dedk *= -2.0 / (to - from) as f32;
            tuner.k += -lr * dedk;
        }

        let error = average_evaluation_error(&tuner);
        println!("Error in epoch {}: {}", epoch, error);
        if error < best_error {
            best_error = error;
            best_k = tuner.k;
        } else {
            lr /= 2.0;
            tuner.k = best_k;
        }
        if lr <= 0.001 || epoch >= 20 {
            break;
        }
    }
    best_k
}

pub fn sigmoid(k: f32, s: f32) -> f32 {
    1. / (1. + 10f32.powf(-k * s / 400.0))
}

pub fn dsigmoiddk(k: f32, s: f32) -> f32 {
    sigmoid(k, s).powf(2.0) * 10f32.ln() * s * 10f32.powf(-k * s / 400.0) / 400.0
}
