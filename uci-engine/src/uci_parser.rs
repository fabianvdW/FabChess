use super::uci_engine::UCIEngine;
use core_sdk::board_representation::game_state::{GameMove, GameMoveType, GameState, PieceType};
use core_sdk::move_generation::makemove::make_move;
use core_sdk::move_generation::movegen;
use core_sdk::search::cache::{Cache, MAX_HASH_SIZE, MIN_HASH_SIZE};
use core_sdk::search::searcher::{
    search_move, InterThreadCommunicationSystem, MAX_SKIP_RATIO, MAX_THREADS, MIN_SKIP_RATIO,
    MIN_THREADS,
};
use core_sdk::search::timecontrol::{TimeControl, MAX_MOVE_OVERHEAD, MIN_MOVE_OVERHEAD};
use core_sdk::search::MAX_SEARCH_DEPTH;
use std::io;
use std::sync::{atomic::Ordering, Arc};
use std::thread;
use std::time::Duration;
use std::u64;

pub fn parse_loop() {
    let mut history: Vec<GameState> = vec![];

    let mut us = UCIEngine::standard();

    let itcs = Arc::new(InterThreadCommunicationSystem::default());
    *itcs.cache() =
        Cache::with_size_threaded(itcs.uci_options().hash_size, itcs.uci_options().threads);
    let mut movelist = movegen::MoveList::default();

    let stdin = io::stdin();
    let mut line = String::new();
    loop {
        line.clear();
        stdin.read_line(&mut line).unwrap();
        if line.is_empty() {
            break;
        }
        let arg: Vec<&str> = line.split_whitespace().collect();
        if arg.is_empty() {
            continue;
        }
        let cmd = arg[0];
        match cmd.trim() {
            "" => {
                continue;
            }
            "uci" => {
                uci(&us, &itcs);
            }
            "setoption" => setoption(&arg[1..], &itcs),

            "ucinewgame" | "newgame" => {
                newgame(&mut us);
                itcs.cache().clear_threaded(itcs.uci_options().threads);
                itcs.saved_time.store(0, Ordering::Relaxed);
            }
            "isready" => isready(&itcs, true),
            "position" => {
                history = position(&mut us, &arg[1..], &mut movelist);
            }
            "go" => {
                isready(&itcs, false);
                let (tc, depth) = go(&us, &arg[1..]);
                let mut new_history = vec![];
                for gs in &history {
                    new_history.push(gs.clone());
                }
                let new_state = us.internal_state.clone();
                let itcs = Arc::clone(&itcs);
                thread::Builder::new()
                    .stack_size(2 * 1024 * 1024)
                    .spawn(move || {
                        search_move(itcs, depth as i16, new_state, new_history, tc);
                    })
                    .expect("Couldn't start thread");
            }
            "stop" => {
                *itcs.timeout_flag.write().unwrap() = true;
                thread::sleep(Duration::from_millis(5));
            }
            "quit" => {
                break;
            }
            "d" => {
                print_internal_state(&us);
            }
            "perft" => perft(&us.internal_state, &arg[1..]),
            "static" => {
                println!(
                    "cp {}",
                    core_sdk::evaluation::eval_game_state(&us.internal_state).final_eval
                );
            }
            _ => {
                println!("Unknown command {}", line);
            }
        }
    }
}

pub fn perft(game_state: &GameState, cmd: &[&str]) {
    let depth = cmd[0].parse::<usize>().unwrap();
    core_sdk::perft_div(&game_state, depth);
}

pub fn print_internal_state(engine: &UCIEngine) {
    println!("{}", engine.internal_state);
}

pub fn go(engine: &UCIEngine, cmd: &[&str]) -> (TimeControl, usize) {
    let mut wtime: u64 = 0;
    let mut btime: u64 = 0;
    let mut winc: u64 = 0;
    let mut binc: u64 = 0;
    let mut depth = MAX_SEARCH_DEPTH;
    if cmd[0].to_lowercase() == "infinite" {
        return (TimeControl::Infinite, depth);
    } else if cmd[0].to_lowercase() == "depth" {
        depth = cmd[1].parse::<usize>().unwrap();
        return (TimeControl::Infinite, depth);
    }
    let mut index = 0;
    let mut movestogo: Option<usize> = None;
    while index < cmd.len() {
        match cmd[index] {
            "wtime" => {
                wtime = cmd[index + 1].parse::<u64>().unwrap_or(0);
            }
            "btime" => {
                btime = cmd[index + 1].parse::<u64>().unwrap_or(0);
            }
            "winc" => {
                winc = cmd[index + 1].parse::<u64>().unwrap_or(0);
            }
            "binc" => {
                binc = cmd[index + 1].parse::<u64>().unwrap_or(0);
            }
            "movetime" => {
                let mvtime = cmd[index + 1].parse::<u64>().unwrap_or(0);
                return (TimeControl::MoveTime(mvtime), depth);
            }
            "movestogo" => movestogo = Some(cmd[index + 1].parse::<usize>().unwrap_or(1)),
            _ => println!("Some parts of the go command weren't recognized well."),
        };
        index += 2;
    }
    if movestogo.is_none() {
        if engine.internal_state.get_color_to_move() == 0 {
            (TimeControl::Incremental(wtime, winc), depth)
        } else {
            (TimeControl::Incremental(btime, binc), depth)
        }
    } else if let Some(mvs) = movestogo {
        if mvs == 0 {
            panic!("movestogo = 0");
        }
        if engine.internal_state.get_color_to_move() == 0 {
            (TimeControl::Tournament(wtime, winc, mvs), depth)
        } else {
            (TimeControl::Tournament(btime, binc, mvs), depth)
        }
    } else {
        panic!("Something went wrong in go!");
    }
}

