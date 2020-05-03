use crate::bitboards::bitboards::constants::{square, KING_ATTACKS, KNIGHT_ATTACKS};
use crate::board_representation::game_state_attack_container::GameStateAttackContainer;
use crate::board_representation::zobrist_hashing::ZOBRIST_KEYS;
use crate::evaluation::params::*;
use crate::evaluation::phase::Phase;
use crate::evaluation::EvaluationScore;
use crate::move_generation::makemove::make_move;
use crate::move_generation::movegen::{
    b_pawn_east_targets, b_pawn_west_targets, bishop_attacks, double_push_pawn_targets,
    generate_moves, pawn_east_targets, pawn_west_targets, rook_attacks, single_push_pawn_targets,
    w_pawn_east_targets, w_pawn_west_targets,
};
use crate::move_generation::movelist::MoveList;
use std::fmt::{Debug, Display, Formatter, Result};

//TODO Remove and add as usize
pub const PAWN: usize = 0;
pub const KNIGHT: usize = 1;
pub const BISHOP: usize = 2;
pub const ROOK: usize = 3;
pub const QUEEN: usize = 4;
pub const KING: usize = 5;
pub const WHITE: usize = 0;
pub const BLACK: usize = 1;

#[derive(PartialEq, Debug)]
pub enum GameResult {
    Ingame,
    WhiteWin,
    BlackWin,
    Draw,
}
impl Display for GameResult {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let res_str = String::from(match self {
            GameResult::Ingame => "*",
            GameResult::WhiteWin => "1-0",
            GameResult::BlackWin => "0-1",
            GameResult::Draw => "1/2-1/2",
        });
        write!(formatter, "{}", res_str)
    }
}
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
impl PieceType {
    #[inline(always)]
    pub fn to_index(self) -> usize {
        match &self {
            PieceType::Pawn => PAWN,
            PieceType::Knight => KNIGHT,
            PieceType::Bishop => BISHOP,
            PieceType::Rook => ROOK,
            PieceType::Queen => QUEEN,
            PieceType::King => KING,
        }
    }
    #[inline(always)]
    pub fn to_psqt(self) -> &'static [[EvaluationScore; 8]; 8] {
        match &self {
            PieceType::Pawn => &PSQT_PAWN,
            PieceType::Knight => &PSQT_KNIGHT,
            PieceType::Bishop => &PSQT_BISHOP,
            PieceType::Rook => &PSQT_ROOK,
            PieceType::Queen => &PSQT_QUEEN,
            PieceType::King => &PSQT_KING,
        }
    }

    #[inline(always)]
    pub fn to_zobrist_key(self) -> (&'static [u64; 64], &'static [u64; 64]) {
        match &self {
            PieceType::Pawn => (&ZOBRIST_KEYS.w_pawns, &ZOBRIST_KEYS.b_pawns),
            PieceType::Knight => (&ZOBRIST_KEYS.w_knights, &ZOBRIST_KEYS.b_knights),
            PieceType::Bishop => (&ZOBRIST_KEYS.w_bishops, &ZOBRIST_KEYS.b_bishops),
            PieceType::Rook => (&ZOBRIST_KEYS.w_rooks, &ZOBRIST_KEYS.b_rooks),
            PieceType::Queen => (&ZOBRIST_KEYS.w_queens, &ZOBRIST_KEYS.b_queens),
            PieceType::King => (&ZOBRIST_KEYS.w_king, &ZOBRIST_KEYS.b_king),
        }
    }

    #[inline(always)]
    pub fn to_piece_score(self) -> EvaluationScore {
        match &self {
            PieceType::Pawn => PAWN_PIECE_VALUE,
            PieceType::Knight => KNIGHT_PIECE_VALUE,
            PieceType::Bishop => BISHOP_PIECE_VALUE,
            PieceType::Rook => ROOK_PIECE_VALUE,
            PieceType::Queen => QUEEN_PIECE_VALUE,
            PieceType::King => panic!("King has no piece score"),
        }
    }

    #[inline(always)]
    pub fn to_phase_score(self) -> i16 {
        match &self {
            PieceType::Pawn => 0,
            PieceType::Knight => 500,
            PieceType::Bishop => 510,
            PieceType::Rook => 650,
            PieceType::Queen => 1500,
            PieceType::King => panic!("King has no phase score"),
        }
    }
}

