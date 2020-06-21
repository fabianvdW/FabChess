use core_sdk::board_representation::game_state::GameState;
use core_sdk::evaluation::nn::{get_evaluation_parameters, nn_evaluate_game_state};
use core_sdk::evaluation::nn_trace::NNTrace;
use ndarray::Array;

fn main() {
    let nn = get_evaluation_parameters();
    let input = Array::ones(676);
    println!("Nn: {}", nn(&input));
    let state = GameState::standard();
    println!(
        "{}",
        nn_evaluate_game_state(&nn, &state, &mut NNTrace::new()).final_eval
    );
    let state = GameState::from_fen("5b2/1p1k4/p7/4n3/3B1Npn/2N1P3/PP1K2P1/8 b - - 8 34");
    println!(
        "{}",
        nn_evaluate_game_state(&nn, &state, &mut NNTrace::new()).final_eval
    );
}
