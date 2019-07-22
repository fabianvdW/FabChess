extern crate core;
extern crate rand;

use core::evaluation::eval_game_state;
use core::tuning::loading::{load_positions, FileFormatSupported, LabelledGameState, Statistics};
use core::tuning::parameters::Parameters;
use rand::{seq::SliceRandom, thread_rng};

pub const POSITION_FILE: &str = "D:/FenCollection/Real/all_positions_qsearch.txt";
pub const PARAM_FILE: &str = "D:/FenCollection/Tuning/";
//pub const POSITION_FILE: &str = "D:/FenCollection/Test/all_positions_qsearch.txt";
const BATCH_SIZE: usize = 8196;
pub fn main() {
    if !cfg!(feature = "texel-tuning") {
        panic!("Feature texel-tuning has to be enabled");
    }
    //Step 1. Load all positions from a file. Those positions should already be the q-searched positions.
    let mut stats = Statistics::new();
    let mut positions: Vec<LabelledGameState> = Vec::with_capacity(8000000);
    load_positions(
        POSITION_FILE,
        FileFormatSupported::OwnEncoding,
        &mut positions,
        &mut stats,
    );
    println!(
        "Loaded file {} with {} positions!",
        POSITION_FILE,
        positions.len()
    );
    let mut tuner = Tuner {
        k: 0.624,
        positions: init_texel_states(positions),
    };
    println!("Started Tuning!");
    minimize_evaluation_error_fork(&mut tuner);
    println!("Optimal K: {}", tuner.k);
    let params = Parameters::default();
    params.write_to_file(&format!("{}tune.txt", PARAM_FILE));
}
pub fn init_texel_states(labelledstates: Vec<LabelledGameState>) -> Vec<TexelState> {
    let mut res: Vec<TexelState> = Vec::with_capacity(labelledstates.len());
    for state in labelledstates {
        let eval = eval_game_state(&state.game_state);
        res.push(TexelState {
            lgs: state,
            eval: eval.final_eval as f64,
        });
    }
    res
}
pub struct TexelState {
    pub lgs: LabelledGameState,
    pub eval: f64,
}
pub struct Tuner {
    pub k: f64,
    pub positions: Vec<TexelState>,
}

pub fn shuffle_positions(tuner: &mut Tuner) {
    tuner.positions.shuffle(&mut thread_rng());
}
pub fn average_evaluation_error(tuner: &Tuner) -> f64 {
    let mut res = 0.;
    for pos in &tuner.positions {
        res += (pos.lgs.label - sigmoid(tuner.k, pos.eval)).powf(2.0);
    }
    res / tuner.positions.len() as f64
}

pub fn minimize_evaluation_error_fork(tuner: &mut Tuner) -> f64 {
    let mut best_k = tuner.k;
    let mut best_error = average_evaluation_error(&tuner);
    println!("Erorr in epoch 0: {}", best_error);
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
                dedk += (pos.lgs.label - sigmoid(tuner.k, eval)) * dsigmoiddk(tuner.k, eval);
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
