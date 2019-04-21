#[macro_use]
extern crate lazy_static;
extern crate rand;

pub mod board_representation;
pub mod misc;
pub mod bitboards;
pub mod move_generation;
pub mod evaluation;
pub mod logging;
pub mod search;
pub mod uci;

use self::board_representation::game_state::GameState;
use self::move_generation::movegen;

pub fn perft_div(g: &GameState, depth: usize) -> u64 {
    let mut count = 0u64;
    let (valid_moves, _in_check) = movegen::generate_moves(&g);
    for mv in valid_moves {
        let next_g = movegen::make_move(&g, &mv);
        let res = perft(&next_g, depth - 1);
        println!("{:?}: {}", mv, res);
        count += res;
    }
    count
}

pub fn perft(g: &GameState, depth: usize) -> u64 {
    if depth == 1 {
        let (vm, _ic) = movegen::generate_moves(&g);
        return vm.len() as u64;
    } else {
        if depth == 0 {
            return 1;
        }
        let mut res = 0;
        let (valid_moves, _incheck) = movegen::generate_moves(&g);
        for mv in valid_moves {
            res += perft(&movegen::make_move(&g, &mv), depth - 1);
        }
        res
    }
}