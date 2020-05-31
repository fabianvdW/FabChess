use core_sdk::board_representation::game_state::{
    char_to_file, char_to_rank, GameMove, GameMoveType, GameState, PieceType, WHITE,
};
use core_sdk::move_generation::makemove::make_move;
use core_sdk::move_generation::movegen;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
                    let last_state = &vec_gs[vec_gs.len() - 1];
                    let parsed_move = parse_move(last_state, &move_str, &mut self.move_list);
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
    for gmv in movelist.move_list.iter() {
        let mv = gmv.0;
        if mv.move_type == GameMoveType::Castle
            && mv.to as isize - mv.from as isize == 2 * if king_side { 1 } else { -1 }
        {
            let state = make_move(g, mv);
            return Ok((mv, state));
        }
    }
    Err(())
}

pub fn find_move(
    movelist: &movegen::MoveList,
    g: &GameState,
    ms: MoveSpecification,
) -> Result<(GameMove, GameState), ()> {
    for gmv in movelist.move_list.iter() {
        /*println!("Checking: {:?}", gmv.0);
        if &format!("{:?}", gmv.0) == "e4d4" {
            println!("{:?} ", ms.target_square);
            println!("{:?} ", ms.from_square);
            println!("{:?} ", ms.from_file);
            println!("{:?} ", ms.from_rank);
            println!("{:?} ", ms.moving_piece_type);
            println!("{:?} ", ms.promotion_piece);
        }*/

        if ms.matches(&gmv.0) {
            let state = make_move(g, gmv.0);
            return Ok((gmv.0, state));
        }
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
    if my_string.to_lowercase().ends_with('q') {
        promotion_piece = Some(PieceType::Queen)
    } else if my_string.to_lowercase().ends_with('r') {
        promotion_piece = Some(PieceType::Rook);
    } else if my_string.to_lowercase().ends_with('b') {
        promotion_piece = Some(PieceType::Bishop);
    } else if my_string.to_lowercase().ends_with('n') {
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
        mv.to as usize == self.target_square
            && (self.from_square.is_none() || self.from_square.unwrap() == mv.from as usize)
            && (self.from_file.is_none() || self.from_file.unwrap() == mv.from as usize % 8)
            && (self.from_rank.is_none() || self.from_rank.unwrap() == mv.from as usize / 8)
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
            if g.get_color_to_move() == WHITE {
                assert_eq!(true, g.castle_white_kingside());
            } else {
                assert_eq!(true, g.castle_black_kingside());
            }
            if let Ok(res) = find_castle(movelist, g, true) {
                return res;
            }
        } else {
            if g.get_color_to_move() == WHITE {
                assert_eq!(true, g.castle_white_queenside());
            } else {
                assert_eq!(true, g.castle_black_queenside());
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

    for gmv in movelist.move_list.iter() {
        println!("{:?}", gmv.0);
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
    char_to_file(c.expect("Invalid"))
}

pub fn match_rank(c: Option<char>) -> usize {
    char_to_rank(c.expect("Invalid"))
}

pub struct PGNParser {
    pub reader: BufReader<File>,
}

impl Iterator for PGNParser {
    type Item = String;

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
                if e.to_string().contains("stream did not contain valid UTF-8") {
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
