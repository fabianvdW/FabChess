use crate::pgn::pgn_reader::{GameParser, PGNParser};
use core_sdk::board_representation::game_state::{GameMove, GameState};
use core_sdk::move_generation::movelist;
use std::fs::File;
use std::io::BufReader;

pub fn load_db_until(db: &str, until: usize) -> (Vec<GameState>, Vec<Vec<GameMove>>) {
    let movelist = movelist::MoveList::default();
    let attack_container =
        core_sdk::board_representation::game_state_attack_container::GameStateAttackContainer::default(
        );
    let mut res: Vec<GameState> = Vec::with_capacity(100_000);
    let mut res_mvs = Vec::with_capacity(100_000);
    let res_file = File::open(db).expect("Unable to open opening database");
    let reader = BufReader::new(res_file);
    let parser = GameParser {
        pgn_parser: PGNParser { reader },
        is_opening: true,
        opening_load_untilply: until,
        move_list: movelist,
        attack_container,
    };
    for game in parser {
        if game.1.len() > until {
            let state: GameState = game.1[until].clone();
            res.push(state);
            res_mvs.push(game.0[..until].to_vec());
        }
    }
    (res, res_mvs)
}
