#[macro_use]
extern crate lazy_static;
extern crate rand;

pub mod bitboards;
pub mod board_representation;
pub mod evaluation;
pub mod logging;
pub mod misc;
pub mod move_generation;
pub mod search;
pub mod testing;
pub mod tuning;
pub mod uci;

use self::board_representation::game_state::GameState;
use self::move_generation::makemove::make_move;
use self::move_generation::movegen;

pub fn perft_div(g: &GameState, depth: usize) -> u64 {
    let mut count = 0u64;
    let mut movelist = movegen::MoveList::new();
    let _ = movegen::generate_moves(&g, false, &mut movelist, depth);
    let mut index = 0;
    while index < movelist.counter[depth] {
        let mv = movelist.move_list[depth][index].unwrap();
        let next_g = make_move(&g, &mv);
        let res = perft(&next_g, depth - 1, &mut movelist);
        println!("{:?}: {}", mv, res);
        count += res;
        index += 1;
    }
    count
}

pub fn perft(g: &GameState, depth: usize, movelist: &mut movegen::MoveList) -> u64 {
    if depth == 1 {
        let _ = movegen::generate_moves(&g, false, movelist, depth);
        return movelist.counter[depth] as u64;
    } else {
        if depth == 0 {
            return 1;
        }
        let mut res = 0;
        let _ = movegen::generate_moves(&g, false, movelist, depth);
        let mut index = 0;
        while index < movelist.counter[depth] {
            let mv = movelist.move_list[depth][index].as_ref().unwrap();
            res += perft(&make_move(&g, &mv), depth - 1, movelist);
            index += 1;
        }
        res
    }
}
