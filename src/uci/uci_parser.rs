use super::uci_engine::UCIEngine;
use crate::board_representation::game_state::{GameMove, GameMoveType, GameState, PieceType};
use crate::move_generation::makemove::make_move;
use crate::move_generation::movegen;
use crate::search::cache::Cache;
use crate::search::cache::CacheEntry;
use crate::search::search::Search;
use crate::search::timecontrol::{TimeControl, TimeControlInformation};
use std::io;
use std::sync::{atomic::AtomicBool, atomic::AtomicU64, atomic::Ordering, Arc, RwLock};
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::u64;

pub fn parse_loop() {
    let mut history: Vec<GameState> = vec![];
    let mut us = UCIEngine::standard();
    let stop = Arc::new(AtomicBool::new(false));
    let cache: Arc<RwLock<Cache>> = Arc::new(RwLock::new(Cache::new()));
    let mut movelist = movegen::MoveList::new();
    let saved_time = Arc::new(AtomicU64::new(0u64));
    let stdin = io::stdin();
    let mut line = String::new();
    loop {
        line.clear();
        stdin.read_line(&mut line).ok().unwrap();
        let arg: Vec<&str> = line.split_whitespace().collect();
        if arg.len() == 0 {
            continue;
        }
        let cmd = arg[0];
        match cmd.trim() {
            "" => continue,
            "uci" => {
                uci(&us);
            }
            "setoption" => setoption(),
            "ucinewgame" | "newgame" => {
                newgame(&mut us);
                (*cache).write().unwrap().clear();
                saved_time.store(0, Ordering::Relaxed);
            }
            "isready" => isready(),
            "position" => {
                history = position(&mut us, &arg[1..], &mut movelist);
            }
            "go" => {
                stop.store(false, Ordering::Relaxed);
                let (tc, depth) = go(&us, &arg[1..]);
                let mut new_history = vec![];
                for gs in &history {
                    new_history.push(gs.clone());
                }
                let new_state = us.internal_state.clone();
                let cl = Arc::clone(&stop);
                let cc = Arc::clone(&cache);
                let st = Arc::clone(&saved_time);
                thread::Builder::new()
                    .stack_size(2 * 1024 * 1024)
                    .spawn(move || {
                        start_search(cl, new_state, new_history, tc, cc, depth, st);
                    })
                    .expect("Couldn't start thread");
            }
            "stop" => {
                stop.store(true, Ordering::Relaxed);
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
                    crate::evaluation::eval_game_state(&us.internal_state).final_eval
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
    let now = Instant::now();
    let nodes = crate::perft_div(&game_state, depth);
    println!("{}", nodes);
    let after = Instant::now();
    let dur = after.duration_since(now);
    let secs = dur.as_millis() as f64 / 1000.0;
    println!(
        "{}",
        &format!("Time {} ({} nps)", secs, nodes as f64 / secs)
    );
}

pub fn start_search(
    stop: Arc<AtomicBool>,
    game_state: GameState,
    history: Vec<GameState>,
    tc: TimeControl,
    cache: Arc<RwLock<Cache>>,
    depth: usize,
    saved_time: Arc<AtomicU64>,
) {
    let mut s = Search::new(
        tc,
        TimeControlInformation::new(saved_time.load(Ordering::Relaxed)),
    );
    s.search(
        depth as i16,
        game_state.clone(),
        history,
        stop,
        cache,
        saved_time,
    );
    let bestmove = if let Some(mv) = s.pv_table[0].pv[0] {
        mv
    } else {
        CacheEntry::u16_to_mv(s.principal_variation[0].as_ref().unwrap().mv, &game_state)
    };
    println!("bestmove {:?}", bestmove);
}

pub fn print_internal_state(engine: &UCIEngine) {
    println!("{}", engine.internal_state);
    println!("FEN: {}", engine.internal_state.to_fen());
}

pub fn go(engine: &UCIEngine, cmd: &[&str]) -> (TimeControl, usize) {
    let mut wtime: u64 = 0;
    let mut btime: u64 = 0;
    let mut winc: u64 = 0;
    let mut binc: u64 = 0;
    let mut depth = 100;
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
            "movetime" => {
                let mvtime = cmd[index + 1].parse::<u64>().unwrap();
                return (TimeControl::MoveTime(mvtime), depth);
            }
            "movestogo" => movestogo = Some(cmd[index + 1].parse::<usize>().unwrap()),
            _ => panic!("Invalid go command"),
        };
        index += 2;
    }
    if let None = movestogo {
        if engine.internal_state.color_to_move == 0 {
            return (TimeControl::Incremental(wtime, winc), depth);
        } else {
            return (TimeControl::Incremental(btime, binc), depth);
        }
    } else if let Some(mvs) = movestogo {
        if mvs == 0 {
            panic!("movestogo = 0");
        }
        if engine.internal_state.color_to_move == 0 {
            return (TimeControl::Tournament(wtime, winc, mvs), depth);
        } else {
            return (TimeControl::Tournament(btime, binc, mvs), depth);
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
    if move_index < cmd.len() {
        if cmd[move_index].to_lowercase() == "moves" {
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
    }
    history.pop();
    return history;
}

pub fn scout_and_make_draftmove(
    from: usize,
    to: usize,
    promo_pieces: Option<PieceType>,
    game_state: &GameState,
    movelist: &mut movegen::MoveList,
) -> GameState {
    movegen::generate_moves(&game_state, false, movelist, 0);
    let mut index = 0;
    while index < movelist.counter[0] {
        let mv = movelist.move_list[0][index].as_ref().unwrap();
        if mv.from == from && mv.to == to {
            if let GameMoveType::Promotion(ps, _) = mv.move_type {
                match promo_pieces {
                    Some(piece) => {
                        if piece != ps {
                            index += 1;
                            continue;
                        }
                    }
                    None => {
                        index += 1;
                        continue;
                    }
                }
            }
            return make_move(&game_state, &mv);
        }
        index += 1;
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
