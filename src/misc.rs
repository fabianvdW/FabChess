use super::board_representation::game_state::{
    GameMove, GameMoveType, GameState, PieceType, WHITE,
};
use super::evaluation;
use crate::logging::log;
use crate::move_generation::makemove::make_move;
use crate::move_generation::movegen;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::prelude::v1::Vec;

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
            Err(why) => panic!("{}", why.description()),
            Ok(file) => file,
        };
        let reader = BufReader::new(file);
        let parser = GameParser {
            pgn_parser: PGNParser { reader },
            is_opening: false,
            opening_load_untilply: 0usize,
            move_list: movegen::MoveList::default(),
        };
        for _game in parser.into_iter() {
            let last_game_state = &_game.1[_game.1.len() - 1];
            let res = _game.2;
            let eval = evaluation::eval_game_state(&last_game_state).final_eval;
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

pub struct GameParser {
    pub pgn_parser: PGNParser,
    pub is_opening: bool,
    pub opening_load_untilply: usize,
    pub move_list: movegen::MoveList,
}

impl Iterator for GameParser {
    type Item = (Vec<GameMove>, Vec<GameState>, isize);
    fn next(&mut self) -> Option<(Vec<GameMove>, Vec<GameState>, isize)> {
        let next = self.pgn_parser.next();
        let mut vec_res: Vec<GameMove> = Vec::new();
        let mut vec_gs: Vec<GameState> = Vec::new();
        vec_gs.push(GameState::standard());
        match next {
            None => None,
            Some(res) => {
                let game = res.replace("\r", "").replace("\n", " ");
                if game.contains("--") || game.contains('*') || game.contains("..") {
                    //Invalid state
                    return Some((vec_res, vec_gs, -2));
                }
                //log(&format!("{}\n", game));
                let moves = game.split(' ').collect::<Vec<&str>>();
                for move_str in moves.iter().take(moves.len() - 2) {
                    let mut move_str = (*move_str).to_string();
                    if move_str.contains('.') {
                        move_str = move_str.rsplit('.').collect::<Vec<&str>>()[0].to_string();
                    }
                    if move_str.is_empty() {
                        continue;
                    }
                    //println!("{} || len: {}", move_str, move_str.len());
                    let parsed_move =
                        parse_move(&vec_gs[vec_gs.len() - 1], &move_str, &mut self.move_list);
                    vec_gs.push(parsed_move.1);
                    vec_res.push(parsed_move.0);
                    if self.is_opening && vec_res.len() == self.opening_load_untilply {
                        break;
                    }
                }
                let last_elem = moves[moves.len() - 2];
                let mut score = 0;
                if last_elem == "1-0" {
                    score = 1;
                } else if last_elem == "0-1" {
                    score = -1;
                } else {
                    assert!(last_elem == "1/2-1/2");
                }
                Some((vec_res, vec_gs, score))
            }
        }
    }
}

pub fn find_castle(
    movelist: &movegen::MoveList,
    g: &GameState,
    king_side: bool,
) -> Result<(GameMove, GameState), ()> {
    let mut index = 0;
    while index < movelist.counter {
        let mv = movelist.move_list[index].as_ref().unwrap();
        if mv.move_type == GameMoveType::Castle
            && mv.to as isize - mv.from as isize == 2 * if king_side { 1 } else { -1 }
        {
            let state = make_move(g, mv);
            return Ok((*mv, state));
        }
        index += 1;
    }
    Err(())
}

pub fn find_move(
    movelist: &movegen::MoveList,
    g: &GameState,
    ms: MoveSpecification,
) -> Result<(GameMove, GameState), ()> {
    let mut index = 0;
    while index < movelist.counter {
        let mv = movelist.move_list[index].as_ref().unwrap();
        /*println!("Checking: {:?}", mv);
        if &format!("{:?}", mv) == "e4d4" {
            println!("{:?} ", ms.target_square);
            println!("{:?} ", ms.from_square);
            println!("{:?} ", ms.from_file);
            println!("{:?} ", ms.from_rank);
            println!("{:?} ", ms.moving_piece_type);
            println!("{:?} ", ms.promotion_piece);
        }*/

        if ms.matches(mv) {
            let state = make_move(g, mv);
            return Ok((*mv, state));
        }
        index += 1;
    }
    Err(())
}

pub fn get_piece_type(my_string: &mut String) -> PieceType {
    let mut moving_piece_type = PieceType::Pawn;
    if my_string.starts_with('N') {
        moving_piece_type = PieceType::Knight;
        *my_string = String::from(&my_string[1..]);
    } else if my_string.starts_with('B') {
        moving_piece_type = PieceType::Bishop;
        *my_string = String::from(&my_string[1..]);
    } else if my_string.starts_with('R') {
        moving_piece_type = PieceType::Rook;
        *my_string = String::from(&my_string[1..]);
    } else if my_string.starts_with('Q') {
        moving_piece_type = PieceType::Queen;
        *my_string = String::from(&my_string[1..]);
    } else if my_string.starts_with('K') {
        moving_piece_type = PieceType::King;
        *my_string = String::from(&my_string[1..]);
    }
    moving_piece_type
}

pub fn is_promotion(my_string: &mut String) -> Option<PieceType> {
    let mut promotion_piece = None;
    if my_string.ends_with('Q') {
        promotion_piece = Some(PieceType::Queen)
    } else if my_string.ends_with('R') {
        promotion_piece = Some(PieceType::Rook);
    } else if my_string.ends_with('B') {
        promotion_piece = Some(PieceType::Bishop);
    } else if my_string.ends_with('N') {
        promotion_piece = Some(PieceType::Knight);
    }
    if promotion_piece.is_some() {
        *my_string = String::from(&my_string[..my_string.len() - 1]);
    }
    promotion_piece
}

pub struct MoveSpecification {
    target_square: usize,
    from_square: Option<usize>,
    from_file: Option<usize>,
    from_rank: Option<usize>,
    moving_piece_type: PieceType,
    promotion_piece: Option<PieceType>,
}

impl MoveSpecification {
    pub fn new(
        target_square: usize,
        moving_piece_type: PieceType,
        promotion_piece: Option<PieceType>,
    ) -> Self {
        MoveSpecification {
            target_square,
            from_square: None,
            from_file: None,
            from_rank: None,
            moving_piece_type,
            promotion_piece,
        }
    }

    pub fn matches(&self, mv: &GameMove) -> bool {
        mv.to == self.target_square
            && (self.from_square.is_none() || self.from_square.unwrap() == mv.from)
            && (self.from_file.is_none() || self.from_file.unwrap() == mv.from % 8)
            && (self.from_rank.is_none() || self.from_rank.unwrap() == mv.from / 8)
            && (mv.piece_type == self.moving_piece_type || self.from_square.is_some())
            && (self.promotion_piece
                == match mv.move_type {
                    GameMoveType::Promotion(piece, _) => Some(piece),
                    _ => None,
                })
    }
}

pub fn parse_move(
    g: &GameState,
    move_str: &str,
    movelist: &mut movegen::MoveList,
) -> (GameMove, GameState) {
    let mut my_string = move_str.to_string();
    my_string = my_string
        .replace("#", "")
        .replace("+", "")
        .replace("=", "")
        .replace("x", "");
    movegen::generate_moves(&g, false, movelist);
    if my_string.contains('-') {
        //Castle
        //Kingside
        if my_string.len() == 3 {
            if g.color_to_move == WHITE {
                assert_eq!(true, g.castle_white_kingside);
            } else {
                assert_eq!(true, g.castle_black_kingside);
            }
            if let Ok(res) = find_castle(movelist, g, true) {
                return res;
            }
        } else {
            if g.color_to_move == WHITE {
                assert_eq!(true, g.castle_white_queenside);
            } else {
                assert_eq!(true, g.castle_black_queenside);
            }
            if let Ok(res) = find_castle(movelist, g, false) {
                return res;
            }
        }
    } else {
        let moving_piece_type = get_piece_type(&mut my_string);
        let promotion_piece = is_promotion(&mut my_string);
        let target_square = 8 * match_rank(my_string.chars().nth(my_string.len() - 1))
            + match_file(my_string.chars().nth(my_string.len() - 2));
        let mut ms = MoveSpecification::new(target_square, moving_piece_type, promotion_piece);

        if my_string.len() == 3 {
            let first = my_string.chars().nth(0);
            if is_file(first) {
                ms.from_file = Some(match_file(first));
            } else {
                ms.from_rank = Some(match_rank(first));
            }
        } else if my_string.len() == 4 {
            ms.from_square = Some(
                8 * match_rank(my_string.chars().nth(1)) + match_file(my_string.chars().nth(0)),
            );
        }
        if let Ok(res) = find_move(movelist, g, ms) {
            return res;
        }
    }
    println!("{}", move_str);
    println!("{}", my_string);
    println!("{}", g);

    let mut index = 0;
    while index < movelist.counter {
        println!("{:?}", movelist.move_list[index].as_ref().unwrap());
        index += 1;
    }
    panic!("Shouldn't get here");
}

pub fn is_file(c: Option<char>) -> bool {
    match c {
        None => panic!("Invalid!"),
        Some(character) => match character {
            'a' => true,
            'b' => true,
            'c' => true,
            'd' => true,
            'e' => true,
            'f' => true,
            'g' => true,
            'h' => true,
            _ => false,
        },
    }
}

pub fn match_file(c: Option<char>) -> usize {
    match c {
        None => panic!("Invalid!"),
        Some(character) => match character {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => panic!("Invalid rank!"),
        },
    }
}

pub fn match_rank(c: Option<char>) -> usize {
    match c {
        None => panic!("Invalid!"),
        Some(character) => match character {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            _ => panic!("Invalid rank!"),
        },
    }
}

pub struct PGNParser {
    pub reader: BufReader<File>,
}

impl Iterator for PGNParser {
    type Item = (String);

    fn next(&mut self) -> Option<String> {
        let mut res_str = String::new();
        let mut line = String::new();
        let mut res = self.reader.read_line(&mut line);
        while match res {
            Err(_e) => false,
            Ok(_e) => true,
        } {
            if line.contains("1.") && !line.contains('[') {
                loop {
                    res_str.push_str(&line);
                    if res_str.contains("1-0")
                        || res_str.contains("0-1")
                        || res_str.contains("1/2-1/2")
                        || res_str.contains('*')
                    {
                        break;
                    }
                    line = String::new();
                    self.reader
                        .read_line(&mut line)
                        .expect("Reader had an error reading moves of game!");
                }
                break;
            }
            line = String::new();
            res = self.reader.read_line(&mut line);
            if let Err(e) = &res {
                if e.description()
                    .contains("stream did not contain valid UTF-8")
                {
                    res = Ok(1);
                }
            }
            if let Ok(e) = &res {
                if *e == 0usize {
                    break;
                }
            }
        }
        if !res_str.is_empty() {
            Some(res_str)
        } else {
            None
        }
    }
}
