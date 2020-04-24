use crate::async_communication::{
    expect_output, expect_output_and_listen_for_info, print_command, write_stderr_to_log,
};
use core_sdk::board_representation::game_state::*;
use core_sdk::move_generation::movegen::MoveList;
use core_sdk::search::timecontrol::TimeControl;
use log::error;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::process::{Command, Stdio};
use tokio_process::{Child, ChildStderr, ChildStdin, ChildStdout, CommandExt};

pub enum EngineReaction<T> {
    ContinueGame(T),
    DisqualifyEngine,
}
pub enum EngineStatus {
    ProclaimsWin,
    ProclaimsLoss,
    ProclaimsDraw,
    ProclaimsNothing,
}
#[derive(Clone, Copy)]
pub enum EndConditionInformation {
    HundredMoveDraw,
    ThreeFoldRepetition,
    DrawByadjudication,
    DrawByMissingPieces,
    StaleMate,
    Mate,
    MateByadjudication,
}

impl Display for EndConditionInformation {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(match *self {
            EndConditionInformation::HundredMoveDraw => "Hundred Move Draw",
            EndConditionInformation::ThreeFoldRepetition => "Draw by Three Fold Repetition",
            EndConditionInformation::DrawByadjudication => "Draw by adjudication",
            EndConditionInformation::DrawByMissingPieces => "Draw by missing pieces",
            EndConditionInformation::StaleMate => "Draw by Stalemate",
            EndConditionInformation::Mate => "Win by Mate",
            EndConditionInformation::MateByadjudication => "Win by adjudication",
        });
        write!(formatter, "{}", res_str)
    }
}

pub fn get_elo_gain(p_a: f64) -> f64 {
    -1.0 * (1.0 / p_a - 1.0).ln() * 400.0 / (10.0 as f64).ln()
}

#[derive(Clone)]
pub struct EngineStats {
    pub moves_played: usize,
    pub avg_depth: f64,
    pub avg_nps: f64,
    pub avg_timeleft: f64,
}

impl EngineStats {
    pub fn divide(&mut self) {
        self.avg_depth /= self.moves_played as f64;
        self.avg_nps /= self.moves_played as f64;
    }
    pub fn add(&mut self, other: &EngineStats) {
        let sum = (self.moves_played + other.moves_played) as f64;
        self.avg_depth = self.avg_depth * self.moves_played as f64 / sum
            + other.avg_depth * other.moves_played as f64 / sum;
        self.avg_nps = self.avg_nps * self.moves_played as f64 / sum
            + other.avg_nps * other.moves_played as f64 / sum;
        self.moves_played += other.moves_played;
    }
}

impl Default for EngineStats {
    fn default() -> Self {
        EngineStats {
            moves_played: 0,
            avg_depth: 0.,
            avg_nps: 0.,
            avg_timeleft: 0.,
        }
    }
}

#[derive(Clone)]
pub struct Engine {
    pub name: String,
    pub path: String,
    pub id: usize,
    pub wins: usize,
    pub draws: usize,
    pub losses: usize,
    pub disqs: usize,
    pub time_control: TimeControl,
    pub stats: EngineStats,
    pub uci_options: HashMap<String, String>,
}

impl Engine {
    pub fn add(&mut self, other: &Engine) {
        let games = self.wins + self.draws + self.losses;
        let other_games = other.wins + other.draws + other.losses;
        self.stats.add(&other.stats);
        if games + other_games != 0 {
            self.stats.avg_timeleft = self.stats.avg_timeleft * games as f64
                / (games + other_games) as f64
                + other.stats.avg_timeleft * other_games as f64 / (games + other_games) as f64;
        }
        self.wins += other.wins;
        self.draws += other.draws;
        self.losses += other.losses;
        self.disqs += other.disqs;
    }
    pub fn get_elo_gain(&self) -> (String, String, f64) {
        //Derived from 1. E_A= 1/(1+10^(-DeltaElo/400)) and 2. |X/N-p|<=1.96*sqrt(N*p*(1-p))/n
        let n: f64 = (self.wins + self.draws + self.losses) as f64;
        let x_a: f64 = self.wins as f64 + self.draws as f64 / 2.0;
        let (elo_gain, elo_bounds) = if n >= 1. || x_a >= 0. {
            let p_a: f64 = x_a / n;
            let k: f64 = (1.96 * 1.96 + 2.0 * x_a) / (-1.0 * 1.96 * 1.96 - n);
            let q = -1.0 * x_a * x_a / (n * (-1.96 * 1.96 - n));
            let root = ((k / 2.0) * (k / 2.0) - q).sqrt();
            let p_a_upper: f64 = -k / 2.0 + root;
            let curr = get_elo_gain(p_a);
            (curr, get_elo_gain(p_a_upper) - curr)
        } else {
            (0., 0.)
        };
        (
            format!(
                "{:25}{:.2}   +/- {:.2}   +{}   ={}   -{}  sc {:.1}%",
                self.name,
                elo_gain,
                elo_bounds,
                self.wins,
                self.draws,
                self.losses,
                100. * (self.wins as f64 + self.draws as f64 / 2.)
                    / (self.wins + self.draws + self.losses) as f64,
            ),
            format!(
                "{:25}disq {} dep {:.2} nps {:.0} time {:.0}",
                self.name,
                self.disqs,
                self.stats.avg_depth,
                self.stats.avg_nps,
                self.stats.avg_timeleft
            ),
            elo_gain,
        )
    }

