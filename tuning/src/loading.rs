use super::TexelState;
use core_sdk::{board_representation::game_state::GameState, evaluation::eval_game_state};
use std::fmt::{Display, Formatter, Result};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
pub enum FileFormatSupported {
    OwnEncoding,
    EPD,
    PGN,
}

pub struct LabelledGameState {
    pub game_state: GameState,
    pub label: f32,
}

pub struct Statistics {
    pub games: usize,
    pub white_wins: usize,
    pub black_wins: usize,
    pub draws: usize,
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics {
            games: 0,
            white_wins: 0,
            black_wins: 0,
            draws: 0,
        }
    }
}

impl Display for Statistics {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!("Games: {}\n", self.games));
        res_str.push_str(&format!("White-Wins: {}\n", self.white_wins));
        res_str.push_str(&format!("Black-Wins: {}\n", self.black_wins));
        res_str.push_str(&format!("Draws: {}\n", self.draws));
        write!(formatter, "{}", res_str)
    }
}

pub fn save_positions(to_file: &str, positions: &[LabelledGameState]) {
    let mut res_str = String::new();
    for pos in positions {
        res_str.push_str(&format!(
            "{} |{}\n",
            pos.game_state.to_fen(),
            if (pos.label - 1.0).abs() < std::f32::EPSILON {
                "White"
            } else if pos.label == 0.0 {
                "Black"
            } else {
                "Draw"
            }
        ));
    }
    fs::write(to_file, res_str).expect("Unable to write positions");
}

pub struct PositionLoader {
    reader: BufReader<File>,
    file_format: FileFormatSupported,
}
impl PositionLoader {
    pub fn new(from_file: &str, file_format: FileFormatSupported) -> Self {
        PositionLoader {
            reader: BufReader::new(File::open(from_file).expect("Could not open file")),
            file_format,
        }
    }
    pub fn next_position(&mut self) -> Option<LabelledGameState> {
        let mut line = String::new();
        self.reader.read_line(&mut line).unwrap();
        if let FileFormatSupported::OwnEncoding = self.file_format {
            if !line.contains('|') {
                return None;
            }
            let fen_split = line.split('|').collect::<Vec<&str>>();
            let game_result = if fen_split[1].contains("Black") {
                0.0
            } else if fen_split[1].contains("White") {
                1.0
            } else if fen_split[1].contains("Draw") {
                0.5
            } else {
                panic!(format!("Invalid split {}", fen_split[1]));
            };
            let state = GameState::from_fen(fen_split[0]);

            return Some(LabelledGameState {
                game_state: state,
                label: game_result,
            });
        } else if let FileFormatSupported::EPD = self.file_format {
            if line.is_empty() {
                return None;
            }
            let split = line.split(' ').collect::<Vec<&str>>();
            let fen = &format!("{} {} {} {}", split[0], split[1], split[2], split[3]);
            let game_result = if line.contains("1-0") || line.contains("1.0") {
                1.0
            } else if line.contains("1/2-1/2") || line.contains("0.5") {
                0.5
            } else {
                0.0
            };
            let state = GameState::from_fen(fen);
            return Some(LabelledGameState {
                game_state: state,
                label: game_result,
            });
        }
        None
    }

    pub fn next_texel_position(&mut self) -> Option<TexelState> {
        let state = self.next_position();
        if state.is_some() {
            let state = state.unwrap();
            let eval = eval_game_state(&state.game_state);
            return Some(TexelState {
                label: state.label,
                eval: eval.final_eval as f32,
                trace: eval.trace.collapse(),
            });
        }
        None
    }

    pub fn load_positions(&mut self, buf: &mut Vec<LabelledGameState>) {
        while let Some(pos) = self.next_position() {
            buf.push(pos);
        }
    }

    pub fn load_texel_positions(&mut self, buf: &mut Vec<TexelState>) {
        while let Some(pos) = self.next_texel_position() {
            buf.push(pos);
        }
    }
}
