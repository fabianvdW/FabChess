use std::fmt::{Formatter, Display, Result, Debug};
use super::zobrist_hashing::ZOBRIST_KEYS;

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum GameMoveType {
    Quiet,
    Capture(PieceType),
    EnPassant,
    Castle,
    Promotion(PieceType, Option<PieceType>),
}

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum PieceType {
    King,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
}

#[derive(Clone, Copy)]
pub struct GameMove {
    pub from: usize,
    pub to: usize,
    pub move_type: GameMoveType,
    pub piece_type: PieceType,
}

impl GameMove {
    pub fn clone(&self) -> GameMove {
        GameMove {
            from: self.from,
            to: self.to,
            move_type: self.move_type.clone(),
            piece_type: self.piece_type.clone(),
        }
    }

    pub fn string_to_move(desc: &str) -> (usize, usize, Option<PieceType>) {
        let mut chars = desc.chars();
        let from_file = match chars.nth(0) {
            Some(s) => {
                char_to_file(s)
            }
            _ => {
                panic!("Invalid move desc!");
            }
        };
        let from_rank = match chars.nth(0) {
            Some(s) => {
                char_to_rank(s)
            }
            _ => {
                panic!("Invalid move desc!");
            }
        };
        let to_file = match chars.nth(0) {
            Some(s) => {
                char_to_file(s)
            }
            _ => {
                panic!("Invalid move desc!");
            }
        };
        let to_rank = match chars.nth(0) {
            Some(s) => {
                char_to_rank(s)
            }
            _ => {
                panic!("Invalid move desc!");
            }
        };
        if desc.len() == 5 {
            return (from_file + 8 * from_rank, to_file + 8 * to_rank, Some(char_to_promotion_piecetype(match chars.nth(0) {
                Some(s) => s,
                _ => panic!("Invalid move desc!")
            })));
        }
        (from_file + 8 * from_rank, to_file + 8 * to_rank, None)
    }
}

impl Debug for GameMove {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!("{}{}{}{}", file_to_string(self.from % 8), self.from / 8 + 1, file_to_string(self.to % 8), self.to / 8 + 1));
        match &self.move_type {
            GameMoveType::Promotion(s, _) => {
                match s {
                    PieceType::Queen => { res_str.push_str("q") }
                    PieceType::Rook => { res_str.push_str("r") }
                    PieceType::Bishop => { res_str.push_str("b") }
                    PieceType::Knight => { res_str.push_str("k") }
                    _ => panic!("Invalid promotion piece type!"),
                }
            }
            _ => {}
        };
        write!(formatter, "{}", res_str)
    }
}

fn char_to_promotion_piecetype(c: char) -> PieceType {
    match c {
        'q' => PieceType::Queen,
        'r' => PieceType::Rook,
        'b' => PieceType::Bishop,
        'n' => PieceType::Knight,
        _ => {
            panic!("Invalid promotion piece")
        }
    }
}

fn char_to_rank(c: char) -> usize {
    match c {
        '1' => 0,
        '2' => 1,
        '3' => 2,
        '4' => 3,
        '5' => 4,
        '6' => 5,
        '7' => 6,
        '8' => 7,
        _ => {
            panic!("Invalid rank");
        }
    }
}

fn char_to_file(c: char) -> usize {
    match c {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => {
            panic!("Invalid char");
        }
    }
}

fn file_to_string(file: usize) -> &'static str {
    match file {
        0 => "a",
        1 => "b",
        2 => "c",
        3 => "d",
        4 => "e",
        5 => "f",
        6 => "g",
        7 => "h",
        _ => panic!("invalid file")
    }
}

pub struct GameState {
    // 0 = white
    // 1 = black
    pub color_to_move: usize,

    //Array saving all the bitboards
    //Index 1:
    // 0 -> Pawns
    // 1 -> Knights
    // 2 -> Bishops
    // 3 -> Rooks
    // 4 -> Queens
    // 5 -> King
    //Index 2:
    // 0 -> White
    // 1 -> Black
    pub pieces: [[u64; 2]; 6],

    //Castle flags
    pub castle_white_kingside: bool,
    pub castle_white_queenside: bool,
    pub castle_black_kingside: bool,
    pub castle_black_queenside: bool,

    pub en_passant: u64,
    //50 move draw counter
    pub half_moves: usize,
    pub full_moves: usize,
    pub hash: u64,
}

