use std::fmt::{Formatter, Display, Result, Debug};

#[derive(PartialEq, Clone)]
pub enum GameMoveType {
    Quiet,
    Capture,
    EnPassant,
    Castle,
    Promotion(PieceType),
}

#[derive(PartialEq, Clone, Debug)]
pub enum PieceType {
    King,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
}

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
}

impl Debug for GameMove {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!("{}{}{}{}", file_to_string(self.from % 8), self.from / 8 + 1, file_to_string(self.to % 8), self.to / 8 + 1));
        match &self.move_type {
            GameMoveType::Capture => { res_str.push_str("") }
            _ => {}
        };
        write!(formatter, "{}", res_str)
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
        let mut full_moves = 0;
        if vec.len() > 4 {
            //HalfMoveClock
            half_moves = vec[4].parse().unwrap();

            full_moves = vec[5].parse().unwrap();
        }
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
        GameState {
            color_to_move: 0usize,
            pieces: [[0xff00u64, 0xff000000000000u64], [0x42u64, 0x4200000000000000u64], [0x24u64, 0x2400000000000000u64], [0x81u64, 0x8100000000000000u64],
                [0x8u64, 0x800000000000000u64], [0x10u64, 0x1000000000000000u64]],
            castle_white_kingside: true,
            castle_white_queenside: true,
            castle_black_kingside: true,
            castle_black_queenside: true,
            en_passant: 0u64,
            half_moves: 0usize,
            full_moves: 1usize,
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
        write!(formatter, "{}", res_str)
    }
}