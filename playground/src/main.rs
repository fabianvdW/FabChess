use core_sdk::bitboards::print_castle_permisssion;
use core_sdk::board_representation::game_state::{GameMoveType, GameState};
use core_sdk::evaluation::eval_game_state;
use core_sdk::move_generation::movegen2;
use core_sdk::search::cache::{Cache, CacheEntry};
use core_sdk::search::moveordering::mvvlva;
use core_sdk::search::quiescence::see;
use core_sdk::search::searcher::{search_move, InterThreadCommunicationSystem};
use core_sdk::search::timecontrol::TimeControl;
use std::fs;
use std::sync::Arc;

fn main() {
    let pos = load_benchmarking_positions();
    let mut sum: i32 = 0;
    let mut see_buffer = vec![0; 32];
    for p in pos {
        let eval = eval_game_state(&p, 0, 0);
        println!("{}", eval.final_eval);
        sum += eval.final_eval as i32;
        let moves = movegen2::generate_legal_moves(&p);
        for mv in moves.move_list.iter() {
            sum += CacheEntry::mv_to_u16(mv.0) as i32;
            sum += p.irreversible.hash as i32;
            if mv.0.is_capture() && mv.0.move_type != GameMoveType::EnPassant {
                sum += see(&p, mv.0, true, &mut see_buffer) as i32;
                sum += mvvlva(mv.0) as i32;
            }
        }
    }
    println!("{}", sum);
    println!(
        "{}",
        core_sdk::board_representation::zobrist_hashing::init_zobrist()
    );
    //print_castle_permisssion();
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