    pub fn from_path(
        path: &str,
        id: usize,
        tc: TimeControl,
        options: HashMap<String, String>,
    ) -> Self {
        let mut res = Engine {
            name: "".to_owned(),
            path: path.to_string(),
            id,
            wins: 0,
            draws: 0,
            losses: 0,
            disqs: 0,
            time_control: tc,
            stats: EngineStats::default(),
            uci_options: options,
        };
        let (_child, input, output, _err) = res.get_handles();
        let mut runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime!");
        let _input = print_command(&mut runtime, input, "uci\n".to_owned());
        let output = expect_output_and_listen_for_info(
            "uciok".to_owned(),
            10000,
            output,
            &mut runtime,
            "id name".to_owned(),
        );
        if output.3.contains("id name") {
            let name = output
                .3
                .rsplit("id name")
                .next()
                .expect(&format!("Couldn't catch the name of engine {}", res.path));
            res.name = name[..name.len() - 1].to_owned();
        } else {
            panic!("Couldn't catch the name of engine {}", res.path);
        }
        res
    }

    pub fn request_move(
        &mut self,
        position_description: String,
        go_string: String,
        mut stdin: ChildStdin,
        mut stdout: ChildStdout,
        mut stderr: ChildStderr,
        runtime: &mut tokio::runtime::Runtime,
        task_id: usize,
        movelist: &MoveList,
    ) -> EngineReaction<(GameMove, ChildStdin, ChildStdout, ChildStderr, EngineStatus)> {
        stdin = print_command(runtime, stdin, position_description);
        let reaction = self.valid_isready_reaction(stdin, stdout, stderr, runtime, task_id);
        match reaction {
            EngineReaction::DisqualifyEngine => return EngineReaction::DisqualifyEngine,
            EngineReaction::ContinueGame(temp) => {
                stdin = temp.0;
                stdout = temp.1;
                stderr = temp.2;
            }
        }
        stdin = print_command(runtime, stdin, go_string);
        let output = expect_output_and_listen_for_info(
            "bestmove".to_owned(),
            self.time_control.time_left(),
            stdout,
            runtime,
            "info".to_owned(),
        );
        if output.0.is_none() {
            error!(
                "Engine {} didn't send bestmove in time in game {}! It had {}ms left!\n",
                self.name,
                task_id,
                self.time_control.time_left(),
            );
            write_stderr_to_log(stderr, runtime);
            return EngineReaction::DisqualifyEngine;
        }
        stdout = output.1.unwrap();
        if output.2 as u64 > self.time_control.time_left() {
            error!("Mistake in Referee! Bestmove found but it took longer than time still left ({}) for engine {}! Disqualifying engine illegitimately in game {}\n",self.time_control.time_left(),self.name ,task_id);
            return EngineReaction::DisqualifyEngine;
        }
        self.time_control.update(output.2 as u64, None);

        //Parse the move
        let line = output.0.unwrap();
        let split_line: Vec<&str> = line.split_whitespace().collect();
        let game_move: GameMove = if split_line[0] == "bestmove" {
            let mv = GameMove::string_to_move(split_line[1]);
            let found_move = find_move(mv.0, mv.1, mv.2, &movelist);
            if found_move.is_none() {
                error!(
                    "Engine {} sent illegal move ({}) in game {}\n",
                    self.name, line, task_id
                );
                write_stderr_to_log(stderr, runtime);
                return EngineReaction::DisqualifyEngine;
            }
            found_move.unwrap()
        } else {
            error!(
                "Bestmove wasn't first argument after bestmove keyword! Disqualifiying engine {} in game {}\n",
                self.name,task_id
            );
            write_stderr_to_log(stderr, runtime);
            return EngineReaction::DisqualifyEngine;
        };

        //Get additional info about engine e.g. how deep it saw, nps, and its evaluation
        let mut status = EngineStatus::ProclaimsNothing;
        self.stats.moves_played += 1;
        let info = fetch_info(output.3.clone());
        if info.negative_mate_found {
            status = EngineStatus::ProclaimsLoss;
        } else if info.positive_mate_found {
            status = EngineStatus::ProclaimsWin;
        } else if info.cp_score.is_some() {
            let score = info.cp_score.unwrap();
            if score.abs() <= 10 {
                status = EngineStatus::ProclaimsDraw;
            }
            if score < -1000 {
                status = EngineStatus::ProclaimsLoss;
            } else if score > 1000 {
                status = EngineStatus::ProclaimsWin
            }
        }

        if let Some(dep) = info.depth {
            self.stats.avg_depth += dep as f64;
        }
        if let Some(nps) = info.nps {
            self.stats.avg_nps += nps as f64;
        }

        EngineReaction::ContinueGame((game_move, stdin, stdout, stderr, status))
    }

