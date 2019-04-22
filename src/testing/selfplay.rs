use crate::queue::ThreadSafeQueue;
use core::board_representation::game_state::GameState;
use core::misc::{GameParser, PGNParser};
use rand::Rng;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
pub fn start_self_play(
    p1: &str,
    p2: &str,
    processors: usize,
    games: usize,
    opening_db: &str,
    opening_load_until: usize,
) {
    let db: Vec<GameState> = load_db_until(opening_db, opening_load_until);
    println!("Loaded database! Preparing games...");
    let queue: Arc<ThreadSafeQueue<PlayTask>> = Arc::new(load_openings_into_queue(games / 2, db));
    println!("Games prepared! Starting...");
    let result_queue: Arc<ThreadSafeQueue<TaskResult>> =
        Arc::new(ThreadSafeQueue::new(Vec::with_capacity(100)));
    let mut childs = Vec::with_capacity(processors);
    for _ in 0..processors {
        let queue_clone = queue.clone();
        let res_clone = result_queue.clone();
        childs.push(thread::spawn(move || {
            start_self_play_thread(queue_clone, res_clone);
        }));
    }
    let mut results_collected = 0;
    while results_collected < (games / 2) * 2 {
        thread::sleep(Duration::from_millis(50));
        if let Some(result) = result_queue.pop() {
            results_collected += 1;
            //Verarbeite Resultat
        }
    }
    for child in childs {
        child.join().expect("Couldn't join thread");
    }
    println!("Testing finished!");
}
pub fn start_self_play_thread(
    queue: Arc<ThreadSafeQueue<PlayTask>>,
    result_queue: Arc<ThreadSafeQueue<TaskResult>>,
) {
    while let Some(task) = queue.pop() {
        result_queue.push(TaskResult {
            p1_won: false,
            draw: false,
            p1_disq: false,
            p2_disq: false,
        });
        thread::sleep(Duration::from_millis(1000));
    }
}
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
    for _ in 0..n {
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
                });
                res.push(PlayTask {
                    opening: state,
                    p1_is_white: false,
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
pub struct PlayTask {
    pub opening: GameState,
    pub p1_is_white: bool,
}
pub struct TaskResult {
    pub p1_won: bool,
    pub draw: bool,
    pub p1_disq: bool,
    pub p2_disq: bool,
}
