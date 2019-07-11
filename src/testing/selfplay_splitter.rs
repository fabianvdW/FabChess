use crate::openings::{load_db_until, load_openings_into_queue};
use crate::queue::ThreadSafeQueue;
use crate::selfplay::{play_game, EndConditionInformation};
use core::board_representation::game_state::GameState;
use core::logging::Logger;
use core::move_generation::movegen;
use core::search::search::TimeControl;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn start_self_play(
    p1: &str,
    p2: &str,
    processors: usize,
    games: usize,
    opening_db: Vec<String>,
    opening_load_until: usize,
    tcp1: TimeControl,
    tcp2: TimeControl,
) {
    let mut db: Vec<GameState> = vec![];
    for database in opening_db {
        db.append(&mut load_db_until(&database, opening_load_until));
    }
    println!(
        "{}",
        &format!(
            "Loaded database with {} games found! Preparing games...",
            db.len()
        )
    );
    let queue: Arc<ThreadSafeQueue<PlayTask>> = Arc::new(load_openings_into_queue(games / 2, db));
    println!("Games prepared! Starting...");
    let result_queue: Arc<ThreadSafeQueue<TaskResult>> =
        Arc::new(ThreadSafeQueue::new(Vec::with_capacity(100)));
    let error_log = Arc::new(Logger::new("referee_error_log.txt", false));
    let fen_log = Logger::new("fens.txt", true);
    let mut childs = Vec::with_capacity(processors);
    for _ in 0..processors {
        let queue_clone = queue.clone();
        let res_clone = result_queue.clone();
        let p1_clone = String::from_str(p1).unwrap();
        let p2_clone = String::from_str(p2).unwrap();
        let tcp1_clone = tcp1.clone();
        let tcp2_clone = tcp2.clone();
        let log_clone = error_log.clone();
        childs.push(thread::spawn(move || {
            start_self_play_thread(
                queue_clone,
                res_clone,
                p1_clone,
                p2_clone,
                &tcp1_clone,
                &tcp2_clone,
                log_clone,
            );
        }));
    }
    let mut results_collected = 0;
    let mut p1_wins = 0;
    let mut p2_wins = 0;
    let mut draws = 0;
    let mut p1_disqs = 0;
    let mut p2_disqs = 0;
    while results_collected < (games / 2) * 2 {
        thread::sleep(Duration::from_millis(50));
        if let Some(result) = result_queue.pop() {
            results_collected += 1;
            //Verarbeite Resultat
            println!("*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*");
            println!("Game {} finished!", result.task_id);
            if let Some(reason) = result.endcondition {
                println!("Reason: {}", reason);
            } else {
                println!("Reason: Disqualification");
            }
            if !result.p1_disq && !result.p2_disq {
                println!("Player    Depth    NPS               TimeLeft");
                println!(
                    "P1         {:.2}      {:.2}     {}",
                    result.depth_p1, result.nps_p1, result.time_left_p1
                );
                println!(
                    "P2         {:.2}      {:.2}     {}",
                    result.depth_p2, result.nps_p2, result.time_left_p2
                );
            }
            println!("*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*");
            if !result.p1_disq && !result.p2_disq {
                if result.draw {
                    draws += 1;
                } else if result.p1_won {
                    p1_wins += 1;
                } else {
                    p2_wins += 1;
                }
            }
            if result.p1_disq {
                p1_disqs += 1;
            }
            if result.p2_disq {
                p2_disqs += 1;
            }
            //Make some statistics
            let mut elo_gain_p1 = 0.0;
            let mut elo_plus_p1 = 0.0;
            //let mut elo_minus_p1 = 0.0;
            if p1_wins != 0 && p2_wins != 0 || draws != 0 {
                //Derived from 1. E_A= 1/(1+10^(-DeltaElo/400)) and 2. |X/N-p|<=1.96*sqrt(N*p*(1-p))/n
                let n: f64 = (p1_wins + p2_wins + draws) as f64;
                let x_a: f64 = p1_wins as f64 + draws as f64 / 2.0;
                let p_a: f64 = x_a / n;
                let k: f64 = (1.96 * 1.96 + 2.0 * x_a) / (-1.0 * 1.96 * 1.96 - n);
                let q = -1.0 * x_a * x_a / (n * (-1.96 * 1.96 - n));
                let root = ((k / 2.0) * (k / 2.0) - q).sqrt();
                let p_a_upper: f64 = -k / 2.0 + root;
                //let p_a_lower: f64 = -k / 2.0 - root;
                /*println!("N: {}", n);
                println!("X_A: {}", x_a);
                println!("P_A: {}", p_a);
                println!("P_A_Upper: {}", p_a_upper);
                println!("P_A_Lower: {}", p_a_lower);*/
                elo_gain_p1 = get_elo_gain(p_a);
                elo_plus_p1 = get_elo_gain(p_a_upper) - elo_gain_p1;
                //elo_minus_p1 = elo_gain_p1 - get_elo_gain(p_a_lower);
            }
            println!("-------------------------------------------------");
            println!("Player   Wins   Draws   Losses   Elo   +/-   Disq.");
            println!(
                "P1       {}     {}      {}     {:.2}   {:.2}    {}",
                p1_wins, draws, p2_wins, elo_gain_p1, elo_plus_p1, p1_disqs
            );
            println!(
                "P2       {}     {}      {}     {:.2}   {:.2}    {}",
                p2_wins, draws, p1_wins, -elo_gain_p1, elo_plus_p1, p2_disqs
            );
            println!("-------------------------------------------------");

            //Write all fens of game to String
            if result.fen_history.len() > 0 {
                let mut game_string = String::new();
                game_string.push_str("New Game:\n");
                for fen in result.fen_history {
                    game_string.push_str(&format!(
                        "{} |{}\n",
                        fen,
                        if result.draw {
                            "Draw"
                        } else if result.white_win {
                            "White"
                        } else {
                            "Black"
                        }
                    ));
                }
                fen_log.log(&game_string, false);
            }
        }
    }
    for child in childs {
        child.join().expect("Couldn't join thread");
    }
    println!("Testing finished!");
}

pub fn get_elo_gain(p_a: f64) -> f64 {
    return -1.0 * (1.0 / p_a - 1.0).ln() * 400.0 / (10.0 as f64).ln();
}

pub fn start_self_play_thread(
    queue: Arc<ThreadSafeQueue<PlayTask>>,
    result_queue: Arc<ThreadSafeQueue<TaskResult>>,
    p1: String,
    p2: String,
    tcp1: &TimeControl,
    tcp2: &TimeControl,
    error_log: Arc<Logger>,
) {
    let mut movelist = movegen::MoveList::new();
    while let Some(task) = queue.pop() {
        println!("Starting game {}", task.id);
        let res = play_game(
            task,
            p1.clone(),
            p2.clone(),
            tcp1,
            tcp2,
            error_log.clone(),
            &mut movelist,
        );
        if res.p1_disq || res.p2_disq {
            thread::sleep(Duration::from_millis(150));
        }
        result_queue.push(res);
    }
}

pub struct PlayTask {
    pub opening: GameState,
    pub p1_is_white: bool,
    pub id: usize,
}

pub struct TaskResult {
    pub p1_won: bool,
    pub draw: bool,
    pub p1_disq: bool,
    pub p2_disq: bool,
    pub endcondition: Option<EndConditionInformation>,
    pub task_id: usize,
    pub fen_history: Vec<String>,
    pub white_win: bool,
    pub nps_p1: f64,
    pub depth_p1: f64,
    pub time_left_p1: usize,
    pub nps_p2: f64,
    pub depth_p2: f64,
    pub time_left_p2: usize,
}
