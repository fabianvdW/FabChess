extern crate core;
extern crate rand;

use core::board_representation::game_state::{BLACK, WHITE};
#[cfg(feature = "texel-tuning")]
use core::evaluation::eval_game_state;
use core::evaluation::{EG, MG};
#[cfg(feature = "texel-tuning")]
use core::tuning::loading::{load_positions, FileFormatSupported, LabelledGameState, Statistics};
use core::tuning::parameters::Parameters;
use core::tuning::trace::Trace;
use rand::{seq::SliceRandom, thread_rng};

//pub const POSITION_FILE: &str = "D:/FenCollection/Real/all_positions_qsearch.txt";
pub const POSITION_FILE: &str = "D:/FenCollection/Zuri/quiet-labeled.epd";
pub const PARAM_FILE: &str = "D:/FenCollection/Tuning/";
//pub const POSITION_FILE: &str = "D:/FenCollection/Test/all_positions_qsearch.txt";
const BATCH_SIZE: usize = 8196;
pub fn main() {
    if !cfg!(feature = "texel-tuning") {
        panic!("Feature texel-tuning has to be enabled");
    }
    #[cfg(feature = "texel-tuning")]
    {
        //Step 1. Load all positions from a file. Those positions should already be the q-searched positions.
        let mut stats = Statistics::new();
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
            k: 0.96,
            positions: init_texel_states(positions),
            params: Parameters::default(),
        };
        println!("Start tuning for k");
        minimize_evaluation_error_fork(&mut tuner);
        println!("Optimal K: {}", tuner.k);
        texel_tuning(&mut tuner);
    }
    //params.write_to_file(&format!("{}tune.txt", PARAM_FILE));
}

