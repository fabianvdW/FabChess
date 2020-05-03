use core_sdk::board_representation::game_state::GameState;
use core_sdk::move_generation::makemove::{copy_make, make_move, unmake_move};
use core_sdk::move_generation::movegen2;
use core_sdk::search::cache::Cache;
use core_sdk::search::reserved_memory::ReservedMoveList;
use core_sdk::search::searcher::{search_move, InterThreadCommunicationSystem};
use core_sdk::search::timecontrol::TimeControl;
use extended_sdk::misc::to_string_board;
use extended_sdk::pgn::pgn_reader::parse_move;
use std::io;
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let state = GameState::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/P7/1PP1NnPP/RNBQK2R b KQ - 0 8");
    println!("{}", state);
    let mv = parse_move(&state, "f2d3");
    let mut after = copy_make(&state, mv.0);
    println!("{}", after);
    println!(
        "{}",
        to_string_board(
            after.square_attackers(after.king_square(after.color_to_move), after.all_pieces())
        )
    );
    let stdin = io::stdin();
    let mut line = String::new();
    let mut state =
        GameState::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    loop {
        line.clear();
        stdin.read_line(&mut line).ok().unwrap();
        let arg: Vec<&str> = line.split_whitespace().collect();
        match arg[0].trim() {
            "position" => {
                state = GameState::from_fen(&arg[1..].join(" "));
            }
            "perft" => {
                let depth = arg[1].parse::<usize>().unwrap();
                perft_div(&mut state.clone(), depth);
            }
            _ => continue,
        }
    }
    //go_infinite_from_startpos();
}
pub fn perft_div(g: &mut GameState, depth: usize) -> u64 {
    let mut count = 0u64;
    let mut movelist = ReservedMoveList::default();
    let now = Instant::now();
    movegen2::generate_pseudolegal_moves(&g, &mut movelist.move_lists[depth]);
    let len = movelist.move_lists[depth].move_list.len();
    for i in 0..len {
        let gmv = movelist.move_lists[depth].move_list[i];
        if g.is_valid_move(gmv.0) {
            let irr = make_move(g, gmv.0);
            let res = perft(g, depth - 1, &mut movelist);
            unmake_move(g, gmv.0, irr);
            println!("{:?}: {}", gmv.0, res);
            count += res;
        }
    }
    println!("{}", count);
    let after = Instant::now();
    let dur = after.duration_since(now);
    let secs = dur.as_millis() as f64 / 1000.0;
    println!(
        "{}",
        &format!("Time {} ({} nps)", secs, count as f64 / secs)
    );
    count
}

pub fn perft(g: &mut GameState, depth: usize, movelist: &mut ReservedMoveList) -> u64 {
    if depth == 1 {
        movegen2::generate_pseudolegal_moves(&g, &mut movelist.move_lists[depth]);
        let mut res = 0u64;
        for mv in movelist.move_lists[depth].move_list.iter() {
            if g.is_valid_move(mv.0) {
                res += 1;
            }
        }
        res
    } else {
        if depth == 0 {
            return 1;
        }
        let mut res = 0;
        let _ = movegen2::generate_pseudolegal_moves(&g, &mut movelist.move_lists[depth]);
        let len = movelist.move_lists[depth].move_list.len();
        for i in 0..len {
            let mv = movelist.move_lists[depth].move_list[i].0;
            if g.is_valid_move(mv) {
                let before = g.clone();
                let irr = make_move(g, mv);
                let after_make = g.clone();
                unmake_move(g, mv, irr);
                if before != *g {
                    panic!("")
                }
                let irr = make_move(g, mv);
                res += perft(g, depth - 1, movelist);
                unmake_move(g, mv, irr);
            }
        }
        res
    }
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
