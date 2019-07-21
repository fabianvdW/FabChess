use crate::board_representation::game_state::{GameResult, GameState};
use std::fmt::{Display, Formatter, Result};
use std::fs;
pub enum FileFormatSupported {
    OwnEncoding,
    PGN,
}
pub struct LabelledGameState {
    pub game_state: GameState,
    pub label: GameResult,
}
pub struct Statistics {
    pub games: usize,
    pub white_wins: usize,
    pub black_wins: usize,
    pub draws: usize,
}
impl Statistics {
    pub fn new() -> Self {
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

pub fn save_positions(to_file: &str, positions: &Vec<LabelledGameState>) {
    let mut res_str = String::new();
    for pos in positions {
        res_str.push_str(&format!(
            "{} |{}\n",
            pos.game_state.to_fen(),
            if let GameResult::WhiteWin = pos.label {
                "White"
            } else if let GameResult::BlackWin = pos.label {
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
        let new_linesplit = positions.split("\n").collect::<Vec<&str>>();
        let mut is_newgame = false;
        for line in new_linesplit {
            if line.contains("New Game") {
                is_newgame = true;
            } else {
                if !line.contains("|") {
                    break;
                }
                let fen_split = line.split("|").collect::<Vec<&str>>();
                let game_result = if fen_split[1].contains("Black") {
                    GameResult::BlackWin
                } else if fen_split[1].contains("White") {
                    GameResult::WhiteWin
                } else if fen_split[1].contains("Draw") {
                    GameResult::Draw
                } else {
                    panic!(format!("Invalid split {}", fen_split[1]));
                };
                if is_newgame {
                    is_newgame = false;
                    stats.games += 1;
                    if let GameResult::WhiteWin = game_result {
                        stats.white_wins += 1;
                    } else if let GameResult::BlackWin = game_result {
                        stats.black_wins += 1;
                    } else if let GameResult::Draw = game_result {
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
    }
    panic!("Not implemented");
}