impl GameState {
    pub fn from_fen(fen: &str) -> GameState {
        let vec: Vec<&str> = fen.split(" ").collect();
        if vec.len() < 4 {
            panic!("Invalid FEN");
        }
        //Parse through FEN
        //Piecess
        let pieces: Vec<&str> = vec[0].split("/").collect();
        if pieces.len() != 8 {
            panic!("Invalid FEN");
        }
        //Iterate over all 8 ranks
        let mut pieces_arr: [[u64; 2]; 6] = [[0u64; 2]; 6];
        for rank in 0..8 {
            let rank_str = pieces[rank];
            let mut file: usize = 0;
            let mut rank_str_idx: usize = 0;
            while file < 8 {
                let idx = (7 - rank) * 8 + file;
                let next_char = rank_str.chars().nth(rank_str_idx);
                rank_str_idx += 1;
                match next_char {
                    Some(x) => {
                        match x {
                            'P' => {
                                pieces_arr[0][0] |= 1u64 << idx;
                                file += 1;
                            }
                            'p' => {
                                pieces_arr[0][1] |= 1u64 << idx;
                                file += 1;
                            }
                            'N' => {
                                pieces_arr[1][0] |= 1u64 << idx;
                                file += 1;
                            }
                            'n' => {
                                pieces_arr[1][1] |= 1u64 << idx;
                                file += 1;
                            }
                            'B' => {
                                pieces_arr[2][0] |= 1u64 << idx;
                                file += 1;
                            }
                            'b' => {
                                pieces_arr[2][1] |= 1u64 << idx;
                                file += 1;
                            }
                            'R' => {
                                pieces_arr[3][0] |= 1u64 << idx;
                                file += 1;
                            }
                            'r' => {
                                pieces_arr[3][1] |= 1u64 << idx;
                                file += 1;
                            }
                            'Q' => {
                                pieces_arr[4][0] |= 1u64 << idx;
                                file += 1;
                            }
                            'q' => {
                                pieces_arr[4][1] |= 1u64 << idx;
                                file += 1;
                            }
                            'K' => {
                                pieces_arr[5][0] |= 1u64 << idx;
                                file += 1;
                            }
                            'k' => {
                                pieces_arr[5][1] |= 1u64 << idx;
                                file += 1;
                            }
                            '1' => {
                                file += 1;
                            }
                            '2' => {
                                file += 2;
                            }
                            '3' => {
                                file += 3;
                            }
                            '4' => {
                                file += 4;
                            }
                            '5' => {
                                file += 5;
                            }
                            '6' => {
                                file += 6;
                            }
                            '7' => {
                                file += 7;
                            }
                            '8' => {
                                file += 8;
                            }
                            _ => {
                                panic!("Invalid FEN");
                            }
                        }
                    }
                    None => panic!("Invalid FEN"),
                }
            }
        }

        //Side to move
        let color_to_move = match vec[1] {
            "w" => 0,
            "b" => 1,
            _ => panic!("Invalid FEN!")
        };

        //CastlingAbilities
        let castle_white_kingside = vec[2].contains("K");
        let castle_white_queenside = vec[2].contains("Q");
        let castle_black_kingside = vec[2].contains("k");
        let castle_black_queenside = vec[2].contains("q");

        //En Passant target square
        let mut en_passant: u64 = 0u64;
        if vec[3] != "-" {
            let mut idx: usize = 0usize;
            let file = vec[3].chars().nth(0);
            let rank = vec[3].chars().nth(1);
            match file {
                Some(x) => {
                    match x {
                        'a' | 'A' => {}
                        'b' | 'B' => {
                            idx += 1;
                        }
                        'c' | 'C' => {
                            idx += 2;
                        }
                        'd' | 'D' => {
                            idx += 3;
                        }
                        'e' | 'E' => {
                            idx += 4;
                        }
                        'f' | 'F' => {
                            idx += 5;
                        }
                        'g' | 'G' => {
                            idx += 6;
                        }
                        'h' | 'H' => {
                            idx += 7;
                        }
                        _ => {
                            panic!("Invalid FEN!");
                        }
                    }
                }
                None => { panic!("Invalid FEN!"); }
            }
            match rank {
                Some(x) => {
                    match x {
                        '1' => {}
                        '2' => {
                            idx += 8;
                        }
                        '3' => {
                            idx += 16;
                        }
                        '4' => {
                            idx += 24;
                        }
                        '5' => {
                            idx += 32;
                        }
                        '6' => {
                            idx += 40;
                        }
                        '7' => {
                            idx += 48;
                        }
                        '8' => {
                            idx += 56;
                        }
                        _ => {
                            panic!("Invalid FEN!");
                        }
                    }
                }
                None => {
                    panic!("Invalid FEN!");
                }
            }
            en_passant = 1u64 << idx;
        }
        let mut half_moves = 0;
        let mut full_moves = 1;
        if vec.len() > 4 {
            //HalfMoveClock
            half_moves = vec[4].parse().unwrap();

            full_moves = vec[5].parse().unwrap();
        }
        let hash = GameState::calculate_zobrist_hash(color_to_move, pieces_arr, castle_white_kingside, castle_white_queenside, castle_black_kingside, castle_black_queenside, en_passant);
        GameState {
            color_to_move,
            pieces: pieces_arr,
            castle_white_kingside,
            castle_white_queenside,
            castle_black_kingside,
            castle_black_queenside,
            half_moves,
            full_moves,
            en_passant,
            hash,
        }
    }
    pub fn to_fen(&self) -> String {
        let mut res_str = String::new();
        for rank in 0..8 {
            let big_endian_rank = 7 - rank;
            //bitscan forward
            let mut file = 0;
            let mut files_skipped = 0;
            while file < 8 {
                let shift = big_endian_rank * 8 + file;
                file += 1;
                files_skipped += 1;
                if (self.pieces[0][0] >> shift) & 1u64 != 0u64 {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str("P");
                } else if (self.pieces[1][0] >> shift) & 1u64 != 0u64 {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str("N");
                } else if (self.pieces[2][0] >> shift) & 1u64 != 0u64 {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str("B");
                } else if (self.pieces[3][0] >> shift) & 1u64 != 0u64 {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str("R");
                } else if (self.pieces[4][0] >> shift) & 1u64 != 0u64 {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str("Q");
                } else if (self.pieces[5][0] >> shift) & 1u64 != 0u64 {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str("K");
                } else if (self.pieces[0][1] >> shift) & 1u64 != 0u64 {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str("p");
                } else if (self.pieces[1][1] >> shift) & 1u64 != 0u64 {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str("n");
                } else if (self.pieces[2][1] >> shift) & 1u64 != 0u64 {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str("b");
                } else if (self.pieces[3][1] >> shift) & 1u64 != 0u64 {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str("r");
                } else if (self.pieces[4][1] >> shift) & 1u64 != 0u64 {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str("q");
                } else if (self.pieces[5][1] >> shift) & 1u64 != 0u64 {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str("k");
                }
            }
            if files_skipped != 0 {
                res_str.push_str(&format!("{}", files_skipped));
            }
            if rank != 7 {
                res_str.push_str("/");
            }
        }
        res_str.push_str(" ");
        if self.color_to_move == 0 {
            res_str.push_str("w");
        } else {
            res_str.push_str("b");
        }
        res_str.push_str(" ");
        if !(self.castle_white_kingside | self.castle_white_queenside | self.castle_black_kingside | self.castle_black_queenside) {
            res_str.push_str("-");
        } else {
            if self.castle_white_kingside {
                res_str.push_str("K");
            }
            if self.castle_white_queenside {
                res_str.push_str("Q");
            }
            if self.castle_black_kingside {
                res_str.push_str("k");
            }
            if self.castle_black_queenside {
                res_str.push_str("q");
            }
        }
        res_str.push_str(" ");

        if self.en_passant == 0u64 {
            res_str.push_str("-");
        } else {
            let idx = self.en_passant.trailing_zeros() as usize;
            let rank = idx / 8;
            let file = idx % 8;
            res_str.push_str(&format!("{}{}", file_to_string(file), rank + 1));
        }
        res_str.push_str(" ");
        res_str.push_str(&format!("{} ", self.half_moves));
        res_str.push_str(&format!("{}", self.full_moves));
        res_str
    }
    pub fn standard() -> GameState {
        let color_to_move = 0usize;
        let pieces = [[0xff00u64, 0xff000000000000u64], [0x42u64, 0x4200000000000000u64], [0x24u64, 0x2400000000000000u64], [0x81u64, 0x8100000000000000u64],
            [0x8u64, 0x800000000000000u64], [0x10u64, 0x1000000000000000u64]];

        GameState {
            color_to_move,
            pieces,
            castle_white_kingside: true,
            castle_white_queenside: true,
            castle_black_kingside: true,
            castle_black_queenside: true,
            en_passant: 0u64,
            half_moves: 0usize,
            full_moves: 1usize,
            hash: GameState::calculate_zobrist_hash(color_to_move, pieces, true, true, true, true, 0u64),
        }
    }
    pub fn calculate_zobrist_hash(color_to_move: usize, pieces: [[u64; 2]; 6], cwk: bool, cwq: bool, cbk: bool, cbq: bool, ep: u64) -> u64 {
        let mut hash = 0u64;
        if color_to_move == 1 {
            hash ^= ZOBRIST_KEYS.side_to_move;
        }
        if cwk {
            hash ^= ZOBRIST_KEYS.castle_w_kingside;
        }
        if cwq {
            hash ^= ZOBRIST_KEYS.castle_w_queenside;
        }
        if cbk {
            hash ^= ZOBRIST_KEYS.castle_b_kingside;
        }
        if cbq {
            hash ^= ZOBRIST_KEYS.castle_b_queenside;
        }
        if ep != 0u64 {
            let file = ep.trailing_zeros() as usize % 8;
            hash ^= ZOBRIST_KEYS.en_passant[file];
        }
        //W Pawns
        let mut w_pawns = pieces[0][0];
        while w_pawns != 0u64 {
            let idx = w_pawns.trailing_zeros() as usize;
            hash ^= ZOBRIST_KEYS.w_pawns[idx];
            w_pawns ^= 1u64 << idx;
        }
        let mut w_knights = pieces[1][0];
        while w_knights != 0u64 {
            let idx = w_knights.trailing_zeros() as usize;
            hash ^= ZOBRIST_KEYS.w_knights[idx];
            w_knights ^= 1u64 << idx;
        }
        let mut w_bishops = pieces[2][0];
        while w_bishops != 0u64 {
            let idx = w_bishops.trailing_zeros() as usize;
            hash ^= ZOBRIST_KEYS.w_bishops[idx];
            w_bishops ^= 1u64 << idx;
        }
        let mut w_rooks = pieces[3][0];
        while w_rooks != 0u64 {
            let idx = w_rooks.trailing_zeros() as usize;
            hash ^= ZOBRIST_KEYS.w_rooks[idx];
            w_rooks ^= 1u64 << idx;
        }
        let mut w_queens = pieces[4][0];
        while w_queens != 0u64 {
            let idx = w_queens.trailing_zeros() as usize;
            hash ^= ZOBRIST_KEYS.w_queens[idx];
            w_queens ^= 1u64 << idx;
        }
        let mut w_king = pieces[5][0];
        while w_king != 0u64 {
            let idx = w_king.trailing_zeros() as usize;
            hash ^= ZOBRIST_KEYS.w_king[idx];
            w_king ^= 1u64 << idx;
        }
        let mut b_pawns = pieces[0][1];
        while b_pawns != 0u64 {
            let idx = b_pawns.trailing_zeros() as usize;
            hash ^= ZOBRIST_KEYS.b_pawns[idx];
            b_pawns ^= 1u64 << idx;
        }
        let mut b_knights = pieces[1][1];
        while b_knights != 0u64 {
            let idx = b_knights.trailing_zeros() as usize;
            hash ^= ZOBRIST_KEYS.b_knights[idx];
            b_knights ^= 1u64 << idx;
        }
        let mut b_bishops = pieces[2][1];
        while b_bishops != 0u64 {
            let idx = b_bishops.trailing_zeros() as usize;
            hash ^= ZOBRIST_KEYS.b_bishops[idx];
            b_bishops ^= 1u64 << idx;
        }
        let mut b_rooks = pieces[3][1];
        while b_rooks != 0u64 {
            let idx = b_rooks.trailing_zeros() as usize;
            hash ^= ZOBRIST_KEYS.b_rooks[idx];
            b_rooks ^= 1u64 << idx;
        }
        let mut b_queens = pieces[4][1];
        while b_queens != 0u64 {
            let idx = b_queens.trailing_zeros() as usize;
            hash ^= ZOBRIST_KEYS.b_queens[idx];
            b_queens ^= 1u64 << idx;
        }
        let mut b_king = pieces[5][1];
        while b_king != 0u64 {
            let idx = b_king.trailing_zeros() as usize;
            hash ^= ZOBRIST_KEYS.b_king[idx];
            b_king ^= 1u64 << idx;
        }
        hash
    }
}
impl Clone for GameState{
    fn clone(&self) -> Self {
        GameState {
            color_to_move: self.color_to_move,
            pieces: self.pieces.clone(),
            castle_white_kingside: self.castle_white_kingside,
            castle_white_queenside: self.castle_white_queenside,
            castle_black_kingside: self.castle_black_kingside,
            castle_black_queenside: self.castle_black_queenside,
            en_passant: self.en_passant,
            half_moves: self.half_moves,
            full_moves: self.full_moves,
            hash: self.hash,
        }
    }
}

