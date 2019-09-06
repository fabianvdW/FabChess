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
use self::search::reserved_memory::{ReservedAttackContainer, ReservedMoveList};

pub fn perft_div(g: &GameState, depth: usize) -> u64 {
    let mut count = 0u64;
    let mut movelist = ReservedMoveList::default();
    let mut attack_container = ReservedAttackContainer::default();
    attack_container.attack_containers[depth].write_state(g);
    let _ = movegen::generate_moves(
        &g,
        false,
        &mut movelist.move_lists[depth],
        &attack_container.attack_containers[depth],
    );
    let mut index = 0;
    while index < movelist.move_lists[depth].counter {
        let mv = movelist.move_lists[depth].move_list[index].unwrap();
        let next_g = make_move(&g, &mv);
        let res = perft(&next_g, depth - 1, &mut movelist, &mut attack_container);
        println!("{:?}: {}", mv, res);
        count += res;
        index += 1;
    }
    count
}

pub fn perft(
    g: &GameState,
    depth: usize,
    movelist: &mut ReservedMoveList,
    attack_container: &mut ReservedAttackContainer,
) -> u64 {
    attack_container.attack_containers[depth].write_state(g);
    if depth == 1 {
        let _ = movegen::generate_moves(
            &g,
            false,
            &mut movelist.move_lists[depth],
            &attack_container.attack_containers[depth],
        );
        movelist.move_lists[depth].counter as u64
    } else {
        if depth == 0 {
            return 1;
        }
        let mut res = 0;
        let _ = movegen::generate_moves(
            &g,
            false,
            &mut movelist.move_lists[depth],
            &attack_container.attack_containers[depth],
        );
        let mut index = 0;
        while index < movelist.move_lists[depth].counter {
            let mv = movelist.move_lists[depth].move_list[index]
                .as_ref()
                .unwrap();
            res += perft(&make_move(&g, &mv), depth - 1, movelist, attack_container);
            index += 1;
        }
        res
    }
}
