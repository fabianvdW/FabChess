extern crate core;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_process;
use core::search::search::TimeControl;
use std::env;
use std::io::BufWriter;
use std::io::Write;

pub mod async_communication;
pub mod lct2;
pub mod openings;
pub mod queue;
pub mod selfplay;
pub mod selfplay_splitter;

const STD_PROCESSORS: usize = 4;
const STD_GAMES: usize = 1000;
const MODE: usize = 0;
const PLAYER1_STD_PATH: &str = "./target/release/schach_reworked.exe";
const PLAYER2_STD_PATH: &str = "./versions/FabChessv1.2.exe";
const LCT2_PATH: &str = "./lct2.epd";
const OPENING_DB: &str = "./O-Deville/o-deville.pgn";
const LOAD_UNTIL_PLY: usize = 8;

const TIMECONTROL_TIME: u64 = 10000;
const TIMECONTROL_INC: u64 = 100;
/*
Error-Margin in +/- (95% Confidence)
Games   :    100    200    400    600    1000    1500    2000    3000    4000     10000
Win/Elo Gain:---------------------------------------------------------------------------------------------
0.5 /  0.0   68.99  48.46  34.16  27.86  21.46   17.60   15.24   12.44   10.77    6.81
0.51/ 6.95   69.28  48.61  34.23  27.91  21.50   17.62   15.25   12.45   10.78    6.81
0.52/ 13.91  69.61  48.78  34.32  27.97  21.63   17.65   15.28   12.47   10.80    6.82
0.53/ 20.87  69.96  48.97  34.43  28.05  21.68   17.68   15.30   12.49   10.81    6.83
0.54/ 27.85  70.35  49.18  34.54  28.13  21.74   17.73   15.34   12.51   10.83    6.84
0.55/ 34.86  70.78  49.41  34.68  28.23  21.81   17.78   15.38   12.55   10.86    6.86
0.575/52.51  72.00  50.09  35.08  28.53  22.02   17.94   15.51   12.65   10.95    6.91
0.6 / 70.44  73.47  50.94  35.58  28.91  22.29   18.15   15.69   12.79   11.06    6.98
0.625/88.74  75.23  51.96  36.21  29.40  22.63   18.42   15.92   12.97   11.21    7.07
0.65/107.54  77.33  53.20  36.97  29.97  23.06   18.75   16.20   13.19   11.40    7.18
0.7 /147.19  82.78  56.41  38.97  31.52  24.19   19.64   16.96   13.79   11.91    7.50
*/
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
                index += 1;
                continue;
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
            "timecontroldouble" | "tcd" => {
                timecontrol_p1_inc *= 2;
                timecontrol_p2_inc *= 2;
                timecontrol_p1_time *= 2;
                timecontrol_p2_time *= 2;
                index += 1;
                continue;
            }
            "timecontrolincrease" | "tci" => {
                timecontrol_p1_inc += TIMECONTROL_INC;
                timecontrol_p2_inc += TIMECONTROL_INC;
                timecontrol_p1_time += TIMECONTROL_TIME;
                timecontrol_p2_time += TIMECONTROL_TIME;
                index += 1;
                continue;
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
        selfplay_splitter::start_self_play(
            player1path,
            player2path,
            processors,
            games,
            path_to_opening_db,
            opening_load_until,
            TimeControl::Incremental(timecontrol_p1_time, timecontrol_p1_inc),
            TimeControl::Incremental(timecontrol_p2_time, timecontrol_p2_inc),
        );
    }
}
pub fn write_to_buf(writer: &mut BufWriter<&mut std::process::ChildStdin>, message: &str) {
    writer
        .write(message.as_bytes())
        .expect("Unable to write to Buf!");
    writer.flush().expect("Unable to flush writer!");
}
