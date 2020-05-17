use core_sdk::board_representation::game_state::GameState;
use core_sdk::evaluation::parameters::Parameters;
use core_sdk::search::cache::Cache;
use core_sdk::search::searcher::{search_move, InterThreadCommunicationSystem};
use core_sdk::search::timecontrol::TimeControl;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

fn main() {
    /*for pattern in generate_rook_patterns(15).0 {
        if rook_attacks_slow(15, pattern.0) != rook_attack(15, pattern.0) {
            println!("{}", to_string_board(pattern.0));
            println!("{}", to_string_board(rook_attacks_slow(15, pattern.0)));
            println!("{}", to_string_board(rook_attack(15, pattern.0)));
            panic!("yup");
        }
    }*/
    //go_infinite_from_startpos();
    let param_string = format!("{}", Parameters::default());
    let param_file = Path::new("parameters.txt");
    let mut file = File::create(param_file).unwrap();
    write!(file, "{}", param_string).unwrap();
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
