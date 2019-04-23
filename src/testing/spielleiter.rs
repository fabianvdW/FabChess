extern crate core;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_process;

use core::search::search::TimeControl;
use std::env;
use std::io::BufWriter;
use std::io::Write;
pub mod lct2;
pub mod queue;
pub mod selfplay;
const STD_PROCESSORS: usize = 4;
const STD_GAMES: usize = 1000;
const MODE: usize = 0;
const PLAYER1_STD_PATH: &str = "./target/release/schach_reworked.exe";
const PLAYER2_STD_PATH: &str = "./schach_reworkedalt.exe";
const LCT2_PATH: &str = "./lct2.epd";
const OPENING_DB: &str = "./O-Deville/o-deville.pgn";
const LOAD_UNTIL_PLY: usize = 6;

const TIMECONTROL_TIME: u64 = 10000;
const TIMECONTROL_INC: u64 = 100;
fn main() {
    let mut games = STD_GAMES;
    let mut processors = STD_PROCESSORS;
    let mut mode = MODE;
    let mut player1path = PLAYER1_STD_PATH;
    let mut player2path = PLAYER2_STD_PATH;
    let mut path_to_lct2 = LCT2_PATH;
    let mut path_to_opening_db = OPENING_DB;
    let mut opening_load_until = LOAD_UNTIL_PLY;
    let mut timecontrol_p1_time = TIMECONTROL_TIME;
    let mut timecontrol_p2_time = TIMECONTROL_TIME;
    let mut timecontrol_p1_inc = TIMECONTROL_INC;
    let mut timecontrol_p2_inc = TIMECONTROL_INC;

    let args: Vec<String> = env::args().collect();
    let mut index: usize = 0;
    while index < args.len() {
        match &args[index][..] {
            "lct2" => {
                mode = 1;
            }
            "processors" | "p" => {
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
            "opening" | "openingdb" | "o" => {
                path_to_opening_db = &args[index + 1];
            }
            "oload" | "openingload" | "loaduntil" => {
                opening_load_until = args[index + 1].parse::<usize>().unwrap();
            }
            "p1inc" | "tcp1inc" | "incp1" | "ip1" => {
                timecontrol_p1_inc = args[index + 1].parse::<u64>().unwrap();
            }
            "p2inc" | "tcp2inc" | "incp2" | "ip2" => {
                timecontrol_p2_inc = args[index + 1].parse::<u64>().unwrap();
            }
            "p1time" | "tcp1time" | "timep1" | "tp1" => {
                timecontrol_p1_time = args[index + 1].parse::<u64>().unwrap();
            }
            "p2time" | "tcp2time" | "timep2" | "tp2" => {
                timecontrol_p2_time = args[index + 1].parse::<u64>().unwrap();
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
        selfplay::start_self_play(
            player1path,
            player2path,
            processors,
            games,
            path_to_opening_db,
            opening_load_until,
            TimeControl {
                mytime: timecontrol_p1_time,
                myinc: timecontrol_p1_inc,
            },
            TimeControl {
                mytime: timecontrol_p2_time,
                myinc: timecontrol_p2_inc,
            },
        );
    }
}
pub fn write_to_buf(writer: &mut BufWriter<&mut std::process::ChildStdin>, message: &str) {
    writer
        .write(message.as_bytes())
        .expect("Unable to write to Buf!");
    writer.flush().expect("Unable to flush writer!");
}
