extern crate core;
extern crate rand;

pub mod loading;

pub use crate::loading::{load_positions, FileFormatSupported, LabelledGameState, Statistics};
use core_sdk::board_representation::game_state::{BLACK, PIECE_TYPES, WHITE};
use core_sdk::evaluation::eval_game_state;
pub use core_sdk::evaluation::parameters::Parameters;
use core_sdk::evaluation::trace::Trace;
use core_sdk::evaluation::{EG, MG};
use rand::{seq::SliceRandom, thread_rng};

//pub const POSITION_FILE: &str = "D:/FenCollection/Test/all_positions_qsearch.txt";
pub const POSITION_FILE: &str = "D:/FenCollection/Zuri/quiet-labeled.epd";
//pub const POSITION_FILE: &str = "D:/FenCollection/Lichess/lichess-quiet.txt";
pub const PARAM_FILE: &str = "D:/FenCollection/Tuning/";

//Override for all others if true
pub const TUNE_ALL: bool = true;

pub const TUNE_TEMPO_BONUS: bool = false;
pub const TUNE_SHIELDING_PAWNS: bool = false;
pub const TUNE_PAWNS: bool = false;
//Category passed pawns
pub const TUNE_PASSED: bool = false;
pub const TUNE_PASSED_PAWN: bool = false;
pub const TUNE_PASSED_PAWN_NOT_BLOCKED: bool = false;

pub const TUNE_KNIGHTS: bool = false;
pub const TUNE_FILES: bool = true;
pub const TUNE_XRAY: bool = true;

pub const TUNE_PIECE_VALUES: bool = false;
pub const TUNE_MOBILITY: bool = false;

pub const TUNE_ATTACK: bool = true;
pub const TUNE_ATTACK_INDEX: bool = true;
pub const TUNE_PSQT: bool = false;

pub const OPTIMIZE_K: bool = false;
pub const BATCH_SIZE: usize = 100_000;
pub const START_LEARNING_RATE: f64 = 10.;
pub const L1_REGULARIZATION: f64 = 0.;
pub const L2_REGULARIZATION: f64 = 0.;

pub fn init_texel_states(labelledstates: Vec<LabelledGameState>) -> Vec<TexelState> {
    let mut res: Vec<TexelState> = Vec::with_capacity(1);
    for state in labelledstates {
        let eval = eval_game_state(&state.game_state, 0, 0);
        res.push(TexelState {
            label: state.label,
            eval: eval.final_eval as f64,
            trace: eval.trace,
        });
    }
    res
}

pub struct TexelState {
    pub label: f64,
    pub eval: f64,
    pub trace: Trace,
}

