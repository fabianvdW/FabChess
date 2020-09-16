extern crate core;
extern crate rand;

pub mod loading;

pub use crate::loading::{FileFormatSupported, LabelledGameState, Statistics};
use core_sdk::board_representation::game_state::PieceType;
pub use core_sdk::evaluation::parameters::{normal_parameters::*, special_parameters::*, *};
use core_sdk::evaluation::trace::CollapsedTrace;
use rand::{seq::SliceRandom, thread_rng};

pub const PARAM_FILE: &str = "D:/FenCollection/Andrews/E12.41-1M-D12-Resolved";
//Override for all others if true
pub const TUNE_ALL: bool = false;

pub const TUNE_TEMPO_BONUS: bool = false;
pub const TUNE_SHIELDING_PAWNS: bool = false;
pub const TUNE_PAWNS: bool = false;
//Category passed pawns
pub const TUNE_PASSED: bool = false;
pub const TUNE_PASSED_PAWN: bool = false;
pub const TUNE_PASSED_PAWN_NOT_BLOCKED: bool = false;

pub const TUNE_KNIGHTS: bool = false;
pub const TUNE_FILES: bool = false;

pub const TUNE_PIECE_VALUES: bool = false;
pub const TUNE_MOBILITY: bool = false;

pub const TUNE_ATTACK: bool = true;
pub const TUNE_BASE_ATTACK_VALUE: bool = true;
pub const TUNE_BASE_SAFECHECK_VALUES: bool = true;
pub const TUNE_BASE_ATTACK_FORCE: bool = true;
pub const TUNE_PSQT: bool = false;

pub const TUNABLE_PARAM: [bool; NORMAL_PARAMS] = init_tunable_param();

pub const OPTIMIZE_K: bool = false;
pub const BATCH_SIZE: usize = 20000000;
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
            let mut base_attack_value = [[0., 0.], [0., 0.]];
            let mut attack_force_base_value = [[0., 0.], [0., 0.]];
            for &pt in [
                PieceType::Pawn,
                PieceType::Knight,
                PieceType::Bishop,
                PieceType::Rook,
                PieceType::Queen,
            ]
            .iter()
            {
                for phase in 0..2 {
                    for side in 0..2 {
                        base_attack_value[side][phase] += tuner.params.special
                            [IDX_PIECE_BASE_ATTACK_VALUE + 2 * pt as usize + phase]
                            * f32::from(pos.trace.attacked_squares[pt as usize][side])
                            + tuner.params.special
                                [IDX_PIECE_BASE_SAFECHECK_VALUE + 2 * pt as usize + phase]
                                * f32::from(pos.trace.safe_checks[pt as usize][side]);
                        attack_force_base_value[side][phase] += tuner.params.special
                            [IDX_PIECE_BASE_ATTACK_FORCE + 2 * pt as usize + phase]
                            * f32::from(pos.trace.attackers[pt as usize][side]);
                    }
                }
            }
            let attack_force = [
                [
                    (attack_force_base_value[0][0] / 100.).powf(2.),
                    (attack_force_base_value[0][1] / 100.).powf(2.),
                ],
                [
                    (attack_force_base_value[1][0] / 100.).powf(2.),
                    (attack_force_base_value[1][1] / 100.).powf(2.),
                ],
            ];
            for side in 0..2 {
                let dside = if side == 0 { 1. } else { -1. };
                for phase in 0..2 {
                    let dphase = if phase == 0 { devaldmg } else { devaldeg };
                    for &pt in [
                        PieceType::Pawn,
                        PieceType::Knight,
                        PieceType::Bishop,
                        PieceType::Rook,
                        PieceType::Queen,
                    ]
                    .iter()
                    {
                        if TUNE_BASE_ATTACK_VALUE {
                            gradient.special
                                [IDX_PIECE_BASE_ATTACK_VALUE + 2 * pt as usize + phase] +=
                                start_of_gradient
                                    * dphase
                                    * dside
                                    * attack_force[side][phase]
                                    * f32::from(pos.trace.attacked_squares[pt as usize][side]);
                        }
                        if TUNE_BASE_SAFECHECK_VALUES {
                            gradient.special
                                [IDX_PIECE_BASE_SAFECHECK_VALUE + 2 * pt as usize + phase] +=
                                start_of_gradient
                                    * dphase
                                    * dside
                                    * attack_force[side][phase]
                                    * f32::from(pos.trace.safe_checks[pt as usize][side]);
                        }
                        if TUNE_BASE_ATTACK_FORCE {
                            gradient.special
                                [IDX_PIECE_BASE_ATTACK_FORCE + 2 * pt as usize + phase] +=
                                start_of_gradient
                                    * dphase
                                    * dside
                                    * base_attack_value[side][phase]
                                    * 2.
                                    * attack_force[side][phase]
                                    * f32::from(pos.trace.attackers[pt as usize][side])
                                    / 100.
                        }
                    }
                }
            }
        }
    }
    gradient.scale(portion);
    add_regularization(&mut gradient, &tuner.params, portion);
    gradient
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