#[derive(Copy, PartialEq)]
pub struct GameMove {
    pub from: u8,
    pub to: u8,
    pub move_type: GameMoveType,
    pub piece_type: PieceType,
}

impl Clone for GameMove {
    fn clone(&self) -> Self {
        GameMove {
            from: self.from,
            to: self.to,
            move_type: self.move_type,
            piece_type: self.piece_type,
        }
    }
}

impl GameMove {
    #[inline(always)]
    pub fn is_capture(self) -> bool {
        match self.move_type {
            GameMoveType::Capture(_) => true,
            GameMoveType::Promotion(_, s) => match s {
                Some(_) => true,
                _ => false,
            },
            GameMoveType::EnPassant => true,
            _ => false,
        }
    }
    #[inline(always)]
    pub fn get_captured_piece(self) -> PieceType {
        debug_assert!(self.is_capture());
        match self.move_type {
            GameMoveType::Capture(p) => p,
            GameMoveType::Promotion(_, Some(p)) => p,
            GameMoveType::EnPassant => PieceType::Pawn,
            _ => panic!("Captured piece type  called on a capture"),
        }
    }
    pub fn string_to_move(desc: &str) -> (usize, usize, Option<PieceType>) {
        let mut chars = desc.chars();
        let from_file = match chars.next() {
            Some(s) => char_to_file(s),
            _ => {
                panic!("Invalid move desc!");
            }
        };
        let from_rank = match chars.next() {
            Some(s) => char_to_rank(s),
            _ => {
                panic!("Invalid move desc!");
            }
        };
        let to_file = match chars.next() {
            Some(s) => char_to_file(s),
            _ => {
                panic!("Invalid move desc!");
            }
        };
        let to_rank = match chars.next() {
            Some(s) => char_to_rank(s),
            _ => {
                panic!("Invalid move desc!");
            }
        };
        if desc.len() == 5 {
            return (
                from_file + 8 * from_rank,
                to_file + 8 * to_rank,
                Some(char_to_promotion_piecetype(match chars.next() {
                    Some(s) => s,
                    _ => panic!("Invalid move desc!"),
                })),
            );
        }
        (from_file + 8 * from_rank, to_file + 8 * to_rank, None)
    }

    pub fn to_san(self, game_state: &GameState) -> String {
        let mut movelist = MoveList::default();
        let mut agsi = GameStateAttackContainer::from_state(game_state);
        generate_moves(game_state, false, &mut movelist, &agsi);
        let mut res_str = String::new();
        if let GameMoveType::Castle = self.move_type {
            if self.to == 2 || self.to == 58 {
                res_str.push_str("O-O-O");
            } else {
                res_str.push_str("O-O");
            }
        } else {
            res_str.push_str(match self.piece_type {
                PieceType::Pawn => "",
                PieceType::Knight => "N",
                PieceType::Bishop => "B",
                PieceType::Rook => "R",
                PieceType::Queen => "Q",
                PieceType::King => "K",
            });
            //Check for disambiguities
            let mut file_needed = false;
            let mut rank_needed = false;
            for gmv in movelist.move_list.iter() {
                let other_mv = gmv.0;
                if other_mv.piece_type == self.piece_type
                    && other_mv.to == self.to
                    && other_mv.from != self.from
                {
                    if other_mv.from % 8 != self.from % 8 {
                        file_needed = true;
                    } else if other_mv.from / 8 != self.from / 8 {
                        rank_needed = true;
                    } else {
                        file_needed = true;
                        rank_needed = true;
                    }
                }
            }
            if file_needed {
                res_str.push_str(file_to_string((self.from % 8) as usize));
            }
            if rank_needed {
                res_str.push_str(&format!("{}", self.from / 8 + 1))
            };
            if self.is_capture() {
                if self.piece_type == PieceType::Pawn && !file_needed {
                    res_str.push_str(file_to_string((self.from % 8) as usize));
                }
                res_str.push_str("x");
            }
            res_str.push_str(file_to_string((self.to % 8) as usize));
            res_str.push_str(&format!("{}", self.to / 8 + 1));
            if let GameMoveType::Promotion(promo_piece, _) = self.move_type {
                res_str.push_str(&format!(
                    "={}",
                    match promo_piece {
                        PieceType::Queen => "Q",
                        PieceType::Rook => "R",
                        PieceType::Bishop => "B",
                        PieceType::Knight => "N",
                        _ => panic!("Invalid promotion piece"),
                    }
                ));
            }
        }
        let game_state = make_move(game_state, self);
        agsi.write_state(&game_state);
        let agsi = generate_moves(&game_state, false, &mut movelist, &agsi);
        if agsi.stm_incheck && !agsi.stm_haslegalmove {
            res_str.push_str("#");
        } else if agsi.stm_incheck {
            res_str.push_str("+");
        }
        res_str
    }
}

