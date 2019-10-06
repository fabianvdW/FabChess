use crate::board_representation::game_state::GameState;
use std::fmt::{Display, Formatter, Result};
use std::fs;

pub enum FileFormatSupported {
    OwnEncoding,
    EPD,
    PGN,
}

pub struct LabelledGameState {
    pub game_state: GameState,
    pub label: f64,
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
            if (pos.label - 1.0).abs() < std::f64::EPSILON {
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

pub fn load_positions(
    from_file: &str,
    file_format: FileFormatSupported,
    buf: &mut Vec<LabelledGameState>,
    stats: &mut Statistics,
) {
    if let FileFormatSupported::OwnEncoding = file_format {
        let positions =
            fs::read_to_string(from_file).expect("Unable to read benchmarking positions");
        let new_linesplit = positions.split('\n').collect::<Vec<&str>>();
        let mut is_newgame = false;
        for line in new_linesplit {
            if line.contains("New Game") {
                is_newgame = true;
            } else {
                if !line.contains('|') {
                    break;
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
                if is_newgame {
                    is_newgame = false;
                    stats.games += 1;
                    if game_result - 1.0 < std::f64::EPSILON {
                        stats.white_wins += 1;
                    } else if game_result == 0.0 {
                        stats.black_wins += 1;
                    } else if game_result - 0.5 < std::f64::EPSILON {
                        stats.draws += 1;
                    }
                }

                buf.push(LabelledGameState {
                    game_state: GameState::from_fen(fen_split[0]),
                    label: game_result,
                });
            }
        }
        return;
    } else if let FileFormatSupported::EPD = file_format {
        let positions =
            fs::read_to_string(from_file).expect("Unable to read benchmarking positions");
        let new_linesplit = positions.split('\n').collect::<Vec<&str>>();
        for line in new_linesplit {
            if !line.contains(';') {
                break;
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
            buf.push(LabelledGameState {
                game_state: GameState::from_fen(fen),
                label: game_result,
            });
        }
        return;
    }
    panic!("Not implemented");
}
