use crate::write_to_buf;
use core::board_representation::game_state::{GameMove, GameState};
use core::misc::parse_move;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process::{Command, Stdio};
use std::time::Instant;

pub fn lct2(p1: &str, processors: usize, path_to_lct2: &str) {
    //Step 1: Parse suit
    let mut mypoints = 1900;
    let suit = load_lct2suit(path_to_lct2);
    //Step 2: Start uci_engine
    let mut child = Command::new(&format!("{}", p1))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute command!");
    let mut child_in = BufWriter::new(child.stdin.as_mut().unwrap());
    let mut child_out = BufReader::new(child.stdout.as_mut().unwrap());
    let mut line = String::new();
    for game in suit {
        println!(
            "\n\n-----------------------------\nLooking at FEN: {}",
            game.game_state.to_fen()
        );
        println!("Bestmove is actually: {:?}", game.optimal_move);
        let str = format!(
            "position fen {}\n go wtime 0 winc 600000 btime 0 binc 600000\n",
            game.game_state.to_fen()
        );
        write_to_buf(&mut child_in, &str);
        let before = Instant::now();
        let bm;
        loop {
            line.clear();
            child_out.read_line(&mut line).unwrap();
            let cmd: Vec<&str> = line.split(" ").collect();
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
                if index < cmd.len() {
                    if cmd[index + 1].trim() == &format!("{:?}", game.optimal_move) {
                        write_to_buf(&mut child_in, "stop\n");
                    }
                }
            }
        }
        let now = Instant::now();
        let dur = now.duration_since(before).as_millis();
        println!("Best move found after {} seconds", dur as f64 / 1000.0);
        println!("Best move found was {}", bm);
        if bm == &format!("{:?}", game.optimal_move) {
            let award = award_points(dur);
            mypoints += award;
            println!("Best move is equal! Awarded Points: {}", award);
        } else {
            println!("Best move is wrong!")
        }
        write_to_buf(&mut child_in, "stop\nnewgame\n");
    }
    write_to_buf(&mut child_in, "quit\n");
    println!("LCT2 Finished! Points: {}", mypoints);
}

fn award_points(dur: u128) -> usize {
    if dur <= 9000 {
        return 30;
    } else if dur <= 29000 {
        return 25;
    } else if dur <= 89000 {
        return 20;
    } else if dur <= 209000 {
        return 15;
    } else if dur <= 389000 {
        return 10;
    } else if dur <= 601000 {
        return 5;
    }
    0usize
}

fn load_lct2suit(path_to_lct2: &str) -> Vec<Lct2Test> {
    let mut res = Vec::with_capacity(30);
    let mut file: File = File::open(path_to_lct2).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read the file");
    let split = contents.split("\n");
    for line in split {
        let linevec = line.split("bm").collect::<Vec<&str>>();
        if linevec.len() == 1 {
            break;
        }
        let state = GameState::from_fen(linevec[0].trim_end());
        let mv = linevec[1].trim().split(" ").collect::<Vec<&str>>()[0].replace(";", "");
        let (optimal_move, _) = parse_move(&state, &mv.to_string());
        res.push(Lct2Test {
            game_state: state,
            optimal_move,
        });
    }
    res
}

struct Lct2Test {
    game_state: GameState,
    optimal_move: GameMove,
}