pub struct Tuner {
    pub k: f64,
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

pub fn add_gradient(
    params: &[f64; 2],
    portion: f64,
    gradient: &mut [f64; 2],
    trace: i8,
    start_of_gradient: f64,
    phase: f64,
) {
    let devaldmg = phase / 128.0;
    let devaldeg = (1. - phase / 128.0) / 1.5;
    let x = f64::from(trace);
    gradient[MG] += start_of_gradient * devaldmg * x - portion * regularization(params[MG]);
    gradient[EG] += start_of_gradient * devaldeg * x - portion * regularization(params[EG]);
}
pub fn regularization(term: f64) -> f64 {
    L1_REGULARIZATION * term.signum() + 2. * L2_REGULARIZATION * term
}
pub fn calculate_gradient(tuner: &mut Tuner, from: usize, to: usize) -> Parameters {
    let mut gradient = Parameters::zero();
    for pos in tuner.positions[from..to].iter_mut() {
        //Step 1. Update evaluation
        pos.eval = pos.trace.evaluate(&tuner.params);
    }
    //let g = tuner.k * 10f64.ln() / 400.0;
    for pos in tuner.positions[from..to].iter() {
        //Step 2. Calculate first half of gradient
        let s = sigmoid(tuner.k, pos.eval);
        let portion = 1. / (to - from) as f64;
        let start_of_gradient = 2. * portion * (pos.label - s) * s * (1. - s);
        let phase = pos.trace.phase;
        let devaldmg = pos.trace.phase / 128.0;
        let devaldeg = (1. - pos.trace.phase / 128.0) / 1.5;
        //Tempo-bonus
        if TUNE_TEMPO_BONUS {
            add_gradient(
                &tuner.params.tempo_bonus,
                portion,
                &mut gradient.tempo_bonus,
                pos.trace.tempo_bonus,
                start_of_gradient,
                phase,
            );
        }
        //Shielding pawns
        if TUNE_SHIELDING_PAWNS || TUNE_ALL {
            for i in 0..4 {
                let x = f64::from(pos.trace.shielding_pawn_missing[i]);
                let y = f64::from(pos.trace.shielding_pawn_onopen_missing[i]);
                gradient.shielding_pawn_missing[MG][i] += start_of_gradient * devaldmg * x
                    - portion * regularization(tuner.params.shielding_pawn_missing[MG][i]);
                gradient.shielding_pawn_missing[EG][i] += start_of_gradient * devaldeg * x
                    - portion * regularization(tuner.params.shielding_pawn_missing[EG][i]);
                gradient.shielding_pawn_onopen_missing[MG][i] += start_of_gradient * devaldmg * y
                    - portion * regularization(tuner.params.shielding_pawn_onopen_missing[MG][i]);
                gradient.shielding_pawn_onopen_missing[EG][i] += start_of_gradient * devaldeg * y
                    - portion * regularization(tuner.params.shielding_pawn_onopen_missing[EG][i]);
            }
        }
        //Pawn bonuses
        if TUNE_PAWNS || TUNE_ALL {
            add_gradient(
                &tuner.params.pawn_doubled,
                portion,
                &mut gradient.pawn_doubled,
                pos.trace.pawn_doubled,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.pawn_isolated,
                portion,
                &mut gradient.pawn_isolated,
                pos.trace.pawn_isolated,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.pawn_backward,
                portion,
                &mut gradient.pawn_backward,
                pos.trace.pawn_backward,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.pawn_attack_center,
                portion,
                &mut gradient.pawn_attack_center,
                pos.trace.pawn_attack_center,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.pawn_attack_center,
                portion,
                &mut gradient.pawn_mobility,
                pos.trace.pawn_mobility,
                start_of_gradient,
                phase,
            );
        }
        //Passed pawns
        if TUNE_PASSED || TUNE_ALL {
            add_gradient(
                &tuner.params.rook_behind_support_passer,
                portion,
                &mut gradient.rook_behind_support_passer,
                pos.trace.rook_behind_support_passer,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.rook_behind_enemy_passer,
                portion,
                &mut gradient.rook_behind_enemy_passer,
                pos.trace.rook_behind_enemy_passer,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.pawn_passed_weak,
                portion,
                &mut gradient.pawn_passed_weak,
                pos.trace.pawn_passed_weak,
                start_of_gradient,
                phase,
            );
            for i in 0..7 {
                let x = f64::from(pos.trace.pawn_passed[i]);
                let y = f64::from(pos.trace.pawn_passed_notblocked[i]);

                if TUNE_PASSED_PAWN || TUNE_ALL {
                    gradient.pawn_passed[MG][i] += start_of_gradient * devaldmg * x
                        - portion * regularization(tuner.params.pawn_passed[MG][i]);
                    gradient.pawn_passed[EG][i] += start_of_gradient * devaldeg * x
                        - portion * regularization(tuner.params.pawn_passed[EG][i]);
                }
                if TUNE_PASSED_PAWN_NOT_BLOCKED || TUNE_ALL {
                    gradient.pawn_passed_notblocked[MG][i] += start_of_gradient * devaldmg * y
                        - portion * regularization(tuner.params.pawn_passed_notblocked[MG][i]);
                    gradient.pawn_passed_notblocked[EG][i] += start_of_gradient * devaldeg * y
                        - portion * regularization(tuner.params.pawn_passed_notblocked[EG][i]);
                }
                let x = f64::from(pos.trace.pawn_passed_kingdistance[i]);
                gradient.pawn_passed_kingdistance[MG][i] += start_of_gradient * devaldmg * x
                    - portion * regularization(tuner.params.pawn_passed_kingdistance[MG][i]);
                gradient.pawn_passed_kingdistance[EG][i] += start_of_gradient * devaldeg * x
                    - portion * regularization(tuner.params.pawn_passed_kingdistance[EG][i]);

                let x = f64::from(pos.trace.pawn_passed_enemykingdistance[i]);
                gradient.pawn_passed_enemykingdistance[MG][i] += start_of_gradient * devaldmg * x
                    - portion * regularization(tuner.params.pawn_passed_enemykingdistance[MG][i]);
                gradient.pawn_passed_enemykingdistance[EG][i] += start_of_gradient * devaldeg * x
                    - portion * regularization(tuner.params.pawn_passed_enemykingdistance[EG][i]);
            }
            for i in 0..13 {
                let x = f64::from(pos.trace.pawn_passed_subdistance[i]);
                gradient.pawn_passed_subdistance[MG][i] += start_of_gradient * devaldmg * x
                    - portion * regularization(tuner.params.pawn_passed_subdistance[MG][i]);
                gradient.pawn_passed_subdistance[EG][i] += start_of_gradient * devaldeg * x
                    - portion * regularization(tuner.params.pawn_passed_subdistance[EG][i]);
            }
        }
        //Knight supported
        if TUNE_KNIGHTS || TUNE_ALL {
            add_gradient(
                &tuner.params.knight_supported,
                portion,
                &mut gradient.knight_supported,
                pos.trace.knight_supported,
                start_of_gradient,
                phase,
            );
        }
        //All PST
        for i in 0..8 {
            for j in 0..8 {
                if TUNE_PAWNS || TUNE_ALL {
                    let supported = f64::from(pos.trace.pawn_supported[i][j]);
                    gradient.pawn_supported[MG][i][j] += start_of_gradient * devaldmg * supported
                        - portion * regularization(tuner.params.pawn_supported[MG][i][j]);
                    gradient.pawn_supported[EG][i][j] += start_of_gradient * devaldeg * supported
                        - portion * regularization(tuner.params.pawn_supported[EG][i][j]);
                }
                if TUNE_KNIGHTS || TUNE_ALL {
                    let outposts = f64::from(pos.trace.knight_outpost_table[i][j]);

                    gradient.knight_outpost_table[MG][i][j] +=
                        start_of_gradient * devaldmg * outposts
                            - portion * regularization(tuner.params.knight_outpost_table[MG][i][j]);
                    gradient.knight_outpost_table[EG][i][j] +=
                        start_of_gradient * devaldeg * outposts
                            - portion * regularization(tuner.params.knight_outpost_table[EG][i][j]);
                }
                if TUNE_PSQT || TUNE_ALL {
                    for pt in PIECE_TYPES.iter() {
                        let piece = f64::from(pos.trace.psqt[*pt as usize][i][j]);
                        gradient.psqt[*pt as usize][MG][i][j] *= start_of_gradient
                            * devaldmg
                            * piece
                            - portion * regularization(tuner.params.psqt[*pt as usize][MG][i][j]);
                        gradient.psqt[*pt as usize][EG][i][j] *= start_of_gradient
                            * devaldeg
                            * piece
                            - portion * regularization(tuner.params.psqt[*pt as usize][EG][i][j]);
                    }
                }
            }
        }

        //On open File / semi open file
        if TUNE_FILES || TUNE_ALL {
            add_gradient(
                &tuner.params.rook_on_open,
                portion,
                &mut gradient.rook_on_open,
                pos.trace.rook_on_open,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.rook_on_semi_open,
                portion,
                &mut gradient.rook_on_semi_open,
                pos.trace.rook_on_open,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.queen_on_open,
                portion,
                &mut gradient.queen_on_open,
                pos.trace.queen_on_open,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.queen_on_semi_open,
                portion,
                &mut gradient.queen_on_semi_open,
                pos.trace.queen_on_semi_open,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.rook_on_seventh,
                portion,
                &mut gradient.rook_on_seventh,
                pos.trace.rook_on_seventh,
                start_of_gradient,
                phase,
            );
        }
        if TUNE_XRAY || TUNE_ALL {
            add_gradient(
                &tuner.params.bishop_xray_king,
                portion,
                &mut gradient.bishop_xray_king,
                pos.trace.bishop_xray_king,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.rook_xray_king,
                portion,
                &mut gradient.rook_xray_king,
                pos.trace.rook_xray_king,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.queen_xray_king,
                portion,
                &mut gradient.queen_xray_king,
                pos.trace.queen_xray_king,
                start_of_gradient,
                phase,
            );
        }
        //Piece values
        if TUNE_PIECE_VALUES || TUNE_ALL {
            add_gradient(
                &tuner.params.pawn_piece_value,
                portion,
                &mut gradient.pawn_piece_value,
                pos.trace.pawns,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.knight_piece_value,
                portion,
                &mut gradient.knight_piece_value,
                pos.trace.knights,
                start_of_gradient,
                phase,
            );
            let knights = f64::from(pos.trace.knights);
            gradient.knight_value_with_pawns[pos.trace.knight_value_with_pawns as usize] +=
                start_of_gradient * knights
                    - portion
                        * regularization(
                            tuner.params.knight_value_with_pawns
                                [pos.trace.knight_value_with_pawns as usize],
                        );

            add_gradient(
                &tuner.params.bishop_piece_value,
                portion,
                &mut gradient.bishop_piece_value,
                pos.trace.bishops,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.bishop_pair,
                portion,
                &mut gradient.bishop_pair,
                pos.trace.bishop_bonus,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.rook_piece_value,
                portion,
                &mut gradient.rook_piece_value,
                pos.trace.rooks,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &tuner.params.queen_piece_value,
                portion,
                &mut gradient.queen_piece_value,
                pos.trace.queens,
                start_of_gradient,
                phase,
            );
        }
        //Diagonally adjacent
        if TUNE_PIECE_VALUES || TUNE_ALL {
            for i in 0..5 {
                let x = f64::from(pos.trace.diagonally_adjacent_squares_withpawns[i]);
                gradient.diagonally_adjacent_squares_withpawns[MG][i] += start_of_gradient
                    * devaldmg
                    * x
                    - portion
                        * regularization(tuner.params.diagonally_adjacent_squares_withpawns[MG][i]);
                gradient.diagonally_adjacent_squares_withpawns[EG][i] += start_of_gradient
                    * devaldeg
                    * x
                    - portion
                        * regularization(tuner.params.diagonally_adjacent_squares_withpawns[EG][i]);
            }
        }
        //Mobility
        if TUNE_MOBILITY || TUNE_ALL {
            for i in 0..9 {
                let x = f64::from(pos.trace.knight_mobility[i]);
                gradient.knight_mobility[MG][i] += start_of_gradient * devaldmg * x
                    - portion * regularization(tuner.params.knight_mobility[MG][i]);
                gradient.knight_mobility[EG][i] += start_of_gradient * devaldeg * x
                    - portion * regularization(tuner.params.knight_mobility[EG][i]);
            }
            for i in 0..14 {
                let x = f64::from(pos.trace.bishop_mobility[i]);
                gradient.bishop_mobility[MG][i] += start_of_gradient * devaldmg * x
                    - portion * regularization(tuner.params.bishop_mobility[MG][i]);
                gradient.bishop_mobility[EG][i] += start_of_gradient * devaldeg * x
                    - portion * regularization(tuner.params.bishop_mobility[EG][i]);
            }
            for i in 0..15 {
                let x = f64::from(pos.trace.rook_mobility[i]);
                gradient.rook_mobility[MG][i] += start_of_gradient * devaldmg * x
                    - portion * regularization(tuner.params.rook_mobility[MG][i]);
                gradient.rook_mobility[EG][i] += start_of_gradient * devaldeg * x
                    - portion * regularization(tuner.params.rook_mobility[EG][i]);
            }
            for i in 0..28 {
                let x = f64::from(pos.trace.queen_mobility[i]);
                gradient.queen_mobility[MG][i] += start_of_gradient * devaldmg * x
                    - portion * regularization(tuner.params.queen_mobility[MG][i]);
                gradient.queen_mobility[EG][i] += start_of_gradient * devaldeg * x
                    - portion * regularization(tuner.params.queen_mobility[EG][i]);
            }
        }
        //Safety
        if TUNE_ATTACK {
            for i in 0..2 {
                let devaldg = if i == 0 { devaldmg } else { devaldeg };
                let attack_knight_white = f64::from(pos.trace.knight_attacked_sq[WHITE])
                    * tuner.params.knight_attack_value[i];
                let attack_bishop_white = f64::from(pos.trace.bishop_attacked_sq[WHITE])
                    * tuner.params.bishop_attack_value[i];
                let attack_rook_white = f64::from(pos.trace.rook_attacked_sq[WHITE])
                    * tuner.params.rook_attack_value[i];
                let attack_queen_white = f64::from(pos.trace.queen_attacked_sq[WHITE])
                    * tuner.params.queen_attack_value[i];
                let knight_check_white = f64::from(pos.trace.knight_safe_check[WHITE])
                    * tuner.params.knight_check_value[i];
                let bishop_check_white = f64::from(pos.trace.bishop_safe_check[WHITE])
                    * tuner.params.bishop_check_value[i];
                let rook_check_white =
                    f64::from(pos.trace.rook_safe_check[WHITE]) * tuner.params.rook_check_value[i];
                let queen_check_white = f64::from(pos.trace.queen_safe_check[WHITE])
                    * tuner.params.queen_check_value[i];
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
                let attack_knight_black = f64::from(pos.trace.knight_attacked_sq[BLACK])
                    * tuner.params.knight_attack_value[i];
                let attack_bishop_black = f64::from(pos.trace.bishop_attacked_sq[BLACK])
                    * tuner.params.bishop_attack_value[i];
                let attack_rook_black = f64::from(pos.trace.rook_attacked_sq[BLACK])
                    * tuner.params.rook_attack_value[i];
                let attack_queen_black = f64::from(pos.trace.queen_attacked_sq[BLACK])
                    * tuner.params.queen_attack_value[i];
                let knight_check_black = f64::from(pos.trace.knight_safe_check[BLACK])
                    * tuner.params.knight_check_value[i];
                let bishop_check_black = f64::from(pos.trace.bishop_safe_check[BLACK])
                    * tuner.params.bishop_check_value[i];
                let rook_check_black =
                    f64::from(pos.trace.rook_safe_check[BLACK]) * tuner.params.rook_check_value[i];
                let queen_check_black = f64::from(pos.trace.queen_safe_check[BLACK])
                    * tuner.params.queen_check_value[i];
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
                gradient.attack_weight[i][pos.trace.attackers[WHITE] as usize] +=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.safety_table[i].safety_table[attacker_value_white as usize]
                        - portion
                            * regularization(
                                tuner.params.attack_weight[i][pos.trace.attackers[WHITE] as usize],
                            );
                gradient.safety_table[i].safety_table[attacker_value_white as usize] +=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.attack_weight[i][pos.trace.attackers[WHITE] as usize]
                        - portion
                            * regularization(
                                tuner.params.safety_table[i].safety_table
                                    [attacker_value_white as usize],
                            );
                gradient.attack_weight[i][pos.trace.attackers[BLACK] as usize] -=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.safety_table[i].safety_table[attacker_value_black as usize]
                        - portion
                            * regularization(
                                tuner.params.attack_weight[i][pos.trace.attackers[BLACK] as usize],
                            );
                gradient.safety_table[i].safety_table[attacker_value_black as usize] +=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.attack_weight[i][pos.trace.attackers[BLACK] as usize]
                        - portion
                            * regularization(
                                tuner.params.safety_table[i].safety_table
                                    [attacker_value_black as usize],
                            );
                //Attack constants
                if TUNE_ATTACK_INDEX {
                    //Knight
                    {
                        let c = tuner.params.knight_attack_value[i];
                        gradient.knight_attack_value[i] += start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[WHITE] as usize]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - attack_knight_white,
                                pos.trace.knight_attacked_sq[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.knight_attack_value[i] -= start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[BLACK] as usize]
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
                        let c = tuner.params.bishop_attack_value[i];
                        gradient.bishop_attack_value[i] += start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[WHITE] as usize]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - attack_bishop_white,
                                pos.trace.bishop_attacked_sq[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.bishop_attack_value[i] -= start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[BLACK] as usize]
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
                        let c = tuner.params.rook_attack_value[i];
                        gradient.rook_attack_value[i] += start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[WHITE] as usize]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - attack_rook_white,
                                pos.trace.rook_attacked_sq[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.rook_attack_value[i] -= start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[BLACK] as usize]
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
                        let c = tuner.params.queen_attack_value[i];
                        gradient.queen_attack_value[i] += start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[WHITE] as usize]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - attack_queen_white,
                                pos.trace.queen_attacked_sq[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.queen_attack_value[i] -= start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[BLACK] as usize]
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
                        let c = tuner.params.knight_check_value[i];
                        gradient.knight_check_value[i] += start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[WHITE] as usize]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - knight_check_white,
                                pos.trace.knight_safe_check[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.knight_check_value[i] -= start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[BLACK] as usize]
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
                        let c = tuner.params.bishop_check_value[i];
                        gradient.bishop_check_value[i] += start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[WHITE] as usize]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - bishop_check_white,
                                pos.trace.bishop_safe_check[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.bishop_check_value[i] -= start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[BLACK] as usize]
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
                        let c = tuner.params.rook_check_value[i];
                        gradient.rook_check_value[i] += start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[WHITE] as usize]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - rook_check_white,
                                pos.trace.rook_safe_check[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.rook_check_value[i] -= start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[BLACK] as usize]
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
                        let c = tuner.params.queen_check_value[i];
                        gradient.queen_check_value[i] += start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[WHITE] as usize]
                            * dsafetytabledconstant(
                                tuner,
                                i,
                                attacker_value_white - queen_check_white,
                                pos.trace.queen_safe_check[WHITE],
                                c,
                            )
                            / 100.0;
                        gradient.queen_check_value[i] -= start_of_gradient
                            * devaldg
                            * tuner.params.attack_weight[i][pos.trace.attackers[BLACK] as usize]
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
    gradient
}

