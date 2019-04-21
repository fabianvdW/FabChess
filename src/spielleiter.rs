extern crate core;

use core::board_representation::game_state::{GameMove, GameMoveType, GameState, PieceType};
use core::misc::parse_move;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const STD_PROCESSORS: usize = 4;
const STD_GAMES: usize = 1000;
const MODE: usize = 0;
const PLAYER1_STD_PATH: &str = "./target/release/schach_reworked.exe";
const PLAYER2_STD_PATH: &str = "./schach_reworkedalt.exe";
const LCT2_PATH: &str = "./lct2.epd";
fn main() {
    let mut games = STD_GAMES;
    let mut processors = STD_PROCESSORS;
    let mut mode = MODE;
    let mut player1path = PLAYER1_STD_PATH;
    let mut player2path = PLAYER2_STD_PATH;
    let mut path_to_lct2 = LCT2_PATH;
    let args: Vec<String> = env::args().collect();
    let mut index: usize = 0;
    while index < args.len() {
        match &args[index][..] {
            "lct2" => {
                mode = 1;
            }
            "processors" => {
                processors = args[index + 1].parse::<usize>().unwrap();
            }
            "games" => {
                games = args[index + 1].parse::<usize>().unwrap();
            }
            "player1" | "p1" => {
                player1path = &args[index + 1];
            }
            "player2" | "p2" => {
                player2path = &args[index + 1];
            }
            "lct2path" => {
                path_to_lct2 = &args[index + 1];
            }

            _ => {
                index += 1;
                continue;
            }
        }
        index += 2;
    }
    if mode == 1 {
        lct2(player1path, processors, path_to_lct2);
    }
}
fn lct2(p1: &str, processors: usize, path_to_lct2: &str) {
    //Step 1: Parse suit
    let mut mypoints = 1900;
    let suit = load_lct2suit(path_to_lct2);
    //Step 2: Start uci_engine
    let mut child = Command::new(&format!("{}", p1))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute command!");
    for game in suit {
        let child_in = child.stdin.as_mut().unwrap();
        let mut child_out = BufReader::new(child.stdout.as_mut().unwrap());
        let mut line = String::new();
        println!("Looking at FEN: {}", game.game_state.to_fen());
        println!("Bestmove is actually: {:?}", game.optimal_move);
        let str = format!(
            "position fen {}\n go wtime 0 winc 600000 btime 0 binc 600000\n",
            game.game_state.to_fen()
        );
        child_in.write(&str.as_bytes()).unwrap();
        let before = Instant::now();
        let (mut from, mut to, mut promotion_piece) = (0usize, 0usize, None);
        loop {
            child_out.read_line(&mut line).unwrap();
            let lines: Vec<&str> = line.split("\n").collect();
            let cmd: Vec<&str> = lines[lines.len() - 2].split_whitespace().collect();
            if cmd.len() == 0 {
                continue;
            }
            if cmd[0] == "bestmove" {
                let res = GameMove::string_to_move(cmd[1]);
                from = res.0;
                to = res.1;
                promotion_piece = res.2;
                println!("Breaking from here!");
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
                if depth <= 5 {
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
                    let res = GameMove::string_to_move(cmd[index + 1]);
                    from = res.0;
                    to = res.1;
                    promotion_piece = res.2;
                    if cmd[index + 1] == &format!("{:?}", game.optimal_move) {
                        break;
                    }
                }
            }
        }
        child_in.write(b"stop\nnewgame\nuci\n").unwrap();
        let now = Instant::now();
        let dur = now.duration_since(before).as_millis();
        println!("Best move found after {} seconds", dur as f64 / 1000.0);
        println!(
            "Best move found was {}",
            &format!("{}->{}, {:?}", from, to, promotion_piece)
        );
        if is_equal(from, to, promotion_piece, &game.optimal_move) {
            let award = award_points(dur);
            mypoints += award;
            println!("Best move is equal! Awarded Points: {}", award);
        } else {
            println!("Best move is wrong!")
        }
    }
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
    } else if dur <= 600000 {
        return 5;
    }
    0usize
}
fn is_equal(from: usize, to: usize, promotion_piece: Option<PieceType>, mv: &GameMove) -> bool {
    if mv.from == from && mv.to == to {
        if let GameMoveType::Promotion(ps, _) = mv.move_type {
            match promotion_piece {
                Some(piece) => {
                    if piece != ps {
                        return false;
                    }
                }
                None => {
                    return false;
                }
            }
        }
        return true;
    }
    false
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
