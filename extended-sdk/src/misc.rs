use crate::pgn::pgn_reader::*;
use core_sdk::board_representation::game_state_attack_container::GameStateAttackContainer;
use core_sdk::evaluation;
use core_sdk::logging::log;
use core_sdk::move_generation::movegen;
use std::fs::File;
use std::io::BufReader;

pub const STD_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const KING_BASE_PATH: [&str; 15] = [
    "./KingBase/KingBase2019-A00-A39.pgn",
    "./KingBase/KingBase2019-A40-A79.pgn",
    "./KingBase/KingBase2019-A80-A99.pgn",
    "./KingBase/KingBase2019-B00-B19.pgn",
    "./KingBase/KingBase2019-B20-B49.pgn",
    "./KingBase/KingBase2019-B50-B99.pgn",
    "./KingBase/KingBase2019-C00-C19.pgn",
    "./KingBase/KingBase2019-C20-C59.pgn",
    "./KingBase/KingBase2019-C60-C99.pgn",
    "./KingBase/KingBase2019-D00-D29.pgn",
    "./KingBase/KingBase2019-D30-D69.pgn",
    "./KingBase/KingBase2019-D70-D99.pgn",
    "./KingBase/KingBase2019-E00-E19.pgn",
    "./KingBase/KingBase2019-E20-E59.pgn",
    "./KingBase/KingBase2019-E60-E99.pgn",
];

#[allow(dead_code)]
pub fn to_string_board(board: u64) -> String {
    let mut res_str: String = String::new();
    res_str.push_str("+---+---+---+---+---+---+---+---+\n");
    for rank in 0..8 {
        res_str.push_str("| ");
        for file in 0..8 {
            let idx = 8 * (7 - rank) + file;
            if ((board >> idx) & 1) != 0 {
                res_str.push_str("X");
            } else {
                res_str.push_str(" ");
            }
            res_str.push_str(" | ");
        }
        res_str.push_str("\n+---+---+---+---+---+---+---+---+\n");
    }
    res_str
}

pub fn parse_pgn_find_static_eval_mistakes() {
    for path in &KING_BASE_PATH {
        let res = File::open(path);
        let file = match res {
            Err(why) => panic!("{}", why),
            Ok(file) => file,
        };
        let reader = BufReader::new(file);
        let parser = GameParser {
            pgn_parser: PGNParser { reader },
            is_opening: false,
            opening_load_untilply: 0usize,
            move_list: movegen::MoveList::default(),
            attack_container: GameStateAttackContainer::default(),
        };
        for _game in parser.into_iter() {
            let last_game_state = &_game.1[_game.1.len() - 1];
            let res = _game.2;
            let eval = evaluation::eval_game_state_from_null(&last_game_state).final_eval;
            if res == 1 {
                if eval < 0 {
                    log(&format!("{} (1-0)\n", &last_game_state.to_fen()));
                }
            } else if res == 0 {
                if eval.abs() > 100 {
                    log(&format!("{} (1/2-1/2)\n", &last_game_state.to_fen()));
                }
            } else if res == -1 && eval > 0 {
                log(&format!("{} (0-1)\n", &last_game_state.to_fen()));
            }
        }
    }
}
