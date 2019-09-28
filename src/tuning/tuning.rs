extern crate core;
extern crate rand;

use core::board_representation::game_state::{BLACK, WHITE};
#[cfg(feature = "texel-tuning")]
use core::evaluation::eval_game_state_from_null;
use core::evaluation::{EG, MG};
#[cfg(feature = "texel-tuning")]
use core::tuning::loading::{load_positions, FileFormatSupported, LabelledGameState, Statistics};
use core::tuning::parameters::Parameters;
use core::tuning::trace::Trace;
use rand::{seq::SliceRandom, thread_rng};

//pub const POSITION_FILE: &str = "D:/FenCollection/Test/all_positions_qsearch.txt";
//pub const POSITION_FILE: &str = "D:/FenCollection/Zuri/quiet-labeled.epd";
pub const POSITION_FILE: &str = "D:/FenCollection/Lichess/lichess-quiet.txt";
pub const PARAM_FILE: &str = "D:/FenCollection/Tuning/";

//Override for all others if true
pub const TUNE_ALL: bool = true;

pub const TUNE_TEMPO_BONUS: bool = false;
pub const TUNE_SHIELDING_PAWNS: bool = false;
pub const TUNE_PAWNS: bool = false;
//Category passed pawns
pub const TUNE_PASSED: bool = true;
pub const TUNE_PASSED_PAWN: bool = true;
pub const TUNE_PASSED_PAWN_NOT_BLOCKED: bool = false;

pub const TUNE_KNIGHTS: bool = false;
pub const TUNE_ROOKS: bool = false;

pub const TUNE_PIECE_VALUES: bool = false;
pub const TUNE_MOBILITY: bool = false;

pub const TUNE_ATTACK: bool = false;
pub const TUNE_ATTACK_INDEX: bool = false;
pub const TUNE_PSQT: bool = false;

//const BATCH_SIZE: usize = 2500000;
//const BATCH_SIZE: usize = 725000;
const BATCH_SIZE: usize = 100_000;

pub fn main() {
    if !cfg!(feature = "texel-tuning") {
        panic!("Feature texel-tuning has to be enabled");
    }
    #[cfg(feature = "texel-tuning")]
    {
        //Step 1. Load all positions from a file. Those positions should already be the q-searched positions.
        let mut stats = Statistics::default();
        let mut positions: Vec<LabelledGameState> = Vec::with_capacity(8000000);
        load_positions(
            POSITION_FILE,
            if POSITION_FILE.ends_with(".txt") {
                FileFormatSupported::OwnEncoding
            } else if POSITION_FILE.ends_with("epd") {
                FileFormatSupported::EPD
            } else {
                panic!("Invalid position file encoding!")
            },
            &mut positions,
            &mut stats,
        );
        println!(
            "Loaded file {} with {} positions!",
            POSITION_FILE,
            positions.len()
        );
        let mut tuner = Tuner {
            k: 1.1155,
            positions: init_texel_states(positions),
            params: Parameters::default(),
        };
        println!("Start tuning for k");
        //minimize_evaluation_error_fork(&mut tuner);
        println!("Optimal K: {}", tuner.k);
        texel_tuning(&mut tuner);
    }
    //params.write_to_file(&format!("{}tune.txt", PARAM_FILE));
}

