extern crate core;

use core::board_representation::game_state::{GameMove, GameMoveType, GameState, PieceType};
use core::misc::parse_move;
use queue::ThreadSafeQueue;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
pub mod lct2;
pub mod queue;

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
        lct2::lct2(player1path, processors, path_to_lct2);
    } else {
        let mut myvec: Vec<GameState> = Vec::with_capacity(30);
        myvec.push(GameState::standard());
        let d: ThreadSafeQueue<GameState> = ThreadSafeQueue::new(myvec);
        d.push(GameState::from_fen(
            "8/1p3pp1/7p/5P1P/2k3P1/8/2K2P2/8 w - -",
        ));
        let res = d.pop().unwrap();
        println!("{}", res);
    }
}
pub fn write_to_buf(writer: &mut BufWriter<&mut std::process::ChildStdin>, message: &str) {
    writer
        .write(message.as_bytes())
        .expect("Unable to write to Buf!");
    writer.flush().expect("Unable to flush writer!");
}
