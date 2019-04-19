use std::io::{self};
use std::u64;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, atomic::AtomicBool, atomic::Ordering};
use super::uci_engine::UCIEngine;
use crate::board_representation::game_state::{GameState, PieceType, GameMoveType, GameMove};
use crate::search::search::TimeControl;
use crate::move_generation::movegen;
use crate::search::search::Search;

pub fn parse_loop() {
    let mut history: Vec<GameState> = vec![];
    let mut us = UCIEngine::standard();
    let stop = Arc::new(AtomicBool::new(false));
    let stdin = io::stdin();
    let mut line = String::new();
    loop {
        line.clear();
        stdin.read_line(&mut line).ok().unwrap();
        let arg: Vec<&str> = line.split_whitespace().collect();
        let cmd = arg[0];
        match cmd.trim() {
            "" => continue,
            "uci" => uci(&us),
            "setoption" => setoption(),
            "newgame" => newgame(&mut us),
            "ucinewgame" => newgame(&mut us),
            "isready" => isready(),
            "position" => {
                history = position(&mut us, &arg[1..]);
            }
            "go" => {
                stop.store(false, Ordering::Relaxed);
                let tc = go(&us, &arg[1..]);
                let mut new_history = vec![];
                for gs in &history {
                    new_history.push(gs.clone());
                }
                let new_state = us.internal_state.clone();
                let cl = Arc::clone(&stop);
                thread::spawn(move || {
                    start_search(cl, new_state, new_history, tc);
                });
            }
            "stop" => {
                stop.store(true, Ordering::Relaxed);
                thread::sleep(Duration::from_millis(50));
            }
            "quit" => {
                break;
            }
            "d" => {
                print_internal_state(&us);
            }
            _ => {
                println!("Unknown command {}", line);
            }
        }
    }
}

pub fn start_search(stop: Arc<AtomicBool>, game_state: GameState, history: Vec<GameState>, tc: TimeControl) {
    let mut s = Search::new(tc);
    let res = s.search(100, game_state, history, stop);
    println!("bestmove {:?}", res.pv[0]);
}

pub fn print_internal_state(engine: &UCIEngine) {
    println!("{}", engine.internal_state);
    println!("FEN: {}", engine.internal_state.to_fen());
}

pub fn go(engine: &UCIEngine, cmd: &[&str]) -> TimeControl {
    let mut wtime: u64 = 0;
    let mut btime: u64 = 0;
    let mut winc: u64 = 0;
    let mut binc: u64 = 0;
    if cmd[0].to_lowercase() == "infinite" {
        wtime = 0u64;
        btime = 0u64;
        winc = u64::MAX;
        binc = u64::MAX;
        if engine.internal_state.color_to_move == 0 {
            return TimeControl {
                mytime: wtime,
                myinc: winc,
            };
        } else {
            return TimeControl {
                mytime: btime,
                myinc: binc,
            };
        }
    }
    let mut index = 0;
    while index < cmd.len() {
        match cmd[index] {
            "wtime" => {
                wtime = cmd[index + 1].parse::<u64>().unwrap();
            }
            "btime" => {
                btime = cmd[index + 1].parse::<u64>().unwrap();
            }
            "winc" => {
                winc = cmd[index + 1].parse::<u64>().unwrap();
            }
            "binc" => {
                binc = cmd[index + 1].parse::<u64>().unwrap();
            }
            _ => {
                panic!("Invalid go command")
            }
        };
        index += 2;
    }
    if engine.internal_state.color_to_move == 0 {
        return TimeControl {
            mytime: wtime,
            myinc: winc,
        };
    } else {
        return TimeControl {
            mytime: btime,
            myinc: binc,
        };
    }
}

pub fn position(engine: &mut UCIEngine, cmd: &[&str]) -> Vec<GameState> {
    let mut move_index = 1;
    match cmd[0] {
        "fen" => {
            let mut fen_string = String::new();
            while move_index < cmd.len() && cmd[move_index].to_lowercase() != "moves" {
                fen_string.push_str(cmd[move_index]);
                fen_string.push_str(" ");
                move_index += 1;
            }
            engine.internal_state = GameState::from_fen(fen_string.trim_end());
        }
        "startpos" => {
            engine.internal_state = GameState::standard();
        }
        _ => {
            panic!("Illegal position cmd");
        }
    }
    let mut history: Vec<GameState> = vec![];
    history.push(engine.internal_state.clone());
    if move_index < cmd.len() {
        if cmd[move_index].to_lowercase() == "moves" {
            move_index += 1;
            while move_index < cmd.len() {
                //Parse the move and make it
                let mv = cmd[move_index];
                let (from, to, promo) = GameMove::string_to_move(mv);
                engine.internal_state = scout_and_make_draftmove(from, to, promo, &engine.internal_state);
                history.push(engine.internal_state.clone());
                move_index += 1;
            }
        }
    }
    history.pop();
    return history;
}

pub fn scout_and_make_draftmove(from: usize, to: usize, promo_pieces: Option<PieceType>, game_state: &GameState) -> GameState {
    let (moves, _) = movegen::generate_moves(&game_state);
    for mv in moves {
        if mv.from == from && mv.to == to {
            if let GameMoveType::Promotion(ps, _) = mv.move_type {
                match promo_pieces {
                    Some(piece) => {
                        if piece != ps {
                            continue;
                        }
                    }
                    None => {
                        continue;
                    }
                }
            }
            return movegen::make_move(&game_state, &mv);
        }
    }
    panic!("Invalid move; not found in list!");
}

pub fn isready() {
    println!("readyok");
}

pub fn uci(engine: &UCIEngine) {
    engine.id_command();
    println!("uciok");
}

pub fn setoption() {}

pub fn newgame(engine: &mut UCIEngine) {
    engine.internal_state = GameState::standard();
}