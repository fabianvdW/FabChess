use crate::queue::ThreadSafeQueue;
use core::board_representation::game_state::GameState;
use core::misc::{GameParser, PGNParser};
use core::move_generation::movegen;
use core::search::alphabeta::GameResult;
use rand::Rng;
use std::fs::File;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::prelude::*;
use tokio_process::CommandExt;

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
        let p1_clone = String::from_str(p1).unwrap();
        let p2_clone = String::from_str(p2).unwrap();
        childs.push(thread::spawn(move || {
            start_self_play_thread(queue_clone, res_clone, p1_clone, p2_clone);
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

pub fn print_command(
    runtime: &mut tokio::runtime::Runtime,
    input: tokio_process::ChildStdin,
    command: String,
) -> tokio_process::ChildStdin {
    let buf = command.as_bytes().to_owned();
    let fut = tokio_io::io::write_all(input, buf);
    runtime.block_on(fut).expect("Could not write!").0
}

pub fn expect_output(
    starts_with: String,
    time_frame: u64,
    output: tokio_process::ChildStdout,
    runtime: &mut tokio::runtime::Runtime,
) -> (Option<String>, Option<tokio_process::ChildStdout>) {
    let lines_codec = tokio::codec::LinesCodec::new();
    let line_fut = tokio::codec::FramedRead::new(output, lines_codec)
        .filter(move |lines| lines.starts_with(&starts_with[..]))
        .into_future()
        .timeout(Duration::from_millis(time_frame));
    let result = runtime.block_on(line_fut);
    match result {
        Ok(s) => match s.0 {
            Some(str) => (Some(str), Some(s.1.into_inner().into_inner())),
            _ => (None, None),
        },
        Err(_) => (None, None),
    }
}

pub fn play_game(task: PlayTask, p1: String, p2: String) -> TaskResult {
    let player1_disq = TaskResult {
        p1_won: false,
        draw: false,
        p1_disq: true,
        p2_disq: false,
    };
    //-------------------------------------------------------------
    //Setup Tokio runtime
    let mut runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime!");
    //-------------------------------------------------------------
    //Setup Players
    let mut player1_process = Command::new(p1)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn_async()
        .expect("Failed to start player 1!");
    let player1_input = player1_process.stdin().take().unwrap();
    let player1_output = player1_process.stdout().take().unwrap();

    let mut player1_stderr = player1_process.stderr().take().unwrap();
    let player1_input = print_command(&mut runtime, player1_input, "uci\n".to_owned());
    let output = expect_output("uciok".to_owned(), 10000, player1_output, &mut runtime);
    if let None = output.0 {
        println!("Player 1 didn't uciok in game {}!", task.id);
        return player1_disq;
    }
    let player1_output = output.1.unwrap();
    let player1_input = print_command(&mut runtime, player1_input, "isready\n".to_owned());
    let output = expect_output("readyok".to_owned(), 10000, player1_output, &mut runtime);
    if let None = output.0 {
        println!("Player 1 didn't readok in game {}!", task.id);
        return player1_disq;
    }
    let player1_output = output.1.unwrap();

    //-------------------------------------------------------------
    //Setup Game
    let mut state = task.opening;
    let mut status = GameResult::Ingame;
    let mut history: Vec<GameState> = Vec::with_capacity(100);
    let (mut legal_moves, mut in_check) = movegen::generate_moves(&state);
    //-------------------------------------------------------------
    while let GameResult::Ingame = status {
        history.push(state);

        //Make new state
        state = GameState::standard();
        let (lm, ic) = movegen::generate_moves(&state);
        legal_moves = lm;
        in_check = ic;
        status = check_end_condition(&state, legal_moves.len() > 0, in_check, &history);
        status = GameResult::Draw;
    }
    //write_to_buf(&mut player1_in, "quit\n");
    TaskResult {
        p1_won: false,
        draw: false,
        p1_disq: false,
        p2_disq: false,
    }
}
pub fn check_end_condition(
    game_state: &GameState,
    has_legal_moves: bool,
    in_check: bool,
    history: &Vec<GameState>,
) -> GameResult {
    let enemy_win = if game_state.color_to_move == 0 {
        GameResult::BlackWin
    } else {
        GameResult::WhiteWin
    };
    if in_check && !has_legal_moves {
        return enemy_win;
    }
    if !in_check && !has_legal_moves {
        return GameResult::Draw;
    }

    //Missing pieces
    if game_state.pieces[0][0]
        | game_state.pieces[1][0]
        | game_state.pieces[2][0]
        | game_state.pieces[3][0]
        | game_state.pieces[4][0]
        | game_state.pieces[0][1]
        | game_state.pieces[1][1]
        | game_state.pieces[2][1]
        | game_state.pieces[3][1]
        | game_state.pieces[4][1]
        == 0u64
    {
        return GameResult::Draw;
    }
    if game_state.half_moves >= 100 {
        return GameResult::Draw;
    }
    if get_occurences(history, game_state) >= 2 {
        return GameResult::Draw;
    }

    GameResult::Ingame
}

pub fn get_occurences(history: &Vec<GameState>, state: &GameState) -> usize {
    let mut occ = 0;
    for other in history {
        if other.hash == state.hash {
            occ += 1;
        }
    }
    occ
}

pub fn start_self_play_thread(
    queue: Arc<ThreadSafeQueue<PlayTask>>,
    result_queue: Arc<ThreadSafeQueue<TaskResult>>,
    p1: String,
    p2: String,
) {
    while let Some(task) = queue.pop() {
        println!("Starting game {}", task.id);
        result_queue.push(play_game(task, p1.clone(), p2.clone()));
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
}
