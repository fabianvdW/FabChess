use crate::queue::ThreadSafeQueue;
use crate::selfplay_splitter::PlayTask;
use core::board_representation::game_state::GameState;
use core::misc::{GameParser, PGNParser};
use rand::Rng;
use std::fs::File;
use std::io::BufReader;

pub fn load_db_until(db: &str, until: usize) -> Vec<GameState> {
    let mut res: Vec<GameState> = Vec::with_capacity(100000);
    let res_file = File::open(db).expect("Unable to open opening database");
    let reader = BufReader::new(res_file);
    let parser = GameParser {
        pgn_parser: PGNParser { reader },
    };
    for game in parser {
        let state: GameState = game.1[until].clone();
        res.push(state);
    }
    res
}

pub fn load_openings_into_queue(n: usize, mut db: Vec<GameState>) -> ThreadSafeQueue<PlayTask> {
    let mut rng = rand::thread_rng();
    let mut res: Vec<PlayTask> = Vec::with_capacity(n);
    for i in 0..n {
        loop {
            if db.len() == 0 {
                panic!("There are not enough different openings in database! Use bigger database or load until higher ply!");
            }
            let index: usize = rng.gen_range(0, db.len());
            let state = db.remove(index);
            if !contains(&res, &state) {
                res.push(PlayTask {
                    opening: state.clone(),
                    p1_is_white: true,
                    id: 2 * i,
                });
                res.push(PlayTask {
                    opening: state,
                    p1_is_white: false,
                    id: 2 * i + 1,
                });
                break;
            }
        }
    }
    ThreadSafeQueue::new(res)
}

pub fn contains(queue: &Vec<PlayTask>, state: &GameState) -> bool {
    for other in queue {
        if other.opening.hash == state.hash {
            return true;
        }
    }
    false
}
