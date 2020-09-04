extern crate core;
extern crate rand;

pub mod loading;

pub use crate::loading::{load_positions, FileFormatSupported, LabelledGameState, Statistics};
use core_sdk::board_representation::game_state::{BLACK, PIECE_TYPES, WHITE};
pub use core_sdk::evaluation::parameters::*;
use core_sdk::evaluation::trace::Trace;
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

pub const OPTIMIZE_K: bool = false;
pub const BATCH_SIZE: usize = 10000000;
pub const START_LEARNING_RATE: f32 = 2.;
pub const L1_REGULARIZATION: f32 = 0.;
pub const L2_REGULARIZATION: f32 = 0.;

pub struct TexelState {
    pub label: f32,
    pub eval: f32,
    pub trace: Trace,
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
    for i in 0..gradient.params.len() {
        gradient.params[i] -= portion * regularization(parameters.params[i]);
    }
}
pub fn regularization(term: f32) -> f32 {
    L1_REGULARIZATION * term.signum() + 2. * L2_REGULARIZATION * term
}
pub fn single_gradient(
    gradient: &mut Parameters,
    idx: usize,
    trace: i8,
    phase: f32,
    start_of_gradient: f32,
) {
    let devaldmg = phase / 128.0;
    let devaldeg = (1. - phase / 128.0) / 1.5;
    let x = f32::from(trace);
    gradient.params[idx] += start_of_gradient * devaldmg * x;
    gradient.params[idx + 1] += start_of_gradient * devaldeg * x;
}
pub fn array_gradient(
    gradient: &mut Parameters,
    idx: usize,
    trace: &[i8],
    phase: f32,
    start_of_gradient: f32,
) {
    for i in 0..trace.len() {
        single_gradient(gradient, idx + 2 * i, trace[i], phase, start_of_gradient);
    }
}
pub fn psqt_gradient(
    gradient: &mut Parameters,
    idx: usize,
    psqt: &[[i8; 8]; 8],
    phase: f32,
    start_of_gradient: f32,
) {
    for i in 0..8 {
        array_gradient(gradient, idx + 16 * i, &psqt[i], phase, start_of_gradient);
    }
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
        let phase = pos.trace.phase;
        let devaldmg = pos.trace.phase / 128.0;
        let devaldeg = (1. - pos.trace.phase / 128.0) / 1.5;
        //Tempo-bonus
        if TUNE_TEMPO_BONUS {
            single_gradient(
                &mut gradient,
                IDX_TEMPO_BONUS,
                pos.trace.tempo_bonus,
                phase,
                start_of_gradient,
            );
        }
        //Shielding pawns
        if TUNE_SHIELDING_PAWNS || TUNE_ALL {
            array_gradient(
                &mut gradient,
                IDX_SHIELDING_PAWN_MISSING,
                &pos.trace.shielding_pawn_missing,
                phase,
                start_of_gradient,
            );
            array_gradient(
                &mut gradient,
                IDX_SHIELDING_PAWN_ONOPEN_MISSING,
                &pos.trace.shielding_pawn_onopen_missing,
                phase,
                start_of_gradient,
            );
        }
        //Pawn bonuses
        if TUNE_PAWNS || TUNE_ALL {
            single_gradient(
                &mut gradient,
                IDX_PAWN_DOUBLED,
                pos.trace.pawn_doubled,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_PAWN_ISOLATED,
                pos.trace.pawn_isolated,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_PAWN_BACKWARD,
                pos.trace.pawn_backward,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_PAWN_ATTACK_CENTER,
                pos.trace.pawn_attack_center,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_PAWN_MOBILITY,
                pos.trace.pawn_mobility,
                phase,
                start_of_gradient,
            );
            psqt_gradient(
                &mut gradient,
                IDX_PAWN_SUPPORTED,
                &pos.trace.pawn_supported,
                phase,
                start_of_gradient,
            );
        }
        //Passed pawns
        if TUNE_PASSED || TUNE_ALL {
            single_gradient(
                &mut gradient,
                IDX_ROOK_BEHIND_SUPPORT_PASSER,
                pos.trace.rook_behind_support_passer,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_ROOK_BEHIND_ENEMY_PASSER,
                pos.trace.rook_behind_enemy_passer,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_PAWN_PASSED_WEAK,
                pos.trace.pawn_passed_weak,
                phase,
                start_of_gradient,
            );
            array_gradient(
                &mut gradient,
                IDX_PAWN_PASSED,
                &pos.trace.pawn_passed,
                phase,
                start_of_gradient,
            );
            array_gradient(
                &mut gradient,
                IDX_PAWN_PASSED_NOTBLOCKED,
                &pos.trace.pawn_passed_notblocked,
                phase,
                start_of_gradient,
            );
            array_gradient(
                &mut gradient,
                IDX_PAWN_PASSED_KINGDISTANCE,
                &pos.trace.pawn_passed_kingdistance,
                phase,
                start_of_gradient,
            );
            array_gradient(
                &mut gradient,
                IDX_PAWN_PASSED_ENEMYKINGDISTANCE,
                &pos.trace.pawn_passed_enemykingdistance,
                phase,
                start_of_gradient,
            );
            array_gradient(
                &mut gradient,
                IDX_PAWN_PASSED_SUBDISTANCE,
                &pos.trace.pawn_passed_subdistance,
                phase,
                start_of_gradient,
            );
        }
        //Knight supported
        if TUNE_KNIGHTS || TUNE_ALL {
            single_gradient(
                &mut gradient,
                IDX_KNIGHT_SUPPORTED,
                pos.trace.knight_supported,
                phase,
                start_of_gradient,
            );
            psqt_gradient(
                &mut gradient,
                IDX_KNIGHT_OUTPOST_TABLE,
                &pos.trace.knight_outpost_table,
                phase,
                start_of_gradient,
            );
        }
        //All PST
        if TUNE_PSQT || TUNE_ALL {
            for &pt in PIECE_TYPES.iter() {
                psqt_gradient(
                    &mut gradient,
                    IDX_PSQT + pt as usize * 128,
                    &pos.trace.psqt[pt as usize],
                    phase,
                    start_of_gradient,
                );
            }
        }

        //On open File / semi open file
        if TUNE_FILES || TUNE_ALL {
            single_gradient(
                &mut gradient,
                IDX_ROOK_ON_OPEN,
                pos.trace.rook_on_open,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_ROOK_ON_SEMI_OPEN,
                pos.trace.rook_on_semi_open,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_QUEEN_ON_OPEN,
                pos.trace.queen_on_open,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_QUEEN_ON_SEMI_OPEN,
                pos.trace.queen_on_semi_open,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_ROOK_ON_SEVENTH,
                pos.trace.rook_on_seventh,
                phase,
                start_of_gradient,
            );
        }
        //Piece values
        if TUNE_PIECE_VALUES || TUNE_ALL {
            single_gradient(
                &mut gradient,
                IDX_PAWN_PIECE_VALUE,
                pos.trace.pawns,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_KNIGHT_PIECE_VALUE,
                pos.trace.knights,
                phase,
                start_of_gradient,
            );
            let knights = f32::from(pos.trace.knights);
            gradient.params
                [IDX_KNIGHT_VALUE_WITH_PAWN + pos.trace.knight_value_with_pawns as usize] +=
                start_of_gradient * knights;
            single_gradient(
                &mut gradient,
                IDX_BISHOP_PIECE_VALUE,
                pos.trace.bishops,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_BISHOP_PAIR,
                pos.trace.bishop_bonus,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_ROOK_PIECE_VALUE,
                pos.trace.rooks,
                phase,
                start_of_gradient,
            );
            single_gradient(
                &mut gradient,
                IDX_QUEEN_PIECE_VALUE,
                pos.trace.queens,
                phase,
                start_of_gradient,
            );
            array_gradient(
                &mut gradient,
                IDX_DIAGONALLY_ADJ_SQ_WPAWNS,
                &pos.trace.diagonally_adjacent_squares_withpawns,
                phase,
                start_of_gradient,
            );
        }
        //Mobility
        if TUNE_MOBILITY || TUNE_ALL {
            array_gradient(
                &mut gradient,
                IDX_KNIGHT_MOBILITY,
                &pos.trace.knight_mobility,
                phase,
                start_of_gradient,
            );
            array_gradient(
                &mut gradient,
                IDX_BISHOP_MOBILITY,
                &pos.trace.bishop_mobility,
                phase,
                start_of_gradient,
            );
            array_gradient(
                &mut gradient,
                IDX_ROOK_MOBILITY,
                &pos.trace.rook_mobility,
                phase,
                start_of_gradient,
            );
            array_gradient(
                &mut gradient,
                IDX_QUEEN_MOBILITY,
                &pos.trace.queen_mobility,
                phase,
                start_of_gradient,
            );
        }
        //Safety
        if TUNE_ATTACK {
            for i in 0..2 {
                let devaldg = if i == 0 { devaldmg } else { devaldeg };
                let attack_knight_white = f32::from(pos.trace.knight_attacked_sq[WHITE])
                    * tuner.params.params[IDX_KNIGHT_ATTACK_VALUE + i];
                let attack_bishop_white = f32::from(pos.trace.bishop_attacked_sq[WHITE])
                    * tuner.params.params[IDX_BISHOP_ATTACK_VALUE + i];
                let attack_rook_white = f32::from(pos.trace.rook_attacked_sq[WHITE])
                    * tuner.params.params[IDX_ROOK_ATTACK_VALUE + i];
                let attack_queen_white = f32::from(pos.trace.queen_attacked_sq[WHITE])
                    * tuner.params.params[IDX_QUEEN_ATTACK_VALUE + i];
                let knight_check_white = f32::from(pos.trace.knight_safe_check[WHITE])
                    * tuner.params.params[IDX_KNIGHT_CHECK_VALUE + i];
                let bishop_check_white = f32::from(pos.trace.bishop_safe_check[WHITE])
                    * tuner.params.params[IDX_BISHOP_CHECK_VALUE + i];
                let rook_check_white = f32::from(pos.trace.rook_safe_check[WHITE])
                    * tuner.params.params[IDX_ROOK_CHECK_VALUE + i];
                let queen_check_white = f32::from(pos.trace.queen_safe_check[WHITE])
                    * tuner.params.params[IDX_QUEEN_CHECK_VALUE + i];
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
                    * tuner.params.params[IDX_KNIGHT_ATTACK_VALUE + i];
                let attack_bishop_black = f32::from(pos.trace.bishop_attacked_sq[BLACK])
                    * tuner.params.params[IDX_BISHOP_ATTACK_VALUE + i];
                let attack_rook_black = f32::from(pos.trace.rook_attacked_sq[BLACK])
                    * tuner.params.params[IDX_ROOK_ATTACK_VALUE + i];
                let attack_queen_black = f32::from(pos.trace.queen_attacked_sq[BLACK])
                    * tuner.params.params[IDX_QUEEN_ATTACK_VALUE + i];
                let knight_check_black = f32::from(pos.trace.knight_safe_check[BLACK])
                    * tuner.params.params[IDX_KNIGHT_CHECK_VALUE + i];
                let bishop_check_black = f32::from(pos.trace.bishop_safe_check[BLACK])
                    * tuner.params.params[IDX_BISHOP_CHECK_VALUE + i];
                let rook_check_black = f32::from(pos.trace.rook_safe_check[BLACK])
                    * tuner.params.params[IDX_ROOK_CHECK_VALUE + i];
                let queen_check_black = f32::from(pos.trace.queen_safe_check[BLACK])
                    * tuner.params.params[IDX_QUEEN_CHECK_VALUE + i];
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
                gradient.params[IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i] +=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.params
                            [IDX_SAFETY_TABLE + 2 * attacker_value_white as usize + i];
                gradient.params[IDX_SAFETY_TABLE + 2 * attacker_value_white as usize + i] +=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.params
                            [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i];
                gradient.params[IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[BLACK] as usize + i] -=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.params
                            [IDX_SAFETY_TABLE + 2 * attacker_value_black as usize + i];
                gradient.params[IDX_SAFETY_TABLE + 2 * attacker_value_black as usize + i] +=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.params
                            [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[BLACK] as usize + i];
                //Attack constants
                if TUNE_ATTACK_INDEX {
                    //Knight
                    {
                        let c = tuner.params.params[IDX_KNIGHT_ATTACK_VALUE + i];
                        gradient.params[IDX_KNIGHT_ATTACK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.params
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - attack_knight_white,
                                pos.trace.knight_attacked_sq[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.params[IDX_KNIGHT_ATTACK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.params
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
                        let c = tuner.params.params[IDX_BISHOP_ATTACK_VALUE + i];
                        gradient.params[IDX_BISHOP_ATTACK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.params
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - attack_bishop_white,
                                pos.trace.bishop_attacked_sq[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.params[IDX_BISHOP_ATTACK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.params
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
                        let c = tuner.params.params[IDX_ROOK_ATTACK_VALUE + i];
                        gradient.params[IDX_ROOK_ATTACK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.params
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - attack_rook_white,
                                pos.trace.rook_attacked_sq[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.params[IDX_ROOK_ATTACK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.params
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
                        let c = tuner.params.params[IDX_QUEEN_ATTACK_VALUE + i];
                        gradient.params[IDX_QUEEN_ATTACK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.params
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - attack_queen_white,
                                pos.trace.queen_attacked_sq[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.params[IDX_QUEEN_ATTACK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.params
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
                        let c = tuner.params.params[IDX_KNIGHT_CHECK_VALUE + i];
                        gradient.params[IDX_KNIGHT_CHECK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.params
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - knight_check_white,
                                pos.trace.knight_safe_check[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.params[IDX_KNIGHT_CHECK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.params
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
                        let c = tuner.params.params[IDX_BISHOP_CHECK_VALUE + i];
                        gradient.params[IDX_BISHOP_CHECK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.params
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - bishop_check_white,
                                pos.trace.bishop_safe_check[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.params[IDX_BISHOP_CHECK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.params
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
                        let c = tuner.params.params[IDX_ROOK_CHECK_VALUE + i];
                        gradient.params[IDX_ROOK_CHECK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.params
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - rook_check_white,
                                pos.trace.rook_safe_check[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.params[IDX_ROOK_CHECK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.params
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
                        let c = tuner.params.params[IDX_QUEEN_CHECK_VALUE + i];
                        gradient.params[IDX_QUEEN_CHECK_VALUE + i] += start_of_gradient
                            * devaldg
                            * tuner.params.params
                                [IDX_ATTACK_WEIGHT + 2 * pos.trace.attackers[WHITE] as usize + i]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - queen_check_white,
                                pos.trace.queen_safe_check[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.params[IDX_QUEEN_CHECK_VALUE + i] -= start_of_gradient
                            * devaldg
                            * tuner.params.params
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
    let safety_table_inc = tuner.params.params[IDX_SAFETY_TABLE
        + 2 * ((other + f32::from(relevant_feature) * (current_constant + 1.)) as usize)
            .max(0)
            .min(99)
        + phase];
    let safety_table_dec = tuner.params.params[IDX_SAFETY_TABLE
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