impl Debug for GameMove {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!(
            "{}{}{}{}",
            file_to_string((self.from % 8) as usize),
            self.from / 8 + 1,
            file_to_string((self.to % 8) as usize),
            self.to / 8 + 1
        ));
        if let GameMoveType::Promotion(s, _) = &self.move_type {
            match s {
                PieceType::Queen => res_str.push_str("q"),
                PieceType::Rook => res_str.push_str("r"),
                PieceType::Bishop => res_str.push_str("b"),
                PieceType::Knight => res_str.push_str("n"),
                _ => panic!("Invalid promotion piece type!"),
            }
        };
        write!(formatter, "{}", res_str)
    }
}

fn char_to_promotion_piecetype(c: char) -> PieceType {
    match c {
        'q' | 'Q' => PieceType::Queen,
        'r' | 'R' => PieceType::Rook,
        'b' | 'B' => PieceType::Bishop,
        'n' | 'N' => PieceType::Knight,
        _ => panic!("Invalid promotion piece {}", c),
    }
}

pub fn char_to_rank(c: char) -> usize {
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

pub fn char_to_file(c: char) -> usize {
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
        _ => panic!("invalid file"),
    }
}

#[derive(Clone)]
pub struct GameState {
    // 0 = White
    // 1 = Black
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

    pub checkers: u64,
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
    pub psqt: EvaluationScore,
    pub phase: Phase,
}

