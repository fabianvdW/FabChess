use crate::bitboards::bitboards::constants::{square, KING_ATTACKS, KNIGHT_ATTACKS};
use crate::board_representation::zobrist_hashing::ZOBRIST_KEYS;
use crate::evaluation::params::*;
use crate::evaluation::phase::Phase;
use crate::evaluation::EvaluationScore;
use crate::move_generation::makemove::make_move;
use crate::move_generation::movegen::{
    bishop_attacks, double_push_pawn_targets, pawn_east_targets, pawn_west_targets, rook_attacks,
    single_push_pawn_targets,
};
use crate::move_generation::movegen2;
use crate::move_generation::movegen2::generate_legal_moves;
use std::fmt::{Debug, Display, Formatter, Result};

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
    King = 0,
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
}
impl PieceType {
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
    pub fn to_zobrist_key(self, side: usize, sq: usize) -> u64 {
        ZOBRIST_KEYS.pieces[side][self as usize][sq]
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

#[derive(Clone, Copy, PartialEq)]
pub struct GameMove {
    pub from: u8,
    pub to: u8,
    pub move_type: GameMoveType,
    pub piece_type: PieceType,
}

impl GameMove {
    pub fn new(from: usize, to: usize, move_type: GameMoveType, piece_type: PieceType) -> Self {
        GameMove {
            from: from as u8,
            to: to as u8,
            move_type,
            piece_type,
        }
    }
    #[inline(always)]
    pub fn is_capture(self) -> bool {
        match self.move_type {
            GameMoveType::Capture(_) => true,
            GameMoveType::Promotion(_, Some(_)) => true,
            GameMoveType::EnPassant => true,
            _ => false,
        }
    }
    #[inline(always)]
    pub fn get_maybe_captured_piece(self) -> Option<PieceType> {
        match self.move_type {
            GameMoveType::Capture(c) => Some(c),
            GameMoveType::EnPassant => Some(PieceType::Pawn),
            GameMoveType::Promotion(_, Some(c)) => Some(c),
            _ => None,
        }
    }
    #[inline(always)]
    pub fn get_captured_piece(self) -> PieceType {
        debug_assert!(self.is_capture());
        match self.move_type {
            GameMoveType::Capture(p) => p,
            GameMoveType::Promotion(_, Some(p)) => p,
            GameMoveType::EnPassant => PieceType::Pawn,
            _ => panic!("Captured piece type  called on a non capture"),
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
        let movelist = movegen2::generate_legal_moves(game_state);
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
                res_str.push_str(file_to_string((self.from % 8) as usize).encode_utf8(&mut [0; 4]));
            }
            if rank_needed {
                res_str.push_str(&format!("{}", self.from / 8 + 1))
            };
            if self.is_capture() {
                if self.piece_type == PieceType::Pawn && !file_needed {
                    res_str.push_str(
                        file_to_string((self.from % 8) as usize).encode_utf8(&mut [0; 4]),
                    );
                }
                res_str.push_str("x");
            }
            res_str.push_str(file_to_string((self.to % 8) as usize).encode_utf8(&mut [0; 4]));
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
        let mut next_state = game_state.clone();
        make_move(&mut next_state, self);
        let mvs = generate_legal_moves(&next_state);
        if mvs.move_list.is_empty() && next_state.in_check() {
            res_str.push_str("#");
        } else if next_state.in_check() {
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
    assert!(c as usize >= '1' as usize && c as usize - '1' as usize <= 7);
    c as usize - '1' as usize
}

pub fn char_to_file(c: char) -> usize {
    assert!(c as usize >= 'a' as usize && c as usize - 'a' as usize <= 7);
    c as usize - 'a' as usize
}

fn file_to_string(file: usize) -> char {
    (file as u8 + b'a') as char
}

#[derive(Clone, PartialEq)]
pub struct Irreversible {
    pub checkers: u64,
    pub castle_white_kingside: bool,
    pub castle_white_queenside: bool,
    pub castle_black_kingside: bool,
    pub castle_black_queenside: bool,
    pub en_passant: u64,
    pub half_moves: usize,
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
    // 2 -> Bisho
    // 3 -> Rooks
    // 4 -> Queens
    // 5 -> King
    pieces_bb: [u64; 6],
    color_bb: [u64; 2],
    piece_square_table: [Option<(PieceType, bool)>; 64],

    pub irreversible: Irreversible,
    pub full_moves: usize,
    pub hash: u64,
    pub psqt: EvaluationScore,
    pub phase: Phase,
}
//Querying functions
impl GameState {
    pub fn unset_piece(&mut self, piece: PieceType, sq: usize, color: usize) {
        debug_assert!(self.pieces_bb[piece as usize] & square(sq) > 0);
        debug_assert!(self.color_bb[color as usize] & square(sq) > 0);
        debug_assert!(self.piece_square_table[sq].is_some());
        self.pieces_bb[piece as usize] ^= square(sq);
        self.color_bb[color] ^= square(sq);
        self.piece_square_table[sq] = None;
    }

    pub fn set_piece(&mut self, piece: PieceType, sq: usize, color: usize) {
        debug_assert!(self.pieces_bb[piece as usize] & square(sq) == 0);
        debug_assert!(self.color_bb[color as usize] & square(sq) == 0);
        debug_assert!(self.piece_square_table[sq].is_none());
        self.pieces_bb[piece as usize] ^= square(sq);
        self.color_bb[color] ^= square(sq);
        self.piece_square_table[sq] = Some((piece, color == WHITE));
    }

    pub fn relative_rank(sq: usize, white: bool) -> usize {
        if white {
            sq / 8
        } else {
            7 - sq / 8
        }
    }

    pub fn all_pieces_bb(&self) -> u64 {
        self.color_bb(WHITE) | self.color_bb(BLACK)
    }

    pub fn empty_bb(&self) -> u64 {
        !self.all_pieces_bb()
    }

    pub fn color_bb(&self, side: usize) -> u64 {
        self.color_bb[side]
    }

    pub fn piece_bb(&self, pt: PieceType) -> u64 {
        self.pieces_bb[pt as usize]
    }

    pub fn get_piece(&self, pt: PieceType, side: usize) -> u64 {
        self.piece_bb(pt) & self.color_bb(side)
    }

    pub fn has_non_pawns(&self, side: usize) -> bool {
        self.all_pieces_bb()
            & !self.piece_bb(PieceType::Pawn)
            & !self.piece_bb(PieceType::King)
            & self.color_bb(side)
            > 0
    }

    pub fn king_square(&self, side: usize) -> usize {
        self.get_piece(PieceType::King, side).trailing_zeros() as usize
    }

    pub fn in_check(&self) -> bool {
        self.irreversible.checkers > 0
    }

    //Returns: Some if any piece type on the given square. Else none
    //Returns: 0: PieceType 1: bool color: True => White, False => Black
    pub fn piecetype_on(&self, sq: usize) -> Option<(PieceType, bool)> {
        self.piece_square_table[sq]
    }

    pub fn move_type_to(&self, to: usize) -> GameMoveType {
        if let Some((pt, color)) = self.piecetype_on(to) {
            debug_assert!(pt != PieceType::King);
            assert!(if self.color_to_move == WHITE {
                !color
            } else {
                color
            });
            GameMoveType::Capture(pt)
        } else {
            GameMoveType::Quiet
        }
    }
}

//Integrity and after initialization functions
impl GameState {
    pub(crate) fn integrity_bb(&self) -> bool {
        if self.color_bb(WHITE) & self.color_bb(BLACK) > 0 {
            return false;
        }
        for sq in 0..64 {
            let sum = (self.piece_bb(PieceType::Pawn) & square(sq)).count_ones()
                | (self.piece_bb(PieceType::Knight) & square(sq)).count_ones()
                | (self.piece_bb(PieceType::Bishop) & square(sq)).count_ones()
                | (self.piece_bb(PieceType::Rook) & square(sq)).count_ones()
                | (self.piece_bb(PieceType::Queen) & square(sq)).count_ones()
                | (self.piece_bb(PieceType::King) & square(sq)).count_ones();
            if sum >= 2
                || sum == 0 && self.all_pieces_bb() & square(sq) > 0
                || sum == 1 && self.all_pieces_bb() & square(sq) == 0
            {
                return false;
            }
        }
        true
    }
    pub(crate) fn integrity_piece_square_table(&self) -> bool {
        for sq in 0..64 {
            if let Some((pt, color)) = self.piecetype_on(sq) {
                if self.get_piece(pt, if color { WHITE } else { BLACK }) & square(sq) == 0 {
                    return false;
                }
            } else if self.all_pieces_bb() & square(sq) > 0 {
                return false;
            }
        }
        true
    }
    pub(crate) fn check_integrity(&self) -> bool {
        let mut other = self.clone();
        other.initialize_details();
        if self.hash != other.hash {
            return false;
        }
        if self.psqt != other.psqt {
            return false;
        }
        if self.phase != other.phase {
            return false;
        }
        if self.irreversible.checkers != other.irreversible.checkers {
            return false;
        }
        if self.all_pieces_bb()
            != (self.piece_bb(PieceType::King)
                | self.piece_bb(PieceType::Pawn)
                | self.piece_bb(PieceType::Knight)
                | self.piece_bb(PieceType::Bishop)
                | self.piece_bb(PieceType::Rook)
                | self.piece_bb(PieceType::Queen))
        {
            return false;
        }
        if !self.integrity_piece_square_table() {
            return false;
        }
        if !self.integrity_bb() {
            return false;
        }
        true
    }

    pub(crate) fn initialize_details(&mut self) {
        self.initialize_hash();
        self.initialize_phase();
        self.initialize_psqt();
        self.initialize_checkers();
    }

    pub fn initialize_hash(&mut self) {
        self.hash = 0u64;
        if self.color_to_move == BLACK {
            self.hash ^= ZOBRIST_KEYS.side_to_move;
        }
        if self.irreversible.castle_white_kingside {
            self.hash ^= ZOBRIST_KEYS.castle_w_kingside;
        }
        if self.irreversible.castle_white_queenside {
            self.hash ^= ZOBRIST_KEYS.castle_w_queenside;
        }
        if self.irreversible.castle_black_kingside {
            self.hash ^= ZOBRIST_KEYS.castle_b_kingside;
        }
        if self.irreversible.castle_black_queenside {
            self.hash ^= ZOBRIST_KEYS.castle_b_queenside;
        }
        if self.irreversible.en_passant != 0u64 {
            let file = self.irreversible.en_passant.trailing_zeros() as usize % 8;
            self.hash ^= ZOBRIST_KEYS.en_passant[file];
        }
        for side in 0..2 {
            for pt in [
                PieceType::King,
                PieceType::Pawn,
                PieceType::Knight,
                PieceType::Rook,
                PieceType::Bishop,
                PieceType::Queen,
            ]
            .iter()
            {
                let mut piece = self.get_piece(*pt, side);
                while piece > 0 {
                    let idx = piece.trailing_zeros() as usize;
                    self.hash ^= ZOBRIST_KEYS.pieces[side][*pt as usize][idx];
                    piece ^= square(idx)
                }
            }
        }
    }

    pub(crate) fn initialize_phase(&mut self) {
        self.phase = Phase::from_state(self);
    }

    pub(crate) fn initialize_psqt(&mut self) {
        let mut _eval = crate::evaluation::EvaluationResult {
            final_eval: 0,
            #[cfg(feature = "texel-tuning")]
            trace: crate::evaluation::trace::Trace::default(),
        };
        let p_w = crate::evaluation::psqt_evaluation::psqt(true, self, &mut _eval);
        let p_b = crate::evaluation::psqt_evaluation::psqt(false, self, &mut _eval);
        self.psqt = p_w - p_b;
    }

    pub(crate) fn initialize_checkers(&mut self) {
        self.irreversible.checkers =
            self.square_attackers(self.king_square(self.color_to_move), self.all_pieces_bb());
    }
}
//Constructor and util functions
impl GameState {
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
        let mut pieces_bb = [0u64; 6];
        let mut color_bb = [0u64; 2];
        let mut piece_square_table = [None; 64];

        for (rank, rank_str) in pieces.iter().enumerate().take(8) {
            let mut file: usize = 0;
            let mut rank_str_idx: usize = 0;
            while file < 8 {
                let idx = (7 - rank) * 8 + file;
                let next_char = rank_str.chars().nth(rank_str_idx);
                rank_str_idx += 1;
                match next_char {
                    Some(x) => {
                        if ['p', 'n', 'b', 'r', 'q', 'k'].contains(&x.to_ascii_lowercase()) {
                            let side = if x.is_uppercase() { WHITE } else { BLACK };
                            let piece_type = match x.to_lowercase().next().unwrap() {
                                'p' => PieceType::Pawn,
                                'n' => PieceType::Knight,
                                'b' => PieceType::Bishop,
                                'r' => PieceType::Rook,
                                'q' => PieceType::Queen,
                                'k' => PieceType::King,
                                _ => panic!("Invalid fen"),
                            };
                            color_bb[side] |= square(idx);
                            pieces_bb[piece_type as usize] |= square(idx);
                            piece_square_table[idx] = Some((piece_type, side == WHITE));
                            file += 1;
                        } else {
                            file += match x {
                                '1' => 1,
                                '2' => 2,
                                '3' => 3,
                                '4' => 4,
                                '5' => 5,
                                '6' => 6,
                                '7' => 7,
                                '8' => 8,
                                _ => {
                                    panic!("Invalid FEN");
                                }
                            };
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
            pieces_bb,
            color_bb,
            piece_square_table,
            irreversible: Irreversible {
                castle_white_kingside,
                castle_white_queenside,
                castle_black_kingside,
                castle_black_queenside,
                half_moves,
                en_passant,
                checkers: 0u64,
            },
            full_moves,
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

    pub fn get_piece_on(&self, sq: usize) -> String {
        if let Some((pt, color)) = self.piecetype_on(sq) {
            let res = match pt {
                PieceType::Pawn => "p",
                PieceType::Knight => "n",
                PieceType::Bishop => "b",
                PieceType::Rook => "r",
                PieceType::Queen => "q",
                PieceType::King => "k",
            };
            if color {
                res.to_uppercase()
            } else {
                res.to_owned()
            }
        } else {
            " ".to_owned()
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
                    res_str.push_str(&piece_on);
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
        if !(self.irreversible.castle_white_kingside
            | self.irreversible.castle_white_queenside
            | self.irreversible.castle_black_kingside
            | self.irreversible.castle_black_queenside)
        {
            res_str.push_str("-");
        } else {
            if self.irreversible.castle_white_kingside {
                res_str.push_str("K");
            }
            if self.irreversible.castle_white_queenside {
                res_str.push_str("Q");
            }
            if self.irreversible.castle_black_kingside {
                res_str.push_str("k");
            }
            if self.irreversible.castle_black_queenside {
                res_str.push_str("q");
            }
        }
        res_str.push_str(" ");

        if self.irreversible.en_passant == 0u64 {
            res_str.push_str("-");
        } else {
            let idx = self.irreversible.en_passant.trailing_zeros() as usize;
            let rank = idx / 8;
            let file = idx % 8;
            res_str.push_str(&format!("{}{}", file_to_string(file), rank + 1));
        }
        res_str.push_str(" ");
        res_str.push_str(&format!("{} ", self.irreversible.half_moves));
        res_str.push_str(&format!("{}", self.full_moves));
        res_str
    }

    pub fn standard() -> GameState {
        let color_to_move = 0usize;
        let mut pieces_bb = [0u64; 6];
        let mut color_bb = [0u64; 2];
        let mut piece_square_table = [None; 64];
        let info = [
            (PieceType::Pawn, WHITE, 0xff00u64),
            (PieceType::Pawn, BLACK, 0x00ff_0000_0000_0000u64),
            (PieceType::Knight, WHITE, 0x42u64),
            (PieceType::Knight, BLACK, 0x4200_0000_0000_0000u64),
            (PieceType::Bishop, WHITE, 0x24u64),
            (PieceType::Bishop, BLACK, 0x2400_0000_0000_0000u64),
            (PieceType::Rook, WHITE, 0x81u64),
            (PieceType::Rook, BLACK, 0x8100_0000_0000_0000u64),
            (PieceType::Queen, WHITE, 0x8u64),
            (PieceType::Queen, BLACK, 0x0800_0000_0000_0000u64),
            (PieceType::King, WHITE, 0x10u64),
            (PieceType::King, BLACK, 0x1000_0000_0000_0000u64),
        ];
        for (pt, side, mut bb) in info.iter() {
            pieces_bb[*pt as usize] |= bb;
            color_bb[*side] |= bb;
            while bb > 0 {
                let idx = bb.trailing_zeros() as usize;
                piece_square_table[idx] = Some((*pt, *side == WHITE));
                bb ^= square(idx);
            }
        }
        let mut res = GameState {
            color_to_move,
            pieces_bb,
            color_bb,
            piece_square_table,
            irreversible: Irreversible {
                castle_white_kingside: true,
                castle_white_queenside: true,
                castle_black_kingside: true,
                castle_black_queenside: true,
                en_passant: 0u64,
                half_moves: 0,
                checkers: 0u64,
            },
            full_moves: 1usize,
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

    pub fn is_valid_tt_move(&self, mv: GameMove) -> bool {
        //println!("{}",self.to_fen());
        //println!("{:?}", mv);
        if self.get_piece(mv.piece_type, self.color_to_move) & square(mv.from as usize) == 0 {
            return false;
        }
        if mv.piece_type == PieceType::Pawn
            && GameState::relative_rank(mv.to as usize, self.color_to_move == WHITE) == 7
        {
            if let GameMoveType::Promotion(_, _) = mv.move_type {
            } else {
                return false;
            }
        } else if let GameMoveType::Promotion(_, _) = mv.move_type {
            if mv.piece_type != PieceType::Pawn
                || GameState::relative_rank(mv.to as usize, self.color_to_move == WHITE) != 7
            {
                return false;
            }
        }

        if mv.move_type == GameMoveType::EnPassant {
            if (self.irreversible.en_passant & square(mv.to as usize)) == 0u64 {
                return false;
            }
        } else if mv.move_type == GameMoveType::Castle {
            if mv.piece_type != PieceType::King {
                return false;
            }

            if mv.to != 6 && mv.to != 2 && mv.to != 62 && mv.to != 58 || self.in_check() {
                return false;
            }
            let all_piece = self.all_pieces_bb();
            let is_invalid_wk = || {
                mv.to == 6
                    && (!self.irreversible.castle_white_kingside
                        || all_piece & (square(5) | square(6)) != 0u64
                        || self.square_attacked(5, all_piece, 0u64)
                        || self.square_attacked(6, all_piece, 0u64))
            };
            let is_invalid_wq = || {
                mv.to == 2
                    && (!self.irreversible.castle_white_queenside
                        || all_piece & (square(1) | square(2) | square(3)) != 0u64
                        || self.square_attacked(2, all_piece, 0u64)
                        || self.square_attacked(3, all_piece, 0u64))
            };
            let is_invalid_bk = || {
                mv.to == 62
                    && (!self.irreversible.castle_black_kingside
                        || all_piece & (square(61) | square(62)) != 0u64
                        || self.square_attacked(61, all_piece, 0u64)
                        || self.square_attacked(62, all_piece, 0u64))
            };
            let is_invalid_bq = || {
                mv.to == 58
                    && (!self.irreversible.castle_black_queenside
                        || all_piece & (square(57) | square(58) | square(59)) != 0u64
                        || self.square_attacked(58, all_piece, 0u64)
                        || self.square_attacked(59, all_piece, 0u64))
            };
            if is_invalid_wk() || is_invalid_wq() || is_invalid_bk() || is_invalid_bq() {
                return false;
            }
        } else {
            let captured_piece = mv.get_maybe_captured_piece();
            if captured_piece.is_none() {
                if self.all_pieces_bb() & (square(mv.to as usize)) != 0u64 {
                    return false;
                }
            } else if self.get_piece(captured_piece.unwrap(), 1 - self.color_to_move)
                & (square(mv.to as usize))
                == 0u64
            {
                return false;
            }
        }
        let all_pieces = self.all_pieces_bb();
        match mv.piece_type {
            PieceType::King => {
                if self.square_attacked(mv.to as usize, all_pieces ^ square(mv.from as usize), 0u64)
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
        self.is_valid_move(mv)
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
                res_str.push_str(&self.get_piece_on(idx));
                res_str.push_str(" | ");
            }
            res_str.push_str("\n+---+---+---+---+---+---+---+---+\n");
        }
        res_str.push_str("Castle Rights: \n");
        res_str.push_str(&format!(
            "White Kingside: {}\n",
            self.irreversible.castle_white_kingside
        ));
        res_str.push_str(&format!(
            "White Queenside: {}\n",
            self.irreversible.castle_white_queenside
        ));
        res_str.push_str(&format!(
            "Black Kingside: {}\n",
            self.irreversible.castle_black_kingside
        ));
        res_str.push_str(&format!(
            "Black Queenside: {}\n",
            self.irreversible.castle_black_queenside
        ));
        res_str.push_str(&format!(
            "En Passant Possible: {:x}\n",
            self.irreversible.en_passant
        ));
        res_str.push_str(&format!("Half-Counter: {}\n", self.irreversible.half_moves));
        res_str.push_str(&format!("Full-Counter: {}\n", self.full_moves));
        res_str.push_str(&format!("Side to Move: {}\n", self.color_to_move));
        res_str.push_str(&format!(
            "Checkers: 0x{:x}u64\n",
            self.irreversible.checkers
        ));
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
            self.get_piece(PieceType::Pawn, WHITE)
        ));
        res_str.push_str(&format!(
            "WhiteKnights: 0x{:x}u64\n",
            self.get_piece(PieceType::Knight, WHITE)
        ));
        res_str.push_str(&format!(
            "WhiteBishops: 0x{:x}u64\n",
            self.get_piece(PieceType::Bishop, WHITE)
        ));
        res_str.push_str(&format!(
            "WhiteRooks: 0x{:x}u64\n",
            self.get_piece(PieceType::Rook, WHITE)
        ));
        res_str.push_str(&format!(
            "WhiteQueens: 0x{:x}u64\n",
            self.get_piece(PieceType::Queen, WHITE)
        ));
        res_str.push_str(&format!(
            "WhiteKing: 0x{:x}u64\n",
            self.get_piece(PieceType::King, WHITE)
        ));
        res_str.push_str(&format!(
            "BlackPawns: 0x{:x}u64\n",
            self.get_piece(PieceType::Pawn, BLACK)
        ));
        res_str.push_str(&format!(
            "BlackKnights: 0x{:x}u64\n",
            self.get_piece(PieceType::Knight, BLACK)
        ));
        res_str.push_str(&format!(
            "BlackBishops: 0x{:x}u64\n",
            self.get_piece(PieceType::Bishop, BLACK)
        ));
        res_str.push_str(&format!(
            "BlackRooks: 0x{:x}u64\n",
            self.get_piece(PieceType::Rook, BLACK)
        ));
        res_str.push_str(&format!(
            "BlackQueens: 0x{:x}u64\n",
            self.get_piece(PieceType::Queen, BLACK)
        ));
        res_str.push_str(&format!(
            "BlackKing: 0x{:x}u64\n",
            self.get_piece(PieceType::King, BLACK)
        ));
        res_str.push_str(&format!(
            "CWK: {}\n",
            self.irreversible.castle_white_kingside
        ));
        res_str.push_str(&format!(
            "CWQ: {}\n",
            self.irreversible.castle_white_queenside
        ));
        res_str.push_str(&format!(
            "CBK: {}\n",
            self.irreversible.castle_black_kingside
        ));
        res_str.push_str(&format!(
            "CBQ: {}\n",
            self.irreversible.castle_black_queenside
        ));
        res_str.push_str(&format!(
            "En-Passant: 0x{:x}u64\n",
            self.irreversible.en_passant
        ));
        res_str.push_str(&format!("half_moves: {}\n", self.irreversible.half_moves));
        res_str.push_str(&format!("full_moves: {}\n", self.full_moves));
        res_str.push_str(&format!("Side to Move: {}\n", self.color_to_move));
        res_str.push_str(&format!("Hash: {}\n", self.hash));
        res_str.push_str(&format!(
            "Checkers: 0x{:x}u64\n",
            self.irreversible.checkers
        ));
        res_str.push_str(&format!("Phase: {}\n", self.phase.phase));
        res_str.push_str(&format!("PSQT: {}\n", self.psqt));
        res_str.push_str(&format!("FEN: {}\n", self.to_fen()));
        write!(formatter, "{}", res_str)
    }
}

impl PartialEq for GameState {
    fn eq(&self, other: &Self) -> bool {
        self.color_to_move == other.color_to_move
            && self.pieces_bb == other.pieces_bb
            && self.color_bb == other.color_bb
            && self.irreversible == other.irreversible
            && self.full_moves == other.full_moves
            && self.hash == other.hash
            && self.psqt == other.psqt
            && self.phase == other.phase
    }
}
