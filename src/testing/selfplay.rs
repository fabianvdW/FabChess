use crate::queue::ThreadSafeQueue;
use core::board_representation::game_state::{GameMove, GameMoveType, GameState, PieceType};
use core::misc::{GameParser, PGNParser};
use core::move_generation::movegen;
use core::search::alphabeta::GameResult;
use core::search::search::TimeControl;
use rand::Rng;
use std::fmt::{Display, Formatter, Result};
use std::fs::File;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio_process::CommandExt;

pub fn start_self_play(
    p1: &str,
    p2: &str,
    processors: usize,
    games: usize,
    opening_db: &str,
    opening_load_until: usize,
    tcp1: TimeControl,
    tcp2: TimeControl,
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
        let tcp1_clone = TimeControl {
            mytime: tcp1.mytime,
            myinc: tcp1.myinc,
        };
        let tcp2_clone = TimeControl {
            mytime: tcp2.mytime,
            myinc: tcp2.myinc,
        };
        childs.push(thread::spawn(move || {
            start_self_play_thread(
                queue_clone,
                res_clone,
                p1_clone,
                p2_clone,
                &tcp1_clone,
                &tcp2_clone,
            );
        }));
    }
    let mut results_collected = 0;
    let mut p1_wins = 0;
    let mut p2_wins = 0;
    let mut draws = 0;
    while results_collected < (games / 2) * 2 {
        thread::sleep(Duration::from_millis(50));
        if let Some(result) = result_queue.pop() {
            results_collected += 1;
            //Verarbeite Resultat
            println!("Game {} finished!", result.task_id);
            if let Some(reason) = result.endcondition {
                println!("Reason: {}", reason);
            } else {
                println!("Reason: Disqualification");
            }
            if result.draw {
                draws += 1;
            } else if result.p1_won {
                p1_wins += 1;
            } else if !result.p1_disq {
                p2_wins += 1;
            }
            println!("Player   Wins   Draws   Losses");
            println!("P1       {}     {}      {}", p1_wins, draws, p2_wins);
            println!("P2       {}     {}      {}", p2_wins, draws, p1_wins);
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
) -> (Option<String>, Option<tokio_process::ChildStdout>, usize) {
    let lines_codec = tokio::codec::LinesCodec::new();
    let line_fut = tokio::codec::FramedRead::new(output, lines_codec)
        .filter(move |lines| lines.starts_with(&starts_with[..]))
        .into_future()
        .timeout(Duration::from_millis(time_frame));
    let before = Instant::now();
    let result = runtime.block_on(line_fut);
    let after = Instant::now();
    let dur = after.duration_since(before).as_millis() as usize;
    match result {
        Ok(s) => match s.0 {
            Some(str) => (Some(str), Some(s.1.into_inner().into_inner()), dur),
            _ => (None, None, dur),
        },
        Err(_) => (None, None, dur),
    }
}

pub fn play_game(
    task: PlayTask,
    p1: String,
    p2: String,
    tcp1: &TimeControl,
    tcp2: &TimeControl,
) -> TaskResult {
    let player1_disq = TaskResult {
        p1_won: false,
        draw: false,
        p1_disq: true,
        p2_disq: false,
        endcondition: None,
        task_id: task.id,
    };
    let player2_disq = TaskResult {
        p1_won: false,
        draw: false,
        p1_disq: false,
        p2_disq: true,
        endcondition: None,
        task_id: task.id,
    };
    //-------------------------------------------------------------
    //Setup Tokio runtime
    let mut runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime!");
    //-------------------------------------------------------------
    //Setup Players
    //Player 1
    let mut player1_time = tcp1.mytime;
    let player1_inc = tcp1.myinc;

    let mut player1_process = Command::new(p1)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn_async()
        .expect("Failed to start player 1!");
    let player1_input = player1_process.stdin().take().unwrap();
    let player1_output = player1_process.stdout().take().unwrap();
    //let player1_stderr = player1_process.stderr().take().unwrap();
    let player1_input = print_command(&mut runtime, player1_input, "uci\n".to_owned());
    let output = expect_output("uciok".to_owned(), 10000, player1_output, &mut runtime);
    if let None = output.0 {
        println!("Player 1 didn't uciok in game {}!", task.id);
        return player1_disq;
    }
    let player1_output = output.1.unwrap();
    let mut player1_input = print_command(&mut runtime, player1_input, "isready\n".to_owned());
    let output = expect_output("readyok".to_owned(), 10000, player1_output, &mut runtime);
    if let None = output.0 {
        println!("Player 1 didn't readyok in game {}!", task.id);
        return player1_disq;
    }
    let mut player1_output = output.1.unwrap();
    //Player 2
    let mut player2_time = tcp2.mytime;
    let player2_inc = tcp2.myinc;

    let mut player2_process = Command::new(p2)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn_async()
        .expect("Failed to start player 2!");
    let player2_input = player2_process.stdin().take().unwrap();
    let player2_output = player2_process.stdout().take().unwrap();
    //let player2_stderr= player2_process.stderr().take().unwrap();
    let player2_input = print_command(&mut runtime, player2_input, "uci\n".to_owned());
    let output = expect_output("uciok".to_owned(), 10000, player2_output, &mut runtime);
    if let None = output.0 {
        println!("Player 2 didn't uciok in game {}!", task.id);
        return player2_disq;
    }
    let player2_output = output.1.unwrap();
    let mut player2_input = print_command(&mut runtime, player2_input, "isready\n".to_owned());
    let output = expect_output("readyok".to_owned(), 10000, player2_output, &mut runtime);
    if let None = output.0 {
        println!("Player 2 didn't readyok in game {}!", task.id);
        return player2_disq;
    }
    let mut player2_output = output.1.unwrap();
    //-------------------------------------------------------------
    //Setup Game
    let opening_fen = task.opening.to_fen();
    let (mut legal_moves, mut in_check) = movegen::generate_moves(&task.opening);
    let mut history: Vec<GameState> = Vec::with_capacity(100);
    let mut status =
        check_end_condition(&task.opening, legal_moves.len() > 0, in_check, &history).0;
    history.push(task.opening.clone());
    let mut move_history: Vec<GameMove> = Vec::with_capacity(100);

    let mut endcondition = None;
    //-------------------------------------------------------------
    while let GameResult::Ingame = status {
        //Request move
        let latest_state = &history[history.len() - 1];
        let player1_move = task.p1_is_white && latest_state.color_to_move == 0
            || !task.p1_is_white && latest_state.color_to_move == 1;
        //Prepare position string
        let mut position_string = String::new();
        position_string.push_str("position fen ");
        position_string.push_str(&opening_fen);
        position_string.push_str(" moves ");
        for mv in &move_history {
            position_string.push_str(&format!("{:?} ", mv));
        }
        position_string.push_str("\n");
        //Prepare go command
        let mut go_string = String::new();
        go_string.push_str(&format!(
            "go wtime {} winc {} btime {} binc {} \n",
            if task.p1_is_white {
                player1_time
            } else {
                player2_time
            },
            if task.p1_is_white {
                player1_inc
            } else {
                player2_inc
            },
            if task.p1_is_white {
                player2_time
            } else {
                player1_time
            },
            if task.p1_is_white {
                player2_inc
            } else {
                player1_inc
            }
        ));
        let game_move: &GameMove;
        if player1_move {
            player1_input = print_command(&mut runtime, player1_input, position_string);
            player1_input = print_command(&mut runtime, player1_input, "isready\n".to_owned());
            let output = expect_output("readyok".to_owned(), 100, player1_output, &mut runtime);
            if let None = output.0 {
                println!(
                    "Player 1 didn't readyok after position description in game {}!",
                    task.id
                );
                return player1_disq;
            }
            player1_output = output.1.unwrap();
            player1_input = print_command(&mut runtime, player1_input, go_string);
            let output = expect_output(
                "bestmove".to_owned(),
                player1_time,
                player1_output,
                &mut runtime,
            );
            if let None = output.0 {
                println!(
                    "Player 1 didn't send bestmove in time in game {}! He had {}ms left!",
                    task.id, player1_time
                );
                return player1_disq;
            }
            player1_output = output.1.unwrap();
            if output.2 as u64 > player1_time {
                println!("Mistake in Referee! Bestmove found but it took longer than time still left for player 1! Disqualifying player1 illegitimately in game {}",task.id);
                return player1_disq;
            }
            player1_time -= output.2 as u64;
            player1_time += player1_inc;

            //Parse the move
            let line = output.0.unwrap();
            let split_line: Vec<&str> = line.split_whitespace().collect();
            if split_line[0] == "bestmove" {
                let mv = GameMove::string_to_move(split_line[1]);
                let found_move = find_move(mv.0, mv.1, mv.2, &legal_moves);
                if let None = found_move {
                    println!("Player 1 sent illegal {} in game {}", line, task.id);
                    return player1_disq;
                }
                game_move = found_move.unwrap();
            } else {
                println!(
                    "Bestmove wasn't first argument after bestmove keyword! Disqualifiying player 1 in game {}",
                    task.id
                );
                return player1_disq;
            }
        } else {
            player2_input = print_command(&mut runtime, player2_input, position_string);
            player2_input = print_command(&mut runtime, player2_input, "isready\n".to_owned());
            let output = expect_output("readyok".to_owned(), 100, player2_output, &mut runtime);
            if let None = output.0 {
                println!(
                    "Player 2 didn't readyok after position description in game {}!",
                    task.id
                );
                return player2_disq;
            }
            player2_output = output.1.unwrap();
            player2_input = print_command(&mut runtime, player2_input, go_string);
            let output = expect_output(
                "bestmove".to_owned(),
                player2_time,
                player2_output,
                &mut runtime,
            );
            if let None = output.0 {
                println!(
                    "Player 2 didn't send bestmove in time in game {}! He had {}ms left!",
                    task.id, player2_time
                );
                return player2_disq;
            }
            player2_output = output.1.unwrap();
            if output.2 as u64 > player2_time {
                println!("Mistake in Referee! Bestmove found but it took longer than time still left for player 2! Disqualifying player1 illegitimately in game {}",task.id);
                return player2_disq;
            }
            player2_time -= output.2 as u64;
            player2_time += player2_inc;

            //Parse the move
            let line = output.0.unwrap();
            let split_line: Vec<&str> = line.split_whitespace().collect();
            if split_line[0] == "bestmove" {
                let mv = GameMove::string_to_move(split_line[1]);
                let found_move = find_move(mv.0, mv.1, mv.2, &legal_moves);
                if let None = found_move {
                    println!("Player 2 sent illegal {} in game {}", line, task.id);
                    return player2_disq;
                }
                game_move = found_move.unwrap();
            } else {
                println!(
                    "Bestmove wasn't first argument after bestmove keyword! Disqualifiying player 2 in game {}",
                    task.id
                );
                return player2_disq;
            }
        }
        //Make new state with move
        move_history.push(game_move.clone());
        let state = movegen::make_move(latest_state, game_move);
        let (lm, ic) = movegen::generate_moves(&state);
        legal_moves = lm;
        in_check = ic;
        let check = check_end_condition(&state, legal_moves.len() > 0, in_check, &history);
        status = check.0;
        endcondition = check.1;
        //Preparing next round
        history.push(state);
    }

    //-------------------------------------------------------------
    //Cleanup players' processes
    print_command(&mut runtime, player1_input, "quit\n".to_owned());
    print_command(&mut runtime, player2_input, "quit\n".to_owned());
    thread::sleep(Duration::from_millis(20));
    let draw = match status {
        GameResult::Draw => true,
        _ => false,
    };
    let p1_win = match status {
        GameResult::Draw => false,
        GameResult::WhiteWin => task.p1_is_white,
        GameResult::BlackWin => !task.p1_is_white,
        _ => panic!("invalid status"),
    };
    TaskResult {
        p1_won: p1_win,
        draw,
        p1_disq: false,
        p2_disq: false,
        endcondition,
        task_id: task.id,
    }
}

pub fn find_move(
    from: usize,
    to: usize,
    promo_pieces: Option<PieceType>,
    legal_moves: &Vec<GameMove>,
) -> Option<&GameMove> {
    for mv in legal_moves {
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
            return Some(mv);
        }
    }
    None
}
pub fn check_end_condition(
    game_state: &GameState,
    has_legal_moves: bool,
    in_check: bool,
    history: &Vec<GameState>,
) -> (GameResult, Option<EndConditionInformation>) {
    let enemy_win = if game_state.color_to_move == 0 {
        GameResult::BlackWin
    } else {
        GameResult::WhiteWin
    };
    if in_check && !has_legal_moves {
        return (enemy_win, Some(EndConditionInformation::Mate));
    }
    if !in_check && !has_legal_moves {
        return (GameResult::Draw, Some(EndConditionInformation::StaleMate));
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
        return (
            GameResult::Draw,
            Some(EndConditionInformation::DrawByMissingPieces),
        );
    }
    if game_state.half_moves >= 100 {
        return (
            GameResult::Draw,
            Some(EndConditionInformation::HundredMoveDraw),
        );
    }
    if get_occurences(history, game_state) >= 2 {
        return (
            GameResult::Draw,
            Some(EndConditionInformation::ThreeFoldRepetition),
        );
    }

    (GameResult::Ingame, None)
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
    tcp1: &TimeControl,
    tcp2: &TimeControl,
) {
    while let Some(task) = queue.pop() {
        println!("Starting game {}", task.id);
        result_queue.push(play_game(task, p1.clone(), p2.clone(), tcp1, tcp2));
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
    pub endcondition: Option<EndConditionInformation>,
    pub task_id: usize,
}
pub enum EndConditionInformation {
    HundredMoveDraw,
    ThreeFoldRepetition,
    DrawByAdjucation,
    DrawByMissingPieces,
    StaleMate,
    Mate,
    MateByAdjucation,
}
impl Display for EndConditionInformation {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(match *self {
            EndConditionInformation::HundredMoveDraw => "Hundred Move Draw",
            EndConditionInformation::ThreeFoldRepetition => "Draw by Three Fold Repetition",
            EndConditionInformation::DrawByAdjucation => "Draw by Adjucation",
            EndConditionInformation::DrawByMissingPieces => "Draw by missing pieces",
            EndConditionInformation::StaleMate => "Draw by Stalemate",
            EndConditionInformation::Mate => "Win by Mate",
            EndConditionInformation::MateByAdjucation => "Win by Adjucation",
        });
        write!(formatter, "{}", res_str)
    }
}
