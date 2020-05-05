use core_sdk::board_representation::game_state::GameState;
use core_sdk::evaluation::eval_game_state;
use core_sdk::move_generation::makemove::{copy_make, make_move, unmake_move};
use core_sdk::move_generation::movegen2;
use core_sdk::search::cache::Cache;
use core_sdk::search::reserved_memory::ReservedMoveList;
use core_sdk::search::searcher::{search_move, InterThreadCommunicationSystem};
use core_sdk::search::timecontrol::TimeControl;
use extended_sdk::misc::to_string_board;
use extended_sdk::pgn::pgn_reader::parse_move;
use std::sync::Arc;
use std::time::Instant;
use std::{fs, io};

fn main() {
    let pos = load_benchmarking_positions();
    let mut sum: i16 = 0;
    for p in pos {
        let eval = eval_game_state(&p, 0, 0);
        println!("{}", eval.final_eval);
        sum += eval.final_eval;
    }
    println!("{}", sum);
    //go_infinite_from_startpos();
}
pub const BENCHMARKING_POSITIONS: &str = "./benchmarking/benchmarking_positions.txt";
pub const BENCHMARKING_POSITIONS_AMOUNT: usize = 1000;
pub fn load_benchmarking_positions() -> Vec<GameState> {
    let mut states = Vec::with_capacity(BENCHMARKING_POSITIONS_AMOUNT);
    let positions =
        fs::read_to_string(BENCHMARKING_POSITIONS).expect("Unable to read benchmarking positions");
    let new_linesplit = positions.split("\n").collect::<Vec<&str>>();
    for i in 0..BENCHMARKING_POSITIONS_AMOUNT {
        states.push(GameState::from_fen(new_linesplit[i]));
    }
    states
}

fn go_infinite_from_startpos() {
    let itcs = Arc::new(InterThreadCommunicationSystem::default());
    *itcs.cache() =
        Cache::with_size_threaded(itcs.uci_options().hash_size, itcs.uci_options().threads);
    InterThreadCommunicationSystem::update_thread_count(&itcs, 1);
    search_move(
        itcs,
        100,
        GameState::standard(),
        Vec::new(),
        TimeControl::Infinite,
    );
}
