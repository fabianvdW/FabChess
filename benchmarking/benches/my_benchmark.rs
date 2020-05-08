use benchmarking::*;
use core_sdk::board_representation::game_state_attack_container::GameStateAttackContainer;
use core_sdk::evaluation::eval_game_state;
use core_sdk::move_generation::makemove::make_move;
use core_sdk::move_generation::movegen::{self, MoveList};
use criterion::{criterion_group, criterion_main, Criterion};

pub fn evaluation_bench(c: &mut Criterion) {
    let states = load_benchmarking_positions();
    let mut attack_container = GameStateAttackContainer::default();
    c.bench_function("evaluation", |b| {
        b.iter(|| {
            let mut sum = 0;
            for i in 0..BENCHMARKING_POSITIONS_AMOUNT {
                attack_container.write_state(&states[i]);
                sum += eval_game_state(&states[i], &attack_container, -16000, 16000).final_eval
                    as isize;
            }
            sum
        })
    });
}

pub fn generate_moves_bench(c: &mut Criterion) {
    let states = load_benchmarking_positions();
    let mut attack_container = GameStateAttackContainer::default();
    let mut movelist = MoveList::default();
    c.bench_function("movegen", |b| {
        b.iter(|| {
            let mut sum = 0;
            for i in 0..BENCHMARKING_POSITIONS_AMOUNT {
                attack_container.write_state(&states[i]);
                movegen::generate_moves(&states[i], false, &mut movelist, &attack_container);
                sum += movelist.move_list.len();
                for mv in movelist.move_list.iter() {
                    let g = make_move(&states[i], mv.0);
                    sum += (g.hash & 0xFF) as usize;
                }
            }
            sum
        })
    });
}
criterion_group!(benches, evaluation_bench, generate_moves_bench);
criterion_main!(benches);