impl Display for GameState {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str("+---+---+---+---+---+---+---+---+\n");
        for rank in 0..8 {
            res_str.push_str("| ");
            for file in 0..8 {
                let idx = 8 * (7 - rank) + file;
                if (self.pieces[0][0] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("P");
                } else if (self.pieces[0][1] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("p");
                } else if (self.pieces[1][0] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("N");
                } else if (self.pieces[1][1] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("n");
                } else if (self.pieces[2][0] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("B");
                } else if (self.pieces[2][1] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("b");
                } else if (self.pieces[3][0] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("R");
                } else if (self.pieces[3][1] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("r");
                } else if (self.pieces[4][0] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("Q");
                } else if (self.pieces[4][1] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("q");
                } else if (self.pieces[5][0] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("K");
                } else if (self.pieces[5][1] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("k");
                } else {
                    res_str.push_str(" ");
                }
                res_str.push_str(" | ");
            }
            res_str.push_str("\n+---+---+---+---+---+---+---+---+\n");
        }
        res_str.push_str("Castle Rights: \n");
        res_str.push_str(&format!("White Kingside: {}\n", self.castle_white_kingside));
        res_str.push_str(&format!("White Queenside: {}\n", self.castle_white_queenside));
        res_str.push_str(&format!("Black Kingside: {}\n", self.castle_black_kingside));
        res_str.push_str(&format!("Black Queenside: {}\n", self.castle_black_queenside));
        res_str.push_str(&format!("En Passant Possible: {:x}\n", self.en_passant));
        res_str.push_str(&format!("Half-Counter: {}\n", self.half_moves));
        res_str.push_str(&format!("Full-Counter: {}\n", self.full_moves));
        res_str.push_str(&format!("Side to Move: {}\n", self.color_to_move));
        res_str.push_str(&format!("Hash: {}\n", self.hash));
        write!(formatter, "{}", res_str)
    }
}

