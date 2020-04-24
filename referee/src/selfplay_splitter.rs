use crate::engine::{EndConditionInformation, Engine};
use crate::engine::{PlayTask, TaskResult};
use crate::logging::FileLogger;
use crate::openings::{load_db_until, load_openings_into_queue};
use crate::queue::ThreadSafeQueue;
use crate::selfplay::play_game;
use crate::Config;
use core_sdk::board_representation::game_state::*;
use core_sdk::search::timecontrol::TimeControl;
use extended_sdk::pgn::pgn_writer::*;
use std::cmp::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn start_self_play(config: Config) {
    let tcp1 = TimeControl::Incremental(
        config.timecontrol_engine_time,
        config.timecontrol_engine_inc,
    );
    let mut gauntlet_engine = Engine::from_path(
        &config.engine_path.0,
        999,
        tcp1,
        config.engine_path.1.clone(),
    );
    let tcp2 = TimeControl::Incremental(
        config.timecontrol_enemies_time,
        config.timecontrol_enemies_inc,
    );
    let mut engines: Vec<Engine> = Vec::new();
    for (index, path) in config.enemies_paths.into_iter().enumerate() {
        engines.push(Engine::from_path(&path.0, index, tcp2.clone(), path.1));
    }
    let mut db: Vec<GameState> = Vec::with_capacity(100_000);
    let mut db_sequences: Vec<Vec<GameMove>> = Vec::with_capacity(100_000);
    for database in config.opening_databases {
        let mut database_loaded = load_db_until(&database, config.opening_load_untilply);
        db.append(&mut database_loaded.0);
        db_sequences.append(&mut database_loaded.1);
    }
    println!(
        "{}",
        &format!(
            "Loaded database with {} games found! Preparing games...",
            db.len()
        )
    );
    let queue: Arc<ThreadSafeQueue<PlayTask>> = Arc::new(load_openings_into_queue(
        config.games / 2,
        db,
        db_sequences,
        &gauntlet_engine,
        &engines,
    ));
    let games = queue.len();
    println!("Prepared {} games! Starting...", games);

    let result_queue: Arc<ThreadSafeQueue<TaskResult>> =
        Arc::new(ThreadSafeQueue::new(Vec::with_capacity(100)));
    FileLogger::new("referee_error_log.txt", false)
        .init()
        .expect("Could not create File Logger");
    let pgn_log = FileLogger::new("pgns.pgn", true);

    //Start all childs
    let mut childs = Vec::with_capacity(config.processors);
    for _ in 0..config.processors {
        let queue_clone = queue.clone();
        let res_clone = result_queue.clone();
        childs.push(thread::spawn(move || {
            start_self_play_thread(queue_clone, res_clone);
        }));
    }

    //Collect results
    let mut results_collected = 0;
    while results_collected < games {
        thread::sleep(Duration::from_millis(50));
        if let Some(mut result) = result_queue.pop() {
            results_collected += 1;
            println!("*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*");
            println!("Game {} finished!", result.task.id);
            if let Some(reason) = result.endcondition {
                println!("Reason: {}", reason);
            } else {
                println!("Reason: Disqualification");
            }
            println!("*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*");
            //Add engines
            gauntlet_engine.add(&result.task.engine1);
            engines[result.task.engine2.id].add(&result.task.engine2);

            println!("-------------------------------------------------");
            let (rank, descr, _) = gauntlet_engine.get_elo_gain();
            println!("{}", rank);
            let mut other: Vec<(String, String, f64)> = Vec::with_capacity(engines.len());
            for engine in &engines {
                other.push(engine.get_elo_gain());
            }
            other.sort_by(|a, b| {
                if a.2 > b.2 {
                    Ordering::Less
                } else if (a.2 - b.2).abs() < std::f64::EPSILON {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }
            });
            for desc in &other {
                println!("{}", desc.0);
            }
            println!("-------------------------------------------------");
            if (results_collected + 1) % 5 == 0 {
                println!("+++++++++++++++++++++++++++++++++++++++++++++++++");
                println!("{}", descr);
                for desc in &other {
                    println!("{}", desc.1);
                }
                println!("+++++++++++++++++++++++++++++++++++++++++++++++++");
            }

            //Write all fens of game to pgn
            let opening_moves = Some(result.task.opening_sequence.len());
            let mut moves = result.task.opening_sequence;
            if !result.move_sequence.is_empty() {
                moves.append(&mut result.move_sequence);
                let mut metadata = PGNMetadata::default();
                metadata.fill_systemdata();
                metadata.event_name = Some("FabChess local gauntlet".to_owned());
                metadata.round = Some(format!("{}", result.task.id));
                metadata.result = Some(result.final_status.to_string());
                metadata.termination = Some(if result.endcondition.is_none() {
                    "rules infraction".to_owned()
                } else {
                    let temp = result.endcondition.unwrap();
                    match temp {
                        EndConditionInformation::DrawByadjudication
                        | EndConditionInformation::MateByadjudication => "adjudication",
                        _ => "normal",
                    }
                    .to_owned()
                });
                metadata.white = Some(if result.task.p1_is_white {
                    result.task.engine1.name.clone()
                } else {
                    result.task.engine2.name.clone()
                });
                metadata.black = Some(if result.task.p1_is_white {
                    result.task.engine2.name.clone()
                } else {
                    result.task.engine1.name.clone()
                });
                pgn_log.dump_msg(&get_pgn_string(&metadata, moves, opening_moves));
            }
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
        println!("Starting game {}", task.id);
        let res = play_game(task);
        if res.endcondition.is_none() {
            thread::sleep(Duration::from_millis(150));
        }
        result_queue.push(res);
    }
}
