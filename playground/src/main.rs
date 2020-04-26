use core_sdk::board_representation::game_state::GameState;
use core_sdk::search::cache::Cache;
use core_sdk::search::searcher::{search_move, InterThreadCommunicationSystem};
use core_sdk::search::timecontrol::TimeControl;
use std::sync::Arc;

fn main() {
    //go_infinite_from_startpos();
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