impl Debug for GameState {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!("Color: {}\n", self.color_to_move));
        res_str.push_str(&format!("WhitePawns: 0x{:x}u64\n", self.pieces[0][0]));
        res_str.push_str(&format!("WhiteKnights: 0x{:x}u64\n", self.pieces[1][0]));
        res_str.push_str(&format!("WhiteBishops: 0x{:x}u64\n", self.pieces[2][0]));
        res_str.push_str(&format!("WhiteRooks: 0x{:x}u64\n", self.pieces[3][0]));
        res_str.push_str(&format!("WhiteQueens: 0x{:x}u64\n", self.pieces[4][0]));
        res_str.push_str(&format!("WhiteKing: 0x{:x}u64\n", self.pieces[5][0]));
        res_str.push_str(&format!("BlackPawns: 0x{:x}u64\n", self.pieces[0][1]));
        res_str.push_str(&format!("BlackKnights: 0x{:x}u64\n", self.pieces[1][1]));
        res_str.push_str(&format!("BlackBishops: 0x{:x}u64\n", self.pieces[2][1]));
        res_str.push_str(&format!("BlackRooks: 0x{:x}u64\n", self.pieces[3][1]));
        res_str.push_str(&format!("BlackQueens: 0x{:x}u64\n", self.pieces[4][1]));
        res_str.push_str(&format!("BlackKing: 0x{:x}u64\n", self.pieces[5][1]));
        res_str.push_str(&format!("CWK: {}\n", self.castle_white_kingside));
        res_str.push_str(&format!("CWQ: {}\n", self.castle_white_queenside));
        res_str.push_str(&format!("CBK: {}\n", self.castle_black_kingside));
        res_str.push_str(&format!("CBQ: {}\n", self.castle_black_queenside));
        res_str.push_str(&format!("En-Passant: 0x{:x}u64\n", self.en_passant));
        res_str.push_str(&format!("half_moves: {}\n", self.half_moves));
        res_str.push_str(&format!("full_moves: {}\n", self.full_moves));
        res_str.push_str(&format!("Side to Move: {}\n", self.color_to_move));
        res_str.push_str(&format!("Hash: {}\n", self.hash));
        write!(formatter, "{}", res_str)
    }
}