use crate::board_representation::game_state::GameState;
use crate::search::cache::{DEFAULT_HASH_SIZE, DEFAULT_LOCKS};
use crate::search::searcher::{DEFAULT_SKIP_RATIO, DEFAULT_THREADS};
use crate::search::timecontrol::DEFAULT_MOVE_OVERHEAD;

#[derive(Copy, Clone)]
pub struct UCIOptions {
    pub hash_size: usize,
    pub hash_locks: usize,
    pub threads: usize,
    pub move_overhead: u64,
    pub debug_print: bool,
    pub skip_ratio: usize,
}
pub struct UCIEngine<'a> {
    pub name: &'a str,
    pub author: &'a str,
    pub internal_state: GameState,
    pub options: UCIOptions,
}

impl<'a> UCIEngine<'a> {
    pub fn standard() -> UCIEngine<'a> {
        UCIEngine {
            name: &"FabChessDev v1.13",
            author: &"Fabian von der Warth, Contributor: Erik Imgrund",
            internal_state: GameState::standard(),
            options: UCIOptions {
                hash_size: DEFAULT_HASH_SIZE,
                hash_locks: DEFAULT_LOCKS,
                threads: DEFAULT_THREADS,
                move_overhead: DEFAULT_MOVE_OVERHEAD,
                debug_print: false,
                skip_ratio: DEFAULT_SKIP_RATIO,
            },
        }
    }

    pub fn id_command(&self) {
        println!("id name {}", self.name);
        println!("id author {}", self.author);
    }
}
