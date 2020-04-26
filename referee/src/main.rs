use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::BufWriter;
use std::io::Write;

pub mod async_communication;
pub mod engine;
pub mod lct2;
pub mod logging;
pub mod openings;
pub mod queue;
pub mod selfplay;
pub mod selfplay_splitter;
pub mod suit;

//STS
pub const STS_SUB_SUITS: [&str; 15] = [
    "Undermine",
    "Open Files and Diagonals",
    "Knight Outposts",
    "Square Vacancy",
    "Bishop vs Knight",
    "Recapturing",
    "STS(v7.0) Simplification",
    "AKPC",
    "Advancement of a/b/c pawns",
    "STS(v10.0) Simplification",
    "King Activity",
    "Center Control",
    "Pawn Play in the Center",
    "7th Rank",
    "STS(v15.0) AT",
];

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub processors: usize,
    selfplaytests: bool,
    pub games: usize,
    pub engine_path: (String, HashMap<String, String>),
    pub enemies_paths: Vec<(String, HashMap<String, String>)>,
    pub opening_databases: Vec<String>,
    pub opening_load_untilply: usize,
    pub timecontrol_engine_time: u64,
    pub timecontrol_engine_inc: u64,
    pub timecontrol_enemies_time: u64,
    pub timecontrol_enemies_inc: u64,
    testsuitetests: bool,
    suite_path: String,
    suite_movetime: u64,
    lct2tests: bool,
    lct2_path: String,
}
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
    let mut config_path = "REFEREE_CONFIG.json";
    let args: Vec<String> = env::args().collect();
    let mut index: usize = 1;
    while index < args.len() {
        match &args[index][..] {
            "config" => {
                config_path = &args[index + 1];
                index += 2;
                continue;
            }
            _ => {
                println!(
                    "Invalid argument {}, use config CONFIG_FILE to specify",
                    &args[index]
                );
                index += 1;
            }
        }
    }
    let config_content = fs::read_to_string(config_path).expect("Unable to read config file!");
    let config: Config = serde_json::from_str(&config_content).unwrap();
    if config.selfplaytests {
        let mut runtime = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .core_threads(config.processors)
            .enable_all()
            .build()
            .expect("Could not create tokio runtime");
        //let mut runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime");
        runtime.block_on(selfplay_splitter::start_self_play(config));
    } else if config.lct2tests {
        lct2::lct2(&config.engine_path.0, config.processors, &config.lct2_path);
    } else if config.testsuitetests {
        suit::start_suit(
            &config.engine_path.0,
            config.processors,
            &config.suite_path,
            config.suite_movetime,
        );
    }
}

pub fn write_to_buf(writer: &mut BufWriter<&mut std::process::ChildStdin>, message: &str) {
    let _ = writer
        .write(message.as_bytes())
        .expect("Unable to write to Buf!");
    writer.flush().expect("Unable to flush writer!");
}