#[cfg(feature = "texel-tuning")]
pub fn init_texel_states(labelledstates: Vec<LabelledGameState>) -> Vec<TexelState> {
    let mut res: Vec<TexelState> = Vec::with_capacity(7881908);
    for state in labelledstates {
        let eval = eval_game_state(&state.game_state);
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
pub fn calculate_gradient(tuner: &mut Tuner, from: usize, to: usize, lr: f64) -> Parameters {
    let mut gradient = Parameters::zero();
    let multiplier: f64 = 2. * lr / (to - from) as f64;
    //let g = tuner.k * 10f64.ln() / 400.0;
    for pos in tuner.positions[from..to].iter_mut() {
        //Step 1. Update evaluation
        pos.eval = pos.trace.evaluate(&tuner.params);
        //Step 2. Calculate first half of gradient
        let s = sigmoid(tuner.k, pos.eval);
        let start_of_gradient = multiplier * (pos.label - s) * s * (1. - s);
        let devaldmg = pos.trace.phase / 128.0;
        let devaldeg = 1. - pos.trace.phase / 128.0;
        //Tempo-bonus
        {
            let x = (pos.trace.tempo_bonus[WHITE] - pos.trace.tempo_bonus[BLACK]) as f64;
            gradient.tempo_bonus[MG] += start_of_gradient * devaldmg * x;
            gradient.tempo_bonus[EG] += start_of_gradient * devaldeg * x;
        }
        //Shielding pawns
        for i in 0..4 {
            let x = (pos.trace.shielding_pawn_missing[WHITE][i]
                - pos.trace.shielding_pawn_missing[BLACK][i]) as f64;
            let y = (pos.trace.shielding_pawn_onopen_missing[WHITE][i]
                - pos.trace.shielding_pawn_onopen_missing[BLACK][i]) as f64;
            gradient.shielding_pawn_missing[MG][i] += start_of_gradient * devaldmg * x;
            gradient.shielding_pawn_missing[EG][i] += start_of_gradient * devaldeg * x;
            gradient.shielding_pawn_onopen_missing[MG][i] += start_of_gradient * devaldmg * y;
            gradient.shielding_pawn_onopen_missing[EG][i] += start_of_gradient * devaldeg * y;
        }
        //Pawn bonuses
        {
            let doubled = (pos.trace.pawn_doubled[WHITE] - pos.trace.pawn_doubled[BLACK]) as f64;
            let isolated = (pos.trace.pawn_isolated[WHITE] - pos.trace.pawn_isolated[BLACK]) as f64;
            let backward = (pos.trace.pawn_backward[WHITE] - pos.trace.pawn_backward[BLACK]) as f64;
            let supported =
                (pos.trace.pawn_supported[WHITE] - pos.trace.pawn_supported[BLACK]) as f64;
            let attack_center =
                (pos.trace.pawn_attack_center[WHITE] - pos.trace.pawn_attack_center[BLACK]) as f64;
            gradient.pawn_doubled[MG] += start_of_gradient * devaldmg * doubled;
            gradient.pawn_doubled[EG] += start_of_gradient * devaldeg * doubled;
            gradient.pawn_isolated[MG] += start_of_gradient * devaldmg * isolated;
            gradient.pawn_isolated[EG] += start_of_gradient * devaldeg * isolated;
            gradient.pawn_backward[MG] += start_of_gradient * devaldmg * backward;
            gradient.pawn_backward[EG] += start_of_gradient * devaldeg * backward;
            gradient.pawn_supported[MG] += start_of_gradient * devaldmg * supported;
            gradient.pawn_supported[EG] += start_of_gradient * devaldeg * supported;
            gradient.pawn_attack_center[MG] += start_of_gradient * devaldmg * attack_center;
            gradient.pawn_attack_center[EG] += start_of_gradient * devaldeg * attack_center;
        }
        //Passed pawns
        for i in 0..7 {
            let x = (pos.trace.pawn_passed[WHITE][i] - pos.trace.pawn_passed[BLACK][i]) as f64;
            let y = (pos.trace.pawn_passed_notblocked[WHITE][i]
                - pos.trace.pawn_passed_notblocked[BLACK][i]) as f64;

            gradient.pawn_passed[MG][i] += start_of_gradient * devaldmg * x;
            gradient.pawn_passed[EG][i] += start_of_gradient * devaldeg * x;

            gradient.pawn_passed_notblocked[MG][i] += start_of_gradient * devaldmg * y;
            gradient.pawn_passed_notblocked[EG][i] += start_of_gradient * devaldeg * y;
        }
        //Knight supported
        {
            let x = (pos.trace.knight_supported[WHITE] - pos.trace.knight_supported[BLACK]) as f64;

            gradient.knight_supported[MG] += start_of_gradient * devaldmg * x;
            gradient.knight_supported[EG] += start_of_gradient * devaldeg * x;
        }
        //All PST
        for i in 0..8 {
            for j in 0..8 {
                let outposts = (pos.trace.knight_outpost_table[WHITE][i][j]
                    - pos.trace.knight_outpost_table[BLACK][i][j])
                    as f64;

                gradient.knight_outpost_table[MG][i][j] += start_of_gradient * devaldmg * outposts;
                gradient.knight_outpost_table[EG][i][j] += start_of_gradient * devaldeg * outposts;

                let pawns =
                    (pos.trace.psqt_pawn[WHITE][i][j] - pos.trace.psqt_pawn[BLACK][i][j]) as f64;
                gradient.psqt_pawn[MG][i][j] += start_of_gradient * devaldmg * pawns;
                gradient.psqt_pawn[EG][i][j] += start_of_gradient * devaldeg * pawns;

                let knights = (pos.trace.psqt_knight[WHITE][i][j]
                    - pos.trace.psqt_knight[BLACK][i][j]) as f64;
                gradient.psqt_knight[MG][i][j] += start_of_gradient * devaldmg * knights;
                gradient.psqt_knight[EG][i][j] += start_of_gradient * devaldeg * knights;

                let bishops = (pos.trace.psqt_bishop[WHITE][i][j]
                    - pos.trace.psqt_bishop[BLACK][i][j]) as f64;
                gradient.psqt_bishop[MG][i][j] += start_of_gradient * devaldmg * bishops;
                gradient.psqt_bishop[EG][i][j] += start_of_gradient * devaldeg * bishops;

                let king =
                    (pos.trace.psqt_king[WHITE][i][j] - pos.trace.psqt_king[BLACK][i][j]) as f64;
                gradient.psqt_king[MG][i][j] += start_of_gradient * devaldmg * king;
                gradient.psqt_king[EG][i][j] += start_of_gradient * devaldeg * king;
            }
        }

        //Rook
        {
            let x = (pos.trace.rook_on_open[WHITE] - pos.trace.rook_on_open[BLACK]) as f64;
            let y = (pos.trace.rook_on_seventh[WHITE] - pos.trace.rook_on_seventh[BLACK]) as f64;

            gradient.rook_on_open[MG] += start_of_gradient * devaldmg * x;
            gradient.rook_on_open[EG] += start_of_gradient * devaldeg * x;

            gradient.rook_on_seventh[MG] += start_of_gradient * devaldmg * y;
            gradient.rook_on_seventh[EG] += start_of_gradient * devaldeg * y;
        }
        //Piece values
        {
            let pawns = (pos.trace.pawns[WHITE] - pos.trace.pawns[BLACK]) as f64;
            let knights = (pos.trace.knights[WHITE] - pos.trace.knights[BLACK]) as f64;
            let bishops = (pos.trace.bishops[WHITE] - pos.trace.bishops[BLACK]) as f64;
            let bishop_pairs =
                (pos.trace.bishop_bonus[WHITE] - pos.trace.bishop_bonus[BLACK]) as f64;
            let rooks = (pos.trace.rooks[WHITE] - pos.trace.rooks[BLACK]) as f64;
            let queens = (pos.trace.queens[WHITE] - pos.trace.queens[BLACK]) as f64;

            gradient.pawn_piece_value[MG] += start_of_gradient * devaldmg * pawns;
            gradient.pawn_piece_value[EG] += start_of_gradient * devaldeg * pawns;

            gradient.knight_piece_value[MG] += start_of_gradient * devaldmg * knights;
            gradient.knight_piece_value[EG] += start_of_gradient * devaldeg * knights;

            gradient.knight_value_with_pawns[pos.trace.knight_value_with_pawns as usize] +=
                start_of_gradient * knights;

            gradient.bishop_piece_value[MG] += start_of_gradient * devaldmg * bishops;
            gradient.bishop_piece_value[EG] += start_of_gradient * devaldeg * bishops;

            gradient.bishop_pair[MG] += start_of_gradient * devaldmg * bishop_pairs;
            gradient.bishop_pair[EG] += start_of_gradient * devaldeg * bishop_pairs;

            gradient.rook_piece_value[MG] += start_of_gradient * devaldmg * rooks;
            gradient.rook_piece_value[EG] += start_of_gradient * devaldeg * rooks;

            gradient.queen_piece_value[MG] += start_of_gradient * devaldmg * queens;
            gradient.queen_piece_value[EG] += start_of_gradient * devaldeg * queens;
        }
        //Diagonally adjacent
        for i in 0..5 {
            let x = (pos.trace.diagonally_adjacent_squares_withpawns[WHITE][i]
                - pos.trace.diagonally_adjacent_squares_withpawns[BLACK][i])
                as f64;
            gradient.diagonally_adjacent_squares_withpawns[MG][i] +=
                start_of_gradient * devaldmg * x;
            gradient.diagonally_adjacent_squares_withpawns[EG][i] +=
                start_of_gradient * devaldeg * x;
        }
        //Mobility
        for i in 0..9 {
            let x =
                (pos.trace.knight_mobility[WHITE][i] - pos.trace.knight_mobility[BLACK][i]) as f64;
            gradient.knight_mobility[MG][i] += start_of_gradient * devaldmg * x;
            gradient.knight_mobility[EG][i] += start_of_gradient * devaldeg * x;
        }
        for i in 0..14 {
            let x =
                (pos.trace.bishop_mobility[WHITE][i] - pos.trace.bishop_mobility[BLACK][i]) as f64;
            gradient.bishop_mobility[MG][i] += start_of_gradient * devaldmg * x;
            gradient.bishop_mobility[EG][i] += start_of_gradient * devaldeg * x;
        }
        for i in 0..15 {
            let x = (pos.trace.rook_mobility[WHITE][i] - pos.trace.rook_mobility[BLACK][i]) as f64;
            gradient.rook_mobility[MG][i] += start_of_gradient * devaldmg * x;
            gradient.rook_mobility[EG][i] += start_of_gradient * devaldeg * x;
        }
        for i in 0..28 {
            let x =
                (pos.trace.queen_mobility[WHITE][i] - pos.trace.queen_mobility[BLACK][i]) as f64;
            gradient.queen_mobility[MG][i] += start_of_gradient * devaldmg * x;
            gradient.queen_mobility[EG][i] += start_of_gradient * devaldeg * x;
        }

        //Safety
        gradient.attack_weight[pos.trace.attackers[WHITE] as usize] += start_of_gradient / 100.0
            * tuner.params.safety_table.safety_table[pos.trace.attacker_value[WHITE] as usize];
        gradient.safety_table.safety_table[pos.trace.attacker_value[WHITE] as usize] +=
            start_of_gradient / 100.0
                * tuner.params.attack_weight[pos.trace.attackers[WHITE] as usize];
        gradient.attack_weight[pos.trace.attackers[BLACK] as usize] -= start_of_gradient / 100.0
            * tuner.params.safety_table.safety_table[pos.trace.attacker_value[BLACK] as usize];
        gradient.safety_table.safety_table[pos.trace.attacker_value[BLACK] as usize] +=
            start_of_gradient / 100.0
                * tuner.params.attack_weight[pos.trace.attackers[BLACK] as usize];
    }
    gradient
}
pub fn texel_tuning(tuner: &mut Tuner) {
    let mut best_error = average_evaluation_error(&tuner);
    println!("Error in epoch 0: {}", best_error);
    let mut epoch = 0;
    let mut lr = 23.0;
    loop {
        epoch += 1;
        shuffle_positions(tuner);
        for batch in 0..(tuner.positions.len() - 1) / BATCH_SIZE + 1 {
            let from = batch * BATCH_SIZE;
            let mut to = (batch + 1) * BATCH_SIZE;
            if to > tuner.positions.len() {
                to = tuner.positions.len();
            }
            let gradient = calculate_gradient(tuner, from, to, lr);
            tuner.params.apply_gradient(&gradient);
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
    let mut lr = 0.05;
    loop {
        epoch += 1;
        //Shuffle positions
        shuffle_positions(tuner);
        //Calculate dE/dk
        for batch in 0..(tuner.positions.len() - 1) / BATCH_SIZE + 1 {
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