pub fn position(
    engine: &mut UCIEngine,
    cmd: &[&str],
    movelist: &mut movegen::MoveList,
) -> Vec<GameState> {
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
    if move_index < cmd.len() && cmd[move_index].to_lowercase() == "moves" {
        move_index += 1;
        while move_index < cmd.len() {
            //Parse the move and make it
            let mv = cmd[move_index];
            let (from, to, promo) = GameMove::string_to_move(mv);
            engine.internal_state =
                scout_and_make_draftmove(from, to, promo, &engine.internal_state, movelist);
            history.push(engine.internal_state.clone());
            move_index += 1;
        }
    }
    history.pop();
    history
}

pub fn scout_and_make_draftmove(
    from: usize,
    to: usize,
    promo_pieces: Option<PieceType>,
    game_state: &GameState,
    movelist: &mut movegen::MoveList,
) -> GameState {
    movegen::generate_moves(&game_state, false, movelist);
    for gmv in movelist.move_list.iter() {
        let mv = gmv.0;
        if mv.from as usize == from && mv.to as usize == to {
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
            return make_move(&game_state, mv);
        }
    }
    panic!("Invalid move; not found in list!");
}

pub fn isready(itcs: &Arc<InterThreadCommunicationSystem>, print_rdy: bool) {
    if itcs.tx.read().unwrap().len() == 0 {
        let threads = itcs.uci_options().threads;
        InterThreadCommunicationSystem::update_thread_count(itcs, threads);
    }
    if print_rdy {
        println!("readyok");
    }
}

pub fn uci(engine: &UCIEngine, itcs: &InterThreadCommunicationSystem) {
    engine.id_command();
    println!(
        "option name Hash type spin default {} min {} max {}",
        itcs.uci_options().hash_size,
        MIN_HASH_SIZE,
        MAX_HASH_SIZE
    );
    println!("option name ClearHash type button");
    println!(
        "option name Threads type spin default {} min {} max {}",
        itcs.uci_options().threads,
        MIN_THREADS,
        MAX_THREADS
    );
    println!(
        "option name MoveOverhead type spin default {} min {} max {}",
        itcs.uci_options().move_overhead,
        MIN_MOVE_OVERHEAD,
        MAX_MOVE_OVERHEAD
    );
    println!(
        "option name DebugSMPPrint type check default {}",
        itcs.uci_options().debug_print
    );
    println!(
        "option name SMPSkipRatio type spin default {} min {} max {}",
        itcs.uci_options().skip_ratio,
        MIN_SKIP_RATIO,
        MAX_SKIP_RATIO
    );
    println!("uciok");
}

pub fn setoption(cmd: &[&str], itcs: &Arc<InterThreadCommunicationSystem>) {
    let mut index = 0;
    while index < cmd.len() {
        let arg = cmd[index];
        match arg.to_lowercase().as_str() {
            "hash" => {
                let num = cmd[index + 2]
                    .parse::<usize>()
                    .expect("Invalid Hash value!");
                itcs.uci_options().hash_size = num;
                let num_threads = itcs.uci_options().threads;
                *itcs.cache() = Cache::with_size_threaded(num, num_threads);
                println!("info String Succesfully set Hash to {}", num);
                return;
            }
            "clearhash" => {
                itcs.cache().clear_threaded(itcs.uci_options().threads);
                println!("info String Succesfully cleared hash!");
                return;
            }
            "threads" => {
                let num = cmd[index + 2]
                    .parse::<usize>()
                    .expect("Invalid Threads value!");
                InterThreadCommunicationSystem::update_thread_count(&itcs, num);
                println!("info String Succesfully set Threads to {}", num);
                return;
            }
            "moveoverhead" => {
                let num = cmd[index + 2]
                    .parse::<u64>()
                    .expect("Invalid MoveOverhead value!");
                itcs.uci_options().move_overhead = num;
                println!("info String Succesfully set MoveOverhad to {}", num);
                return;
            }
            "debugsmpprint" => {
                let val = cmd[index + 2]
                    .parse::<bool>()
                    .expect("Invalid DebugSMPPrint value!");
                itcs.uci_options().debug_print = val;
                println!("info String Succesfully set DebugSMPPrint to {}", val);
                return;
            }
            "smpskipratio" => {
                let num = cmd[index + 2]
                    .parse::<usize>()
                    .expect("Invalid SMPSkipRatio value!");
                itcs.uci_options().skip_ratio = num;
                println!("info String Succesfully set SMPSkipRatio to {}", num);
                return;
            }
            _ => {
                index += 1;
            }
        }
    }
}

pub fn newgame(engine: &mut UCIEngine) {
    engine.internal_state = GameState::standard();
}
