use benchmarking::*;
use core_sdk::evaluation::eval_game_state;
use core_sdk::move_generation::makemove::make_move;
use core_sdk::move_generation::movegen::{self, MoveList};
use criterion::{criterion_group, criterion_main, Criterion};

pub fn evaluation_bench(c: &mut Criterion) {
    let states = load_benchmarking_positions();
    c.bench_function("evaluation", |b| {
        b.iter(|| {
            let mut sum = 0;
            for i in 0..BENCHMARKING_POSITIONS_AMOUNT {
                sum += eval_game_state(&states[i], -16000, 16000).final_eval as isize;
            }
            sum
        })
    });
}

pub fn generate_moves_bench(c: &mut Criterion) {
    let states = load_benchmarking_positions();
    let mut movelist = MoveList::default();
    c.bench_function("movegen", |b| {
        b.iter(|| {
            let mut sum = 0;
            for i in 0..BENCHMARKING_POSITIONS_AMOUNT {
                movegen::generate_moves(&states[i], false, &mut movelist);
                sum += movelist.move_list.len();
                for mv in movelist.move_list.iter() {
                    let g = make_move(&states[i], mv.0);
                    sum += (g.get_hash() & 0xFF) as usize;
                }
            }
            sum
        })
    });
}
criterion_group!(benches, evaluation_bench, generate_moves_bench);
criterion_main!(benches);
