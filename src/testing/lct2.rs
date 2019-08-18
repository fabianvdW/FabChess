use crate::write_to_buf;
use core::board_representation::game_state::{GameMove, GameState};
use core::misc::parse_move;
use core::move_generation::movegen;
use core::testing::queue::ThreadSafeQueue;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
pub fn start_lct2_thread(
    queue: Arc<ThreadSafeQueue<Lct2Test>>,
    p1: String,
    points: Arc<AtomicUsize>,
    thread_name: &str,
) {
    let mut child = Command::new(&p1.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute command!");
    let mut child_in = BufWriter::new(child.stdin.as_mut().unwrap());
    let mut child_out = BufReader::new(child.stdout.as_mut().unwrap());
    let mut line = String::new();
    while let Some(test) = queue.pop() {
        println!(
            "{}",
            &format!(
                "\n\n-----------------------------\nThread {} is looking at FEN: {}",
                thread_name,
                test.game_state.to_fen()
            )
        );
        println!("Bestmove is actually: {:?}", test.optimal_move);
        let str = format!(
            "ucinewgame\nposition fen {}\n go wtime 0 winc 600000 btime 0 binc 600000\n",
            test.game_state.to_fen()
        );
        write_to_buf(&mut child_in, &str);
        let before = Instant::now();
        let bm;
        loop {
            line.clear();
            child_out.read_line(&mut line).unwrap();
            let cmd: Vec<&str> = line.split(' ').collect();
            if cmd[0] == "bestmove" {
                bm = cmd[1].trim();
                break;
            } else {
                //Find mv
                let mut index = 0;
                while index < cmd.len() {
                    if cmd[index] == "depth" {
                        break;
                    }
                    index += 1;
                }
                if index >= cmd.len() {
                    continue;
                }
                let depth = cmd[index + 1].parse::<usize>().unwrap();
                if depth <= 7 {
                    continue;
                }
                index = 0;
                while index < cmd.len() {
                    if cmd[index] == "pv" {
                        break;
                    }
                    index += 1;
                }
                if index < cmd.len() && *cmd[index + 1].trim() == format!("{:?}", test.optimal_move)
                {
                    write_to_buf(&mut child_in, "stop\n");
                }
            }
        }
        let now = Instant::now();
        let dur = now.duration_since(before).as_millis();
        println!(
            "{}",
            &format!(
                "\n\n-----------------------------\nThread {} has looked at FEN: {}",
                thread_name,
                test.game_state.to_fen()
            )
        );
        println!("Bestmove is actually: {:?}", test.optimal_move);
        println!("Best move found after {} seconds", dur as f64 / 1000.0);
        println!("Best move found was {}", bm);
        if *bm == format!("{:?}", test.optimal_move) {
            let award = award_points(dur);
            let newpoints = points.load(Ordering::SeqCst) + award;
            (*points).store(newpoints, Ordering::SeqCst);
            println!("Best move is equal! Awarded Points: {}", award);
            println!("New Points: {}", newpoints);
        } else {
            println!("Best move is wrong!")
        }
        write_to_buf(&mut child_in, "stop\nnewgame\n");
        thread::sleep(Duration::from_millis(50));
    }

    write_to_buf(&mut child_in, "quit\n");
}

pub fn lct2(p1: &str, processors: usize, path_to_lct2: &str) {
    //Step 1: Parse suit
    let mypoints = Arc::new(AtomicUsize::new(1900));
    let suit = load_lct2suit(path_to_lct2);
    //Step 2: Prepare Arcs
    let queue = Arc::new(ThreadSafeQueue::new(suit));
    let mut childs = Vec::with_capacity(processors);
    for i in 0..processors {
        let queue_clone = queue.clone();
        let points_clone = mypoints.clone();
        let name = String::from_str(p1).unwrap();
        childs.push(thread::spawn(move || {
            start_lct2_thread(queue_clone, name, points_clone, &format!("{}", i))
        }))
    }
    for child in childs {
        child.join().expect("Couldn't join thread");
    }
    println!("LCT2 Finished! Points: {}", mypoints.load(Ordering::SeqCst));
}

fn award_points(dur: u128) -> usize {
    if dur <= 9000 {
        return 30;
    } else if dur <= 29000 {
        return 25;
    } else if dur <= 89000 {
        return 20;
    } else if dur <= 209_000 {
        return 15;
    } else if dur <= 389_000 {
        return 10;
    } else if dur <= 601_000 {
        return 5;
    }
    0usize
}

fn load_lct2suit(path_to_lct2: &str) -> Vec<Lct2Test> {
    let mut movelist = movegen::MoveList::default();
    let mut res = Vec::with_capacity(30);
    let mut file: File = File::open(path_to_lct2).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read the file");
    let split = contents.split('\n');
    for line in split {
        let linevec = line.split("bm").collect::<Vec<&str>>();
        if linevec.len() == 1 {
            break;
        }
        let state = GameState::from_fen(linevec[0].trim_end());
        let mv = linevec[1].trim().split(' ').collect::<Vec<&str>>()[0].replace(";", "");
        let (optimal_move, _) = parse_move(&state, &mv.to_string(), &mut movelist);
        res.push(Lct2Test {
            game_state: state,
            optimal_move,
        });
    }
    res
}

pub struct Lct2Test {
    game_state: GameState,
    optimal_move: GameMove,
}