#[cfg(feature = "texel-tuning")]
pub fn init_texel_states(labelledstates: Vec<LabelledGameState>) -> Vec<TexelState> {
    let mut res: Vec<TexelState> = Vec::with_capacity(7881908);
    for state in labelledstates {
        let eval = eval_game_state_from_null(&state.game_state);
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

pub fn add_gradient(gradient: &mut [f64; 2], trace: [i8; 2], start_of_gradient: f64, phase: f64) {
    let devaldmg = phase / 128.0;
    let devaldeg = (1. - phase / 128.0) / 1.5;
    let x = f64::from(trace[WHITE] - trace[BLACK]);
    gradient[MG] += start_of_gradient * devaldmg * x;
    gradient[EG] += start_of_gradient * devaldeg * x;
}

pub fn calculate_gradient(tuner: &mut Tuner, from: usize, to: usize, lr: f64) -> (Parameters, f64) {
    let mut gradient = Parameters::zero();
    for pos in tuner.positions[from..to].iter_mut() {
        //Step 1. Update evaluation
        pos.eval = pos.trace.evaluate(&tuner.params);
    }
    let multiplier: f64 = 2. * lr / (to - from) as f64;
    //let g = tuner.k * 10f64.ln() / 400.0;
    for pos in tuner.positions[from..to].iter() {
        //Step 2. Calculate first half of gradient
        let s = sigmoid(tuner.k, pos.eval);
        let start_of_gradient = (pos.label - s) * s * (1. - s);
        let phase = pos.trace.phase;
        let devaldmg = pos.trace.phase / 128.0;
        let devaldeg = (1. - pos.trace.phase / 128.0) / 1.5;
        //Tempo-bonus
        if TUNE_TEMPO_BONUS {
            add_gradient(
                &mut gradient.tempo_bonus,
                pos.trace.tempo_bonus,
                start_of_gradient,
                phase,
            );
        }
        //Shielding pawns
        if TUNE_SHIELDING_PAWNS || TUNE_ALL {
            for i in 0..4 {
                let x = f64::from(
                    pos.trace.shielding_pawn_missing[WHITE][i]
                        - pos.trace.shielding_pawn_missing[BLACK][i],
                );
                let y = f64::from(
                    pos.trace.shielding_pawn_onopen_missing[WHITE][i]
                        - pos.trace.shielding_pawn_onopen_missing[BLACK][i],
                );
                gradient.shielding_pawn_missing[MG][i] += start_of_gradient * devaldmg * x;
                gradient.shielding_pawn_missing[EG][i] += start_of_gradient * devaldeg * x;
                gradient.shielding_pawn_onopen_missing[MG][i] += start_of_gradient * devaldmg * y;
                gradient.shielding_pawn_onopen_missing[EG][i] += start_of_gradient * devaldeg * y;
            }
        }
        //Pawn bonuses
        if TUNE_PAWNS || TUNE_ALL {
            add_gradient(
                &mut gradient.pawn_doubled,
                pos.trace.pawn_doubled,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &mut gradient.pawn_isolated,
                pos.trace.pawn_isolated,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &mut gradient.pawn_backward,
                pos.trace.pawn_backward,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &mut gradient.pawn_attack_center,
                pos.trace.pawn_attack_center,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &mut gradient.pawn_mobility,
                pos.trace.pawn_mobility,
                start_of_gradient,
                phase,
            );
        }
        //Passed pawns
        if TUNE_PASSED || TUNE_ALL {
            add_gradient(
                &mut gradient.rook_behind_support_passer,
                pos.trace.rook_behind_support_passer,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &mut gradient.rook_behind_enemy_passer,
                pos.trace.rook_behind_enemy_passer,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &mut gradient.pawn_passed_weak,
                pos.trace.pawn_passed_weak,
                start_of_gradient,
                phase,
            );
            for i in 0..7 {
                let x =
                    f64::from(pos.trace.pawn_passed[WHITE][i] - pos.trace.pawn_passed[BLACK][i]);
                let y = f64::from(
                    pos.trace.pawn_passed_notblocked[WHITE][i]
                        - pos.trace.pawn_passed_notblocked[BLACK][i],
                );

                if TUNE_PASSED_PAWN || TUNE_ALL {
                    gradient.pawn_passed[MG][i] += start_of_gradient * devaldmg * x;
                    gradient.pawn_passed[EG][i] += start_of_gradient * devaldeg * x;
                }
                if TUNE_PASSED_PAWN_NOT_BLOCKED || TUNE_ALL {
                    gradient.pawn_passed_notblocked[MG][i] += start_of_gradient * devaldmg * y;
                    gradient.pawn_passed_notblocked[EG][i] += start_of_gradient * devaldeg * y;
                }
                let x = f64::from(
                    pos.trace.pawn_passed_kingdistance[WHITE][i]
                        - pos.trace.pawn_passed_kingdistance[BLACK][i],
                );
                gradient.pawn_passed_kingdistance[MG][i] += start_of_gradient * devaldmg * x;
                gradient.pawn_passed_kingdistance[EG][i] += start_of_gradient * devaldeg * x;

                let x = f64::from(
                    pos.trace.pawn_passed_enemykingdistance[WHITE][i]
                        - pos.trace.pawn_passed_enemykingdistance[BLACK][i],
                );
                gradient.pawn_passed_enemykingdistance[MG][i] += start_of_gradient * devaldmg * x;
                gradient.pawn_passed_enemykingdistance[EG][i] += start_of_gradient * devaldeg * x;
            }
            for i in 0..13 {
                let x = f64::from(
                    pos.trace.pawn_passed_subdistance[WHITE][i]
                        - pos.trace.pawn_passed_subdistance[BLACK][i],
                );
                gradient.pawn_passed_subdistance[MG][i] += start_of_gradient * devaldmg * x;
                gradient.pawn_passed_subdistance[EG][i] += start_of_gradient * devaldeg * x;
            }
        }
        //Knight supported
        if TUNE_KNIGHTS || TUNE_ALL {
            add_gradient(
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
                    let supported = f64::from(
                        pos.trace.pawn_supported[WHITE][i][j]
                            - pos.trace.pawn_supported[BLACK][i][j],
                    );
                    gradient.pawn_supported[MG][i][j] += start_of_gradient * devaldmg * supported;
                    gradient.pawn_supported[EG][i][j] += start_of_gradient * devaldeg * supported;
                }
                if TUNE_KNIGHTS || TUNE_ALL {
                    let outposts = f64::from(
                        pos.trace.knight_outpost_table[WHITE][i][j]
                            - pos.trace.knight_outpost_table[BLACK][i][j],
                    );

                    gradient.knight_outpost_table[MG][i][j] +=
                        start_of_gradient * devaldmg * outposts;
                    gradient.knight_outpost_table[EG][i][j] +=
                        start_of_gradient * devaldeg * outposts;
                }
                if TUNE_PSQT || TUNE_ALL {
                    let pawns = f64::from(
                        pos.trace.psqt_pawn[WHITE][i][j] - pos.trace.psqt_pawn[BLACK][i][j],
                    );
                    gradient.psqt_pawn[MG][i][j] += start_of_gradient * devaldmg * pawns;
                    gradient.psqt_pawn[EG][i][j] += start_of_gradient * devaldeg * pawns;

                    let knights = f64::from(
                        pos.trace.psqt_knight[WHITE][i][j] - pos.trace.psqt_knight[BLACK][i][j],
                    );
                    gradient.psqt_knight[MG][i][j] += start_of_gradient * devaldmg * knights;
                    gradient.psqt_knight[EG][i][j] += start_of_gradient * devaldeg * knights;

                    let bishops = f64::from(
                        pos.trace.psqt_bishop[WHITE][i][j] - pos.trace.psqt_bishop[BLACK][i][j],
                    );
                    gradient.psqt_bishop[MG][i][j] += start_of_gradient * devaldmg * bishops;
                    gradient.psqt_bishop[EG][i][j] += start_of_gradient * devaldeg * bishops;

                    let rooks = f64::from(
                        pos.trace.psqt_rook[WHITE][i][j] - pos.trace.psqt_rook[BLACK][i][j],
                    );
                    gradient.psqt_rook[MG][i][j] += start_of_gradient * devaldmg * rooks;
                    gradient.psqt_rook[EG][i][j] += start_of_gradient * devaldeg * rooks;

                    let queens = f64::from(
                        pos.trace.psqt_queen[WHITE][i][j] - pos.trace.psqt_queen[BLACK][i][j],
                    );
                    gradient.psqt_queen[MG][i][j] += start_of_gradient * devaldmg * queens;
                    gradient.psqt_queen[EG][i][j] += start_of_gradient * devaldeg * queens;

                    let king = f64::from(
                        pos.trace.psqt_king[WHITE][i][j] - pos.trace.psqt_king[BLACK][i][j],
                    );
                    gradient.psqt_king[MG][i][j] += start_of_gradient * devaldmg * king;
                    gradient.psqt_king[EG][i][j] += start_of_gradient * devaldeg * king;
                }
            }
        }

        //Rook
        if TUNE_ROOKS || TUNE_ALL {
            add_gradient(
                &mut gradient.rook_on_open,
                pos.trace.rook_on_open,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &mut gradient.rook_on_seventh,
                pos.trace.rook_on_seventh,
                start_of_gradient,
                phase,
            );
        }
        //Piece values
        if TUNE_PIECE_VALUES || TUNE_ALL {
            add_gradient(
                &mut gradient.pawn_piece_value,
                pos.trace.pawns,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &mut gradient.knight_piece_value,
                pos.trace.knights,
                start_of_gradient,
                phase,
            );
            let knights = f64::from(pos.trace.knights[WHITE] - pos.trace.knights[BLACK]);
            gradient.knight_value_with_pawns[pos.trace.knight_value_with_pawns as usize] +=
                start_of_gradient * knights;

            add_gradient(
                &mut gradient.bishop_piece_value,
                pos.trace.bishops,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &mut gradient.bishop_pair,
                pos.trace.bishop_bonus,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &mut gradient.rook_piece_value,
                pos.trace.rooks,
                start_of_gradient,
                phase,
            );
            add_gradient(
                &mut gradient.queen_piece_value,
                pos.trace.queens,
                start_of_gradient,
                phase,
            );
        }
        //Diagonally adjacent
        if TUNE_PIECE_VALUES || TUNE_ALL {
            for i in 0..5 {
                let x = f64::from(
                    pos.trace.diagonally_adjacent_squares_withpawns[WHITE][i]
                        - pos.trace.diagonally_adjacent_squares_withpawns[BLACK][i],
                );
                gradient.diagonally_adjacent_squares_withpawns[MG][i] +=
                    start_of_gradient * devaldmg * x;
                gradient.diagonally_adjacent_squares_withpawns[EG][i] +=
                    start_of_gradient * devaldeg * x;
            }
        }
        //Mobility
        if TUNE_MOBILITY || TUNE_ALL {
            for i in 0..9 {
                let x = f64::from(
                    pos.trace.knight_mobility[WHITE][i] - pos.trace.knight_mobility[BLACK][i],
                );
                gradient.knight_mobility[MG][i] += start_of_gradient * devaldmg * x;
                gradient.knight_mobility[EG][i] += start_of_gradient * devaldeg * x;
            }
            for i in 0..14 {
                let x = f64::from(
                    pos.trace.bishop_mobility[WHITE][i] - pos.trace.bishop_mobility[BLACK][i],
                );
                gradient.bishop_mobility[MG][i] += start_of_gradient * devaldmg * x;
                gradient.bishop_mobility[EG][i] += start_of_gradient * devaldeg * x;
            }
            for i in 0..15 {
                let x = f64::from(
                    pos.trace.rook_mobility[WHITE][i] - pos.trace.rook_mobility[BLACK][i],
                );
                gradient.rook_mobility[MG][i] += start_of_gradient * devaldmg * x;
                gradient.rook_mobility[EG][i] += start_of_gradient * devaldeg * x;
            }
            for i in 0..28 {
                let x = f64::from(
                    pos.trace.queen_mobility[WHITE][i] - pos.trace.queen_mobility[BLACK][i],
                );
                gradient.queen_mobility[MG][i] += start_of_gradient * devaldmg * x;
                gradient.queen_mobility[EG][i] += start_of_gradient * devaldeg * x;
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
                        * tuner.params.safety_table[i].safety_table[attacker_value_white as usize];
                gradient.safety_table[i].safety_table[attacker_value_white as usize] +=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.attack_weight[i][pos.trace.attackers[WHITE] as usize];
                gradient.attack_weight[i][pos.trace.attackers[BLACK] as usize] -=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.safety_table[i].safety_table[attacker_value_black as usize];
                gradient.safety_table[i].safety_table[attacker_value_black as usize] +=
                    start_of_gradient * devaldg / 100.0
                        * tuner.params.attack_weight[i][pos.trace.attackers[BLACK] as usize];
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
    //Norm gradient
    let mut norm: f64 = 0.;
    for i in 0..2 {
        norm += gradient.tempo_bonus[i].powf(2.);
        for j in 0..4 {
            norm += gradient.shielding_pawn_missing[i][j].powf(2.);
            norm += gradient.shielding_pawn_onopen_missing[i][j].powf(2.);
        }
        norm += gradient.pawn_doubled[i].powf(2.);
        norm += gradient.pawn_isolated[i].powf(2.);
        norm += gradient.pawn_backward[i].powf(2.);
        norm += gradient.pawn_attack_center[i].powf(2.);
        norm += gradient.pawn_mobility[i].powf(2.);
        for j in 0..7 {
            norm += gradient.pawn_passed[i][j].powf(2.);
            norm += gradient.pawn_passed_notblocked[i][j].powf(2.);
            norm += gradient.pawn_passed_kingdistance[i][j].powf(2.);
            norm += gradient.pawn_passed_enemykingdistance[i][j].powf(2.);
        }
        for j in 0..13 {
            norm += gradient.pawn_passed_subdistance[i][j].powf(2.);
        }
        norm += gradient.rook_behind_support_passer[i].powf(2.);
        norm += gradient.rook_behind_enemy_passer[i].powf(2.);
        norm += gradient.pawn_passed_weak[i].powf(2.);
        norm += gradient.knight_supported[i].powf(2.);
        for j in 0..8 {
            for k in 0..8 {
                norm += gradient.pawn_supported[i][j][k].powf(2.);
                norm += gradient.knight_outpost_table[i][j][k].powf(2.);
                norm += gradient.psqt_pawn[i][j][k].powf(2.);
                norm += gradient.psqt_knight[i][j][k].powf(2.);
                norm += gradient.psqt_bishop[i][j][k].powf(2.);
                norm += gradient.psqt_rook[i][j][k].powf(2.);
                norm += gradient.psqt_queen[i][j][k].powf(2.);
                norm += gradient.psqt_king[i][j][k].powf(2.);
            }
        }
        norm += gradient.rook_on_open[i].powf(2.);
        norm += gradient.rook_on_seventh[i].powf(2.);
        norm += gradient.pawn_piece_value[i].powf(2.);
        norm += gradient.knight_piece_value[i].powf(2.);
        norm += gradient.bishop_piece_value[i].powf(2.);
        norm += gradient.bishop_pair[i].powf(2.);
        norm += gradient.rook_piece_value[i].powf(2.);
        norm += gradient.queen_piece_value[i].powf(2.);
        for j in 0..5 {
            norm += gradient.diagonally_adjacent_squares_withpawns[i][j].powf(2.);
        }
        for j in 0..9 {
            norm += gradient.knight_mobility[i][j].powf(2.);
        }
        for j in 0..14 {
            norm += gradient.bishop_mobility[i][j].powf(2.);
        }
        for j in 0..15 {
            norm += gradient.rook_mobility[i][j].powf(2.);
        }
        for j in 0..28 {
            norm += gradient.queen_mobility[i][j].powf(2.);
        }
    }
    for i in 0..17 {
        norm += gradient.knight_value_with_pawns[i].powf(2.);
    }
    for i in 0..2 {
        norm += gradient.knight_attack_value[i].powf(2.);
        norm += gradient.bishop_attack_value[i].powf(2.);
        norm += gradient.rook_attack_value[i].powf(2.);
        norm += gradient.queen_attack_value[i].powf(2.);
        norm += gradient.knight_check_value[i].powf(2.);
        norm += gradient.bishop_check_value[i].powf(2.);
        norm += gradient.rook_check_value[i].powf(2.);
        norm += gradient.queen_check_value[i].powf(2.);
        for j in 0..8 {
            norm += gradient.attack_weight[i][j].powf(2.);
        }
    }
    for i in 0..2 {
        for j in 0..100 {
            norm += gradient.safety_table[i].safety_table[j].powf(2.);
        }
    }
    norm = norm.sqrt();
    (gradient, norm / multiplier)
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
    let mut lr = 100_000.0;
    loop {
        epoch += 1;
        shuffle_positions(tuner);
        for batch in 0..=(tuner.positions.len() - 1) / BATCH_SIZE {
            let from = batch * BATCH_SIZE;
            let mut to = (batch + 1) * BATCH_SIZE;
            if to > tuner.positions.len() {
                to = tuner.positions.len();
            }
            let (gradient, norm) = calculate_gradient(tuner, from, to, lr);
            tuner.params.apply_gradient(&gradient, norm);
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
    let mut lr = 0.1;
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
        if lr <= 0.001 {
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