    pub fn valid_isready_reaction(
        &self,
        stdin: ChildStdin,
        stdout: ChildStdout,
        stderr: ChildStderr,
        runtime: &mut tokio::runtime::Runtime,
        task_id: usize,
    ) -> EngineReaction<(ChildStdin, ChildStdout, ChildStderr)> {
        let stdin = print_command(runtime, stdin, "isready\n".to_owned());
        let output = expect_output("readyok".to_owned(), 10000, stdout, runtime);
        if output.0.is_none() {
            error!("Engine {} didn't readyok in game {}!\n", self.name, task_id);
            write_stderr_to_log(stderr, runtime);
            return EngineReaction::DisqualifyEngine;
        }
        let stdout = output.1.unwrap();
        EngineReaction::ContinueGame((stdin, stdout, stderr))
    }
    pub fn valid_uci_isready_reaction(
        &self,
        stdin: ChildStdin,
        stdout: ChildStdout,
        stderr: ChildStderr,
        runtime: &mut tokio::runtime::Runtime,
        task_id: usize,
    ) -> EngineReaction<(ChildStdin, ChildStdout, ChildStderr)> {
        let mut stdin = print_command(runtime, stdin, "uci\n".to_owned());
        let output = expect_output("uciok".to_owned(), 10000, stdout, runtime);
        if output.0.is_none() {
            error!("Engine {} didn't uciok in game {}!\n", self.name, task_id);
            write_stderr_to_log(stderr, runtime);
            return EngineReaction::DisqualifyEngine;
        }
        let stdout = output.1.unwrap();
        for pair in &self.uci_options {
            stdin = print_command(
                runtime,
                stdin,
                format!("setoption name {} value {}\n", pair.0, pair.1),
            );
        }
        self.valid_isready_reaction(stdin, stdout, stderr, runtime, task_id)
    }

    pub fn get_handles(&self) -> (Child, ChildStdin, ChildStdout, ChildStderr) {
        let mut process = Command::new(self.path.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn_async()
            .unwrap_or_else(|_| panic!("Failed to start engine {}!", self.path));
        let input = process.stdin().take().unwrap();
        let output = process.stdout().take().unwrap();
        let stderr = process.stderr().take().unwrap();
        (process, input, output, stderr)
    }
}

pub fn find_move(
    from: usize,
    to: usize,
    promo_pieces: Option<PieceType>,
    move_list: &MoveList,
) -> Option<GameMove> {
    for gmv in move_list.move_list.iter() {
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
            return Some(mv);
        }
    }
    None
}

pub fn fetch_info(info: String) -> UCIInfo {
    let split_line: Vec<&str> = info.split_whitespace().collect();
    let mut depth = None;
    let mut nps = None;
    let mut cp_score = None;
    let mut positive_mate_found = false;
    let mut negative_mate_found = false;
    let mut index = 0;
    while index < split_line.len() {
        match split_line[index] {
            "depth" => {
                depth = split_line[index + 1].parse::<usize>().ok();
                index += 1;
            }
            "cp" => {
                cp_score = split_line[index + 1].parse::<isize>().ok();
                index += 1;
            }
            "nps" => {
                nps = split_line[index + 1].parse::<usize>().ok();
                index += 1;
            }
            "mate" => {
                let mate_score = match split_line[index + 1].parse::<isize>() {
                    Ok(s) => s,
                    _ => 0,
                };
                if mate_score < 0 {
                    negative_mate_found = true;
                } else if mate_score > 0 {
                    positive_mate_found = true;
                }
            }
            _ => {}
        }
        index += 1;
    }
    UCIInfo {
        depth,
        nps,
        cp_score,
        positive_mate_found,
        negative_mate_found,
    }
}

pub struct UCIInfo {
    depth: Option<usize>,
    nps: Option<usize>,
    cp_score: Option<isize>,
    positive_mate_found: bool,
    negative_mate_found: bool,
}

pub struct PlayTask {
    pub opening: GameState,
    pub opening_sequence: Vec<GameMove>,
    pub p1_is_white: bool,
    pub id: usize,
    pub engine1: Engine,
    pub engine2: Engine,
}

pub struct TaskResult {
    pub task: PlayTask,
    pub endcondition: Option<EndConditionInformation>,
    pub move_sequence: Vec<GameMove>,
    pub final_status: GameResult,
}

impl TaskResult {
    pub fn disq(
        mut task: PlayTask,
        p1: bool,
        move_sequence: Vec<GameMove>,
        final_status: GameResult,
    ) -> Self {
        if p1 {
            task.engine1.disqs += 1;
        } else {
            task.engine2.disqs += 1;
        }
        task.engine1.stats.divide();
        task.engine2.stats.divide();
        TaskResult {
            task,
            endcondition: None,
            move_sequence,
            final_status,
        }
    }
}
