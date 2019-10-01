use super::queue::ThreadSafeQueue;
use crate::board_representation::game_state::*;
use crate::move_generation::movegen;
use crate::pgn::pgn_reader::{GameParser, PGNParser};
use crate::testing::{Engine, PlayTask};
use rand::Rng;
use std::fs::File;
use std::io::BufReader;

pub fn load_db_until(db: &str, until: usize) -> (Vec<GameState>, Vec<Vec<GameMove>>) {
    let movelist = movegen::MoveList::default();
    let attack_container =
        crate::board_representation::game_state_attack_container::GameStateAttackContainer::default(
        );
    let mut res: Vec<GameState> = Vec::with_capacity(100_000);
    let mut res_mvs = Vec::with_capacity(100_000);
    let res_file = File::open(db).expect("Unable to open opening database");
    let reader = BufReader::new(res_file);
    let parser = GameParser {
        pgn_parser: PGNParser { reader },
        is_opening: true,
        opening_load_untilply: until,
        move_list: movelist,
        attack_container,
    };
    for game in parser {
        if game.1.len() > until {
            let state: GameState = game.1[until].clone();
            res.push(state);
            res_mvs.push(game.0[..until].to_vec());
        }
    }
    (res, res_mvs)
}

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
    for other in queue {
        if other.opening.hash == state.hash {
            return true;
        }
    }
    false
}
