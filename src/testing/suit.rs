use crate::queue::ThreadSafeQueue;
use crate::write_to_buf;
use crate::STS_SUB_SUITS;
use core::board_representation::game_state::{GameMove, GameState};
use core::misc::parse_move;
use core::search::GradedMove;
use std::fmt::{Display, Formatter, Result};
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;

pub struct TestSuitResult {
    suit: SuitTest,
    mv: GameMove,
}
pub struct SuitInfos {
    move_time: u64,
    points: usize,
    positions: usize,
    optimal_moves_found: usize,
    subsuit_points: [usize; 15],
    subsuit_optimal_moves_found: [usize; 15],
    subsuit_positions: [usize; 15],
}

impl SuitInfos {
    fn new(move_time: u64) -> SuitInfos {
        SuitInfos {
            move_time,
            points: 0,
            optimal_moves_found: 0,
            subsuit_points: [0; 15],
            subsuit_optimal_moves_found: [0; 15],
            positions: 0,
            subsuit_positions: [0; 15],
        }
    }
}

impl Display for SuitInfos {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str = String::new();
        res_str.push_str(&format!("movetime {}\n", self.move_time));
        for i in 0..15 {
            if self.subsuit_positions[i] > 0 {
                res_str.push_str(&format!(
                    "Subsuit {} : {}/{}   Points: {}/{}\n",
                    STS_SUB_SUITS[i],
                    self.subsuit_optimal_moves_found[i],
                    self.subsuit_positions[i],
                    self.subsuit_points[i],
                    self.subsuit_positions[i] * 10
                ));
            }
        }
        res_str.push_str(&format!(
            "Found: {}/{}    Points: {}/{}\n",
            self.optimal_moves_found,
            self.positions,
            self.points,
            self.positions * 10
        ));
        write!(formatter, "{}", res_str)
    }
}

pub struct SuitTest {
    game_state: GameState,
    optimal_moves: Vec<GradedMove>,
    subsuit: isize,
}

impl SuitTest {
    pub fn award_points(&self, mv: &GameMove) -> usize {
        for other_mv in &self.optimal_moves {
            if other_mv.mv == *mv {
                return other_mv.score as usize;
            }
        }
        0
    }
}

pub fn start_suit(p1: &str, processors: usize, path_to_suit: &str, move_time: u64) {
    let queue = Arc::new(ThreadSafeQueue::new(load_suit(path_to_suit)));
    let resultqueue: Arc<ThreadSafeQueue<TestSuitResult>> =
        Arc::new(ThreadSafeQueue::new(Vec::new()));

    let mut infos = SuitInfos::new(move_time);

    let mut childs = Vec::with_capacity(processors);
    for _ in 0..processors {
        let queue_clone = queue.clone();
        let resultqueue_clone = resultqueue.clone();
        let engine = p1.to_owned();
        childs.push(thread::spawn(move || {
            suit_thread(engine, queue_clone, resultqueue_clone, move_time)
        }));
    }
    for child in childs {
        child.join().expect("Couldn't join thread");
    }
    while let Some(res) = resultqueue.pop() {
        infos.positions += 1;
        let points = res.suit.award_points(&res.mv);
        infos.points += points;
        if res.suit.subsuit != -1 {
            infos.subsuit_positions[res.suit.subsuit as usize] += 1;
            infos.subsuit_points[res.suit.subsuit as usize] += points;
        }
        if points == 10 {
            infos.optimal_moves_found += 1;
            if res.suit.subsuit != -1 {
                infos.subsuit_optimal_moves_found[res.suit.subsuit as usize] += 1;
            }
        }
    }
    //Format SuitInfos
    println!("{}", infos);
    println!("Suit Testing finished!");
}

fn suit_thread(
    p1: String,
    queue: Arc<ThreadSafeQueue<SuitTest>>,
    resultqueue: Arc<ThreadSafeQueue<TestSuitResult>>,
    move_time: u64,
) {
    let mut child = Command::new(&format!("{}", p1))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute command!");
    let mut child_in = BufWriter::new(child.stdin.as_mut().unwrap());
    let mut child_out = BufReader::new(child.stdout.as_mut().unwrap());
    let mut line = String::new();
    while let Some(test) = queue.pop() {
        let str = format!(
            "ucinewgame\nposition fen {}\n go movetime {}\n",
            test.game_state.to_fen(),
            move_time
        );
        write_to_buf(&mut child_in, &str);
        loop {
            line.clear();
            child_out.read_line(&mut line).unwrap();
            let cmd: Vec<&str> = line.split(" ").collect();
            if cmd[0] == "bestmove" {
                let bm = cmd[1].trim();
                let (mv, _) = parse_move(&test.game_state, &bm.to_owned());
                resultqueue.push(TestSuitResult { suit: test, mv });
                break;
            }
        }
    }
    write_to_buf(&mut child_in, "quit\n");
}
fn load_suit(path_to_suit: &str) -> Vec<SuitTest> {
    let mut res = Vec::with_capacity(30);
    let mut file: File = File::open(path_to_suit).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read the file");
    let split = contents.split("\n");
    for line in split {
        let linevec = line.split(";").collect::<Vec<&str>>();
        if linevec.len() == 1 {
            break;
        }
        let fenbmvec = linevec[0].split("bm").collect::<Vec<&str>>();
        let state = GameState::from_fen(fenbmvec[0].trim_end());
        let mut optimal_moves: Vec<GradedMove> = Vec::with_capacity(4);
        let mut subsuit_index = -1;
        for otherlines in linevec {
            if otherlines.trim().starts_with("c0") {
                let comment_line = otherlines.trim();
                let comment_line = comment_line.replace("c0", "");
                let comment_line = comment_line.replace("\"", "");
                let comment_line = comment_line.split(",").collect::<Vec<&str>>();
                for optimal_move_desc in comment_line {
                    let move_desc_split = optimal_move_desc.split("=").collect::<Vec<&str>>();
                    //println!("MoveDesc: {:?}", move_desc_split);
                    let (move_desc, _) = parse_move(&state, &move_desc_split[0].trim().to_owned());
                    let score = move_desc_split[1].parse::<u64>().unwrap();
                    optimal_moves.push(GradedMove {
                        mv: move_desc,
                        score: score as f64,
                    });
                }
            } else if otherlines.trim().starts_with("id") {
                subsuit_index = match_sub_suit(otherlines);
            }
        }
        res.push(SuitTest {
            game_state: state,
            optimal_moves,
            subsuit: subsuit_index,
        });
    }
    println!("Loaded {} positions from Suite!", res.len());
    res
}

fn match_sub_suit(desc: &str) -> isize {
    let mut res = -1;
    for (i, subsuits) in STS_SUB_SUITS.iter().enumerate() {
        if desc.contains(subsuits) {
            res = i as isize;
            break;
        }
    }
    res
}