impl GameState {
    pub(crate) fn initialize_details(&mut self) {
        self.initialize_hash();
        self.initialize_phase();
        self.initialize_psqt();
        self.initialize_checkers();
    }
    pub fn initialize_hash(&mut self) {
        debug_assert_eq!(self.hash, 0u64);
        self.hash = 0u64;
        if self.color_to_move == BLACK {
            self.hash ^= ZOBRIST_KEYS.side_to_move;
        }
        if self.castle_white_kingside {
            self.hash ^= ZOBRIST_KEYS.castle_w_kingside;
        }
        if self.castle_white_queenside {
            self.hash ^= ZOBRIST_KEYS.castle_w_queenside;
        }
        if self.castle_black_kingside {
            self.hash ^= ZOBRIST_KEYS.castle_b_kingside;
        }
        if self.castle_black_queenside {
            self.hash ^= ZOBRIST_KEYS.castle_b_queenside;
        }
        if self.en_passant != 0u64 {
            let file = self.en_passant.trailing_zeros() as usize % 8;
            self.hash ^= ZOBRIST_KEYS.en_passant[file];
        }

        let mut w_pawns = self.pieces[PAWN][WHITE];
        while w_pawns != 0u64 {
            let idx = w_pawns.trailing_zeros() as usize;
            self.hash ^= ZOBRIST_KEYS.w_pawns[idx];
            w_pawns ^= square(idx);
        }
        let mut w_knights = self.pieces[KNIGHT][WHITE];
        while w_knights != 0u64 {
            let idx = w_knights.trailing_zeros() as usize;
            self.hash ^= ZOBRIST_KEYS.w_knights[idx];
            w_knights ^= square(idx);
        }
        let mut w_bishops = self.pieces[BISHOP][WHITE];
        while w_bishops != 0u64 {
            let idx = w_bishops.trailing_zeros() as usize;
            self.hash ^= ZOBRIST_KEYS.w_bishops[idx];
            w_bishops ^= square(idx);
        }
        let mut w_rooks = self.pieces[ROOK][WHITE];
        while w_rooks != 0u64 {
            let idx = w_rooks.trailing_zeros() as usize;
            self.hash ^= ZOBRIST_KEYS.w_rooks[idx];
            w_rooks ^= square(idx);
        }
        let mut w_queens = self.pieces[QUEEN][WHITE];
        while w_queens != 0u64 {
            let idx = w_queens.trailing_zeros() as usize;
            self.hash ^= ZOBRIST_KEYS.w_queens[idx];
            w_queens ^= square(idx);
        }
        let mut w_king = self.pieces[KING][WHITE];
        while w_king != 0u64 {
            let idx = w_king.trailing_zeros() as usize;
            self.hash ^= ZOBRIST_KEYS.w_king[idx];
            w_king ^= square(idx);
        }
        let mut b_pawns = self.pieces[PAWN][BLACK];
        while b_pawns != 0u64 {
            let idx = b_pawns.trailing_zeros() as usize;
            self.hash ^= ZOBRIST_KEYS.b_pawns[idx];
            b_pawns ^= square(idx);
        }
        let mut b_knights = self.pieces[KNIGHT][BLACK];
        while b_knights != 0u64 {
            let idx = b_knights.trailing_zeros() as usize;
            self.hash ^= ZOBRIST_KEYS.b_knights[idx];
            b_knights ^= square(idx);
        }
        let mut b_bishops = self.pieces[BISHOP][BLACK];
        while b_bishops != 0u64 {
            let idx = b_bishops.trailing_zeros() as usize;
            self.hash ^= ZOBRIST_KEYS.b_bishops[idx];
            b_bishops ^= square(idx);
        }
        let mut b_rooks = self.pieces[ROOK][BLACK];
        while b_rooks != 0u64 {
            let idx = b_rooks.trailing_zeros() as usize;
            self.hash ^= ZOBRIST_KEYS.b_rooks[idx];
            b_rooks ^= square(idx);
        }
        let mut b_queens = self.pieces[QUEEN][BLACK];
        while b_queens != 0u64 {
            let idx = b_queens.trailing_zeros() as usize;
            self.hash ^= ZOBRIST_KEYS.b_queens[idx];
            b_queens ^= square(idx);
        }
        let mut b_king = self.pieces[KING][BLACK];
        while b_king != 0u64 {
            let idx = b_king.trailing_zeros() as usize;
            self.hash ^= ZOBRIST_KEYS.b_king[idx];
            b_king ^= square(idx);
        }
    }
    pub(crate) fn initialize_phase(&mut self) {
        self.phase = Phase::from_pieces(&self.pieces);
    }
    pub(crate) fn initialize_psqt(&mut self) {
        let mut _eval = crate::evaluation::EvaluationResult {
            final_eval: 0,
            #[cfg(feature = "texel-tuning")]
            trace: crate::evaluation::trace::Trace::default(),
        };
        let p_w = crate::evaluation::psqt_evaluation::psqt(true, &self.pieces, &mut _eval);
        let p_b = crate::evaluation::psqt_evaluation::psqt(false, &self.pieces, &mut _eval);
        self.psqt = p_w - p_b;
    }
    pub(crate) fn initialize_checkers(&mut self) {
        self.checkers =
            self.square_attackers(self.king_square(self.color_to_move), self.all_pieces());
    }

