use core_sdk::board_representation::game_state::GameState;
use core_sdk::move_generation::makemove::make_move;
use core_sdk::move_generation::movegen::{bishop_attacks, rook_attacks};
use core_sdk::move_generation::movegen2;
use core_sdk::search::cache::Cache;
use core_sdk::search::reserved_memory::ReservedMoveList;
use core_sdk::search::searcher::{search_move, InterThreadCommunicationSystem};
use core_sdk::search::timecontrol::TimeControl;
use extended_sdk::misc::to_string_board;
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let state = GameState::standard();
    println!("{}", state);
    perft_div(&state, 6);
    //go_infinite_from_startpos();
}
pub fn perft_div(g: &GameState, depth: usize) -> u64 {
    let mut count = 0u64;
    let mut movelist = ReservedMoveList::default();
    let now = Instant::now();
    movegen2::generate_pseudolegal_moves(&g, &mut movelist.move_lists[depth]);
    let len = movelist.move_lists[depth].move_list.len();
    for i in 0..len {
        let gmv = movelist.move_lists[depth].move_list[i];
        if (g.is_valid_move(gmv.0)) {
            let next_g = make_move(&g, gmv.0);
            let res = perft(&next_g, depth - 1, &mut movelist);
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

pub fn perft(g: &GameState, depth: usize, movelist: &mut ReservedMoveList) -> u64 {
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
                res += perft(&make_move(&g, mv), depth - 1, movelist);
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
