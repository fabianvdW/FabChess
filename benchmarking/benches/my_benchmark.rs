use benchmarking::*;
use core_sdk::board_representation::game_state::Irreversible;
use core_sdk::evaluation::eval_game_state;
use core_sdk::move_generation::makemove::{make_move, unmake_move};
use core_sdk::move_generation::movegen2;
use core_sdk::move_generation::movelist::MoveList;
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
    let mut states = load_benchmarking_positions();
    let mut movelist = MoveList::default();
    c.bench_function("movegen", |b| {
        b.iter(|| {
            let mut sum = 0;
            for i in 0..BENCHMARKING_POSITIONS_AMOUNT {
                movegen2::generate_pseudolegal_moves(&states[i], &mut movelist);
                movelist.move_list.retain(|x| states[i].is_valid_move(x.0));
                for mv in movelist.move_list.iter() {
                    let mut irreversible = Irreversible::none();
                    make_move(&mut states[i], mv.0, &mut irreversible);
                    unmake_move(&mut states[i], mv.0, irreversible);
                }
                sum += movelist.move_list.len();
            }
            sum
        })
    });
}
criterion_group!(benches, evaluation_bench, generate_moves_bench);
criterion_main!(benches);