pub fn dsafetytabledconstant(
    tuner: &Tuner,
    phase: usize,
    other: f64,
    relevant_feature: u8,
    current_constant: f64,
) -> f64 {
    let safety_table_inc = tuner.params.safety_table[phase].safety_table[((other
        + f64::from(relevant_feature) * (current_constant + 1.))
        as usize)
        .max(0)
        .min(99)];
    let safety_table_dec = tuner.params.safety_table[phase].safety_table[((other
        + f64::from(relevant_feature) * (current_constant - 1.))
        as usize)
        .max(0)
        .min(99)];

    (safety_table_inc - safety_table_dec) / 2.
}

pub fn texel_tuning(tuner: &mut Tuner) {
    let mut best_error = average_evaluation_error(&tuner);
    println!("Error in epoch 0: {}", best_error);
    let mut epoch = 0;
    let mut lr = START_LEARNING_RATE;
    loop {
        epoch += 1;
        shuffle_positions(tuner);
        for batch in 0..=(tuner.positions.len() - 1) / BATCH_SIZE {
            let from = batch * BATCH_SIZE;
            let mut to = (batch + 1) * BATCH_SIZE;
            if to > tuner.positions.len() {
                to = tuner.positions.len();
            }
            let gradient = calculate_gradient(tuner, from, to);
            tuner.params.apply_gradient(&gradient, lr);
        }

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

pub fn average_evaluation_error(tuner: &Tuner) -> f64 {
    let mut res = 0.;
    for pos in &tuner.positions {
        res += (pos.label - sigmoid(tuner.k, pos.eval)).powf(2.0);
    }
    res / tuner.positions.len() as f64
}

pub fn minimize_evaluation_error_fork(tuner: &mut Tuner) -> f64 {
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
            dedk *= -2.0 / (to - from) as f64;
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

pub fn sigmoid(k: f64, s: f64) -> f64 {
    1. / (1. + 10f64.powf(-k * s / 400.0))
}

pub fn dsigmoiddk(k: f64, s: f64) -> f64 {
    sigmoid(k, s).powf(2.0) * 10f64.ln() * s * 10f64.powf(-k * s / 400.0) / 400.0
}