    pub fn from_fen(fen: &str) -> GameState {
        let vec: Vec<&str> = fen.trim().split(' ').collect();
        if vec.len() < 4 {
            panic!("Invalid FEN");
        }
        //Parse through FEN
        //Pieces
        let pieces: Vec<&str> = vec[0].split('/').collect();
        if pieces.len() != 8 {
            panic!("Invalid FEN");
        }
        //Iterate over all 8 ranks
        let mut pieces_arr: [[u64; 2]; 6] = [[0u64; 2]; 6];
        for (rank, rank_str) in pieces.iter().enumerate().take(8) {
            let mut file: usize = 0;
            let mut rank_str_idx: usize = 0;
            while file < 8 {
                let idx = (7 - rank) * 8 + file;
                let next_char = rank_str.chars().nth(rank_str_idx);
                rank_str_idx += 1;
                match next_char {
                    Some(x) => match x {
                        'P' => {
                            pieces_arr[PAWN][WHITE] |= square(idx);
                            file += 1;
                        }
                        'p' => {
                            pieces_arr[PAWN][BLACK] |= square(idx);
                            file += 1;
                        }
                        'N' => {
                            pieces_arr[KNIGHT][WHITE] |= square(idx);
                            file += 1;
                        }
                        'n' => {
                            pieces_arr[KNIGHT][BLACK] |= square(idx);
                            file += 1;
                        }
                        'B' => {
                            pieces_arr[BISHOP][WHITE] |= square(idx);
                            file += 1;
                        }
                        'b' => {
                            pieces_arr[BISHOP][BLACK] |= square(idx);
                            file += 1;
                        }
                        'R' => {
                            pieces_arr[ROOK][WHITE] |= square(idx);
                            file += 1;
                        }
                        'r' => {
                            pieces_arr[ROOK][BLACK] |= square(idx);
                            file += 1;
                        }
                        'Q' => {
                            pieces_arr[QUEEN][WHITE] |= square(idx);
                            file += 1;
                        }
                        'q' => {
                            pieces_arr[QUEEN][BLACK] |= square(idx);
                            file += 1;
                        }
                        'K' => {
                            pieces_arr[KING][WHITE] |= square(idx);
                            file += 1;
                        }
                        'k' => {
                            pieces_arr[KING][BLACK] |= square(idx);
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
                    },
                    None => panic!("Invalid FEN"),
                }
            }
        }

        //Side to move
        let color_to_move = match vec[1] {
            "w" => 0,
            "b" => 1,
            _ => panic!("Invalid FEN!"),
        };

        //Castling-Abilities
        let castle_white_kingside = vec[2].contains('K');
        let castle_white_queenside = vec[2].contains('Q');
        let castle_black_kingside = vec[2].contains('k');
        let castle_black_queenside = vec[2].contains('q');

        //En passant target square
        let en_passant: u64 = if vec[3] != "-" {
            let mut idx: usize = 0usize;
            let file = vec[3].chars().next();
            let rank = vec[3].chars().nth(1);
            match file {
                Some(x) => match x {
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
                },
                None => {
                    panic!("Invalid FEN!");
                }
            }
            match rank {
                Some(x) => match x {
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
                },
                None => {
                    panic!("Invalid FEN!");
                }
            }
            square(idx)
        } else {
            0u64
        };
        let (half_moves, full_moves) = if vec.len() > 4 {
            (
                vec[4].parse().expect("unable to parse half moves"),
                vec[5].parse().expect("unable to parse full moves"),
            )
        } else {
            (0, 1)
        };

        let mut res = GameState {
            color_to_move,
            pieces: pieces_arr,
            castle_white_kingside,
            castle_white_queenside,
            castle_black_kingside,
            castle_black_queenside,
            half_moves,
            full_moves,
            en_passant,
            checkers: 0u64,
            hash: 0u64,
            psqt: EvaluationScore(0, 0),
            phase: Phase {
                phase: 0.,
                material_score: 0,
            },
        };
        res.initialize_details();
        res
    }

    pub fn get_piece_on(&self, shift: i32) -> &str {
        if (self.pieces[PAWN][WHITE] >> shift) & 1u64 != 0u64 {
            "P"
        } else if (self.pieces[KNIGHT][WHITE] >> shift) & 1u64 != 0u64 {
            "N"
        } else if (self.pieces[BISHOP][WHITE] >> shift) & 1u64 != 0u64 {
            "B"
        } else if (self.pieces[ROOK][WHITE] >> shift) & 1u64 != 0u64 {
            "R"
        } else if (self.pieces[QUEEN][WHITE] >> shift) & 1u64 != 0u64 {
            "Q"
        } else if (self.pieces[KING][WHITE] >> shift) & 1u64 != 0u64 {
            "K"
        } else if (self.pieces[PAWN][BLACK] >> shift) & 1u64 != 0u64 {
            "p"
        } else if (self.pieces[KNIGHT][BLACK] >> shift) & 1u64 != 0u64 {
            "n"
        } else if (self.pieces[BISHOP][BLACK] >> shift) & 1u64 != 0u64 {
            "b"
        } else if (self.pieces[ROOK][BLACK] >> shift) & 1u64 != 0u64 {
            "r"
        } else if (self.pieces[QUEEN][BLACK] >> shift) & 1u64 != 0u64 {
            "q"
        } else if (self.pieces[KING][BLACK] >> shift) & 1u64 != 0u64 {
            "k"
        } else {
            " "
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
                let piece_on = self.get_piece_on(shift);

                if piece_on != " " {
                    if files_skipped != 1 {
                        res_str.push_str(&format!("{}", files_skipped - 1));
                    }
                    files_skipped = 0;
                    res_str.push_str(piece_on);
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
        if !(self.castle_white_kingside
            | self.castle_white_queenside
            | self.castle_black_kingside
            | self.castle_black_queenside)
        {
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
        let pieces = [
            [0xff00u64, 0x00ff_0000_0000_0000u64],
            [0x42u64, 0x4200_0000_0000_0000u64],
            [0x24u64, 0x2400_0000_0000_0000u64],
            [0x81u64, 0x8100_0000_0000_0000u64],
            [0x8u64, 0x0800_0000_0000_0000u64],
            [0x10u64, 0x1000_0000_0000_0000u64],
        ];
        let mut res = GameState {
            color_to_move,
            pieces,
            castle_white_kingside: true,
            castle_white_queenside: true,
            castle_black_kingside: true,
            castle_black_queenside: true,
            en_passant: 0u64,
            half_moves: 0usize,
            full_moves: 1usize,
            checkers: 0u64,
            hash: 0u64,
            psqt: EvaluationScore(0, 0),
            phase: Phase {
                phase: 0.,
                material_score: 0,
            },
        };
        res.initialize_details();
        res
    }

    pub fn is_valid_tt_move(
        &self,
        mv: GameMove,
        attack_container: &GameStateAttackContainer,
    ) -> bool {
        //println!("{}",self.to_fen());
        //println!("{:?}", mv);
        if self.pieces[mv.piece_type.to_index()][self.color_to_move] & square(mv.from as usize)
            == 0u64
        {
            return false;
        }
        if mv.piece_type == PieceType::Pawn
            && if self.color_to_move == WHITE {
                mv.to / 8
            } else {
                7 - mv.to / 8
            } == 7
        {
            if let GameMoveType::Promotion(_, _) = mv.move_type {
            } else {
                return false;
            }
        } else if let GameMoveType::Promotion(_, _) = mv.move_type {
            if mv.piece_type != PieceType::Pawn
                || if self.color_to_move == WHITE {
                    mv.to / 8
                } else {
                    7 - mv.to / 8
                } != 7
            {
                return false;
            }
        }

        if mv.move_type == GameMoveType::EnPassant {
            if (self.en_passant & square(mv.to as usize)) == 0u64 {
                return false;
            }
        } else if mv.move_type == GameMoveType::Castle {
            if mv.piece_type != PieceType::King {
                return false;
            }
            let all_piece = self.all_pieces();
            let blocked = all_piece | attack_container.attacks_sum[1 - self.color_to_move];
            if mv.to != 6 && mv.to != 2 && mv.to != 62 && mv.to != 58
                || attack_container.attacks_sum[1 - self.color_to_move] & square(mv.from as usize)
                    != 0u64
            {
                return false;
            }
            let is_invalid_wk = || {
                mv.to == 6
                    && (!self.castle_white_kingside || blocked & (1u64 << 5 | 1u64 << 6) != 0u64)
            };
            let is_invalid_wq = || {
                mv.to == 2
                    && (!self.castle_white_queenside
                        || blocked & (1u64 << 2 | 1u64 << 3) != 0u64
                        || all_piece & (1u64 << 1) != 0u64)
            };
            let is_invalid_bk = || {
                mv.to == 62
                    && (!self.castle_black_kingside || blocked & (1u64 << 61 | 1u64 << 62) != 0u64)
            };
            let is_invalid_bq = || {
                mv.to == 58
                    && (!self.castle_black_queenside
                        || blocked & (1u64 << 58 | 1u64 << 59) != 0u64
                        || all_piece & (1u64 << 57) != 0u64)
            };
            if is_invalid_wk() || is_invalid_wq() || is_invalid_bk() || is_invalid_bq() {
                return false;
            }
        } else {
            let captured_piece = match mv.move_type {
                GameMoveType::Capture(p) => Some(p),
                GameMoveType::Promotion(_, p) => p,
                _ => None,
            };
            if captured_piece.is_none() {
                if self.all_pieces() & (square(mv.to as usize)) != 0u64 {
                    return false;
                }
            } else if self.pieces[captured_piece.unwrap().to_index()][1 - self.color_to_move]
                & (square(mv.to as usize))
                == 0u64
            {
                return false;
            }
        }
        let mut all_pieces = self.all_pieces();
        match mv.piece_type {
            PieceType::King => {
                if square(mv.to as usize) & (attack_container.attacks_sum[1 - self.color_to_move])
                    != 0u64
                    || mv.move_type != GameMoveType::Castle
                        && (square(mv.to as usize)) & (KING_ATTACKS[mv.from as usize]) == 0u64
                {
                    return false;
                }
            }
            PieceType::Bishop => {
                if square(mv.to as usize) & (bishop_attacks(mv.from as usize, all_pieces)) == 0u64 {
                    return false;
                }
            }
            PieceType::Rook => {
                if square(mv.to as usize) & (rook_attacks(mv.from as usize, all_pieces)) == 0u64 {
                    return false;
                }
            }
            PieceType::Queen => {
                if square(mv.to as usize)
                    & (bishop_attacks(mv.from as usize, all_pieces)
                        | rook_attacks(mv.from as usize, all_pieces))
                    == 0u64
                {
                    return false;
                }
            }
            PieceType::Knight => {
                if square(mv.to as usize) & (KNIGHT_ATTACKS[mv.from as usize]) == 0u64 {
                    return false;
                }
            }
            PieceType::Pawn => {
                if mv.is_capture() {
                    if (pawn_west_targets(self.color_to_move, square(mv.from as usize))
                        | pawn_east_targets(self.color_to_move, square(mv.from as usize)))
                        & square(mv.to as usize)
                        == 0u64
                    {
                        return false;
                    }
                } else if (single_push_pawn_targets(
                    self.color_to_move,
                    square(mv.from as usize),
                    !all_pieces,
                ) | double_push_pawn_targets(
                    self.color_to_move,
                    square(mv.from as usize),
                    !all_pieces,
                )) & square(mv.to as usize)
                    == 0u64
                {
                    return false;
                }
            }
        }
        if mv.move_type != GameMoveType::Castle {
            all_pieces ^= square(mv.from as usize);
            all_pieces |= square(mv.to as usize);
            let cap_piece = if mv.move_type == GameMoveType::EnPassant {
                1u64 << (if self.color_to_move == WHITE {
                    mv.to - 8
                } else {
                    mv.to + 8
                })
            } else {
                1u64 << mv.to
            };
            let king_square = if mv.piece_type == PieceType::King {
                mv.to as usize
            } else {
                self.king_square(self.color_to_move)
            };
            if bishop_attacks(king_square, all_pieces)
                & (self.pieces[BISHOP][1 - self.color_to_move]
                    | self.pieces[QUEEN][1 - self.color_to_move])
                & !cap_piece
                != 0u64
            {
                return false;
            }
            if rook_attacks(king_square, all_pieces)
                & (self.pieces[ROOK][1 - self.color_to_move]
                    | self.pieces[QUEEN][1 - self.color_to_move])
                & !cap_piece
                != 0u64
            {
                return false;
            }
            if KNIGHT_ATTACKS[king_square]
                & (self.pieces[KNIGHT][1 - self.color_to_move])
                & !cap_piece
                != 0u64
            {
                return false;
            }
            if self.color_to_move == WHITE {
                if (w_pawn_east_targets(square(king_square))
                    | w_pawn_west_targets(square(king_square)))
                    & (self.pieces[PAWN][1 - self.color_to_move])
                    & !cap_piece
                    != 0u64
                {
                    return false;
                }
            } else if (b_pawn_east_targets(square(king_square))
                | b_pawn_west_targets(square(king_square)))
                & (self.pieces[PAWN][1 - self.color_to_move])
                & !cap_piece
                != 0u64
            {
                return false;
            }
        }
        true
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
                if (self.pieces[PAWN][WHITE] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("P");
                } else if (self.pieces[PAWN][BLACK] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("p");
                } else if (self.pieces[KNIGHT][WHITE] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("N");
                } else if (self.pieces[KNIGHT][BLACK] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("n");
                } else if (self.pieces[BISHOP][WHITE] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("B");
                } else if (self.pieces[BISHOP][BLACK] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("b");
                } else if (self.pieces[ROOK][WHITE] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("R");
                } else if (self.pieces[ROOK][BLACK] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("r");
                } else if (self.pieces[QUEEN][WHITE] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("Q");
                } else if (self.pieces[QUEEN][BLACK] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("q");
                } else if (self.pieces[KING][WHITE] >> idx) & 1u64 != 0u64 {
                    res_str.push_str("K");
                } else if (self.pieces[KING][BLACK] >> idx) & 1u64 != 0u64 {
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
        res_str.push_str(&format!(
            "White Queenside: {}\n",
            self.castle_white_queenside
        ));
        res_str.push_str(&format!("Black Kingside: {}\n", self.castle_black_kingside));
        res_str.push_str(&format!(
            "Black Queenside: {}\n",
            self.castle_black_queenside
        ));
        res_str.push_str(&format!("En Passant Possible: {:x}\n", self.en_passant));
        res_str.push_str(&format!("Half-Counter: {}\n", self.half_moves));
        res_str.push_str(&format!("Full-Counter: {}\n", self.full_moves));
        res_str.push_str(&format!("Side to Move: {}\n", self.color_to_move));
        res_str.push_str(&format!("Checkers: 0x{:x}u64\n", self.checkers));
        res_str.push_str(&format!("Hash: {}\n", self.hash));
        res_str.push_str(&format!("FEN: {}\n", self.to_fen()));
        write!(formatter, "{}", res_str)
    }
}

impl Debug for GameState {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!("Color: {}\n", self.color_to_move));
        res_str.push_str(&format!(
            "WhitePawns: 0x{:x}u64\n",
            self.pieces[PAWN][WHITE]
        ));
        res_str.push_str(&format!(
            "WhiteKnights: 0x{:x}u64\n",
            self.pieces[KNIGHT][WHITE]
        ));
        res_str.push_str(&format!(
            "WhiteBishops: 0x{:x}u64\n",
            self.pieces[BISHOP][WHITE]
        ));
        res_str.push_str(&format!(
            "WhiteRooks: 0x{:x}u64\n",
            self.pieces[ROOK][WHITE]
        ));
        res_str.push_str(&format!(
            "WhiteQueens: 0x{:x}u64\n",
            self.pieces[QUEEN][WHITE]
        ));
        res_str.push_str(&format!("WhiteKing: 0x{:x}u64\n", self.pieces[KING][WHITE]));
        res_str.push_str(&format!(
            "BlackPawns: 0x{:x}u64\n",
            self.pieces[PAWN][BLACK]
        ));
        res_str.push_str(&format!(
            "BlackKnights: 0x{:x}u64\n",
            self.pieces[KNIGHT][BLACK]
        ));
        res_str.push_str(&format!(
            "BlackBishops: 0x{:x}u64\n",
            self.pieces[BISHOP][BLACK]
        ));
        res_str.push_str(&format!(
            "BlackRooks: 0x{:x}u64\n",
            self.pieces[ROOK][BLACK]
        ));
        res_str.push_str(&format!(
            "BlackQueens: 0x{:x}u64\n",
            self.pieces[QUEEN][BLACK]
        ));
        res_str.push_str(&format!("BlackKing: 0x{:x}u64\n", self.pieces[KING][BLACK]));
        res_str.push_str(&format!("CWK: {}\n", self.castle_white_kingside));
        res_str.push_str(&format!("CWQ: {}\n", self.castle_white_queenside));
        res_str.push_str(&format!("CBK: {}\n", self.castle_black_kingside));
        res_str.push_str(&format!("CBQ: {}\n", self.castle_black_queenside));
        res_str.push_str(&format!("En-Passant: 0x{:x}u64\n", self.en_passant));
        res_str.push_str(&format!("half_moves: {}\n", self.half_moves));
        res_str.push_str(&format!("full_moves: {}\n", self.full_moves));
        res_str.push_str(&format!("Side to Move: {}\n", self.color_to_move));
        res_str.push_str(&format!("Hash: {}\n", self.hash));
        res_str.push_str(&format!("Checkers: 0x{:x}u64\n", self.checkers));
        res_str.push_str(&format!("Phase: {}\n", self.phase.phase));
        res_str.push_str(&format!("PSQT: {}\n", self.psqt));
        res_str.push_str(&format!("FEN: {}\n", self.to_fen()));
        write!(formatter, "{}", res_str)
    }
}
