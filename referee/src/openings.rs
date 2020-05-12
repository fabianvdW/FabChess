use crate::engine::{Engine, PlayTask};
use crate::queue::ThreadSafeQueue;
use core_sdk::board_representation::game_state::*;
use rand::Rng;

pub fn load_openings_into_queue(
    n: usize,
    mut db: Vec<GameState>,
    mut db_sequences: Vec<Vec<GameMove>>,
    gauntlet_engine: &Engine,
    enemies: &[Engine],
) -> ThreadSafeQueue<PlayTask> {
    let mut rng = rand::thread_rng();
    let mut res: Vec<PlayTask> = Vec::with_capacity(n);
    let mut id = 0;
    for _ in 0..n {
        loop {
            if db.is_empty() {
                panic!("There are not enough different openings in database! Use bigger database or load until higher ply!");
            }
            let index: usize = rng.gen_range(0, db.len());
            let state = db.remove(index);
            let sequence = db_sequences.remove(index);
            if !contains(&res, &state) {
                for enemy_engine in enemies {
                    res.push(PlayTask {
                        opening: state.clone(),
                        opening_sequence: sequence.clone(),
                        p1_is_white: true,
                        id,
                        engine1: gauntlet_engine.clone(),
                        engine2: enemy_engine.clone(),
                    });
                    id += 1;
                    res.push(PlayTask {
                        opening: state.clone(),
                        opening_sequence: sequence.clone(),
                        p1_is_white: false,
                        id,
                        engine1: gauntlet_engine.clone(),
                        engine2: enemy_engine.clone(),
                    });
                    id += 1;
                }
                break;
            }
        }
    }
    ThreadSafeQueue::new(res)
}

pub fn contains(queue: &[PlayTask], state: &GameState) -> bool {
    queue
        .iter()
        .any(|other| other.opening.get_hash() == state.get_hash())
}
