use crate::bitboards::bitboards::constants::square;
use crate::bitboards::bitboards::constants::{KING_ATTACKS, KNIGHT_ATTACKS};
use crate::bitboards::bitboards::square;
use crate::board_representation::zobrist_hashing::ZOBRIST_KEYS;
use crate::evaluation::params::*;
use crate::evaluation::phase::Phase;
use crate::evaluation::EvaluationScore;
use crate::move_generation::makemove::make_move;
use crate::move_generation::movegen::{
    b_pawn_east_targets, b_pawn_west_targets, bishop_attack, double_push_pawn_targets,
    generate_moves, pawn_east_targets, pawn_west_targets, rook_attack, single_push_pawn_targets,
    w_pawn_east_targets, w_pawn_west_targets, MoveList,
};
use std::fmt::{Debug, Display, Formatter, Result};
pub const CASTLE_WHITE_KS: u8 = 0b1000;
pub const CASTLE_WHITE_QS: u8 = 0b100;
pub const CASTLE_BLACK_KS: u8 = 0b10;
pub const CASTLE_BLACK_QS: u8 = 0b1;
pub const CASTLE_ALL_WHITE: u8 = CASTLE_WHITE_KS | CASTLE_WHITE_QS;
pub const CASTLE_ALL_BLACK: u8 = CASTLE_BLACK_KS | CASTLE_BLACK_QS;
pub const CASTLE_ALL: u8 = CASTLE_ALL_WHITE | CASTLE_ALL_BLACK;
pub const WHITE: usize = 0;
pub const BLACK: usize = 1;
pub const PIECE_TYPES: [PieceType; 6] = [
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen,
    PieceType::King,
];

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
    King = 5,
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
}
impl PieceType {
    pub fn is_valid_promotion_piece(self) -> bool {
        self != PieceType::Pawn && self != PieceType::King
    }
    pub fn lowercase(self) -> &'static str {
        match self {
            PieceType::Pawn => "p",
            PieceType::Knight => "n",
            PieceType::Bishop => "b",
            PieceType::Rook => "r",
            PieceType::Queen => "q",
            PieceType::King => "k",
        }
    }
    pub fn uppercase(self) -> &'static str {
        match self {
            PieceType::Pawn => "P",
            PieceType::Knight => "N",
            PieceType::Bishop => "B",
            PieceType::Rook => "R",
            PieceType::Queen => "Q",
            PieceType::King => "K",
        }
    }

    #[inline(always)]
    pub fn to_psqt(self, side: usize, sq: usize) -> EvaluationScore {
        PSQT[self as usize][side][sq / 8][sq % 8]
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

#[derive(Copy, Clone, PartialEq)]
pub struct GameMove {
    pub from: u8,
    pub to: u8,
    pub move_type: GameMoveType,
    pub piece_type: PieceType,
}

impl GameMove {
    #[inline(always)]
    pub fn is_capture(self) -> bool {
        match self.move_type {
            GameMoveType::Capture(_)
            | GameMoveType::Promotion(_, Some(_))
            | GameMoveType::EnPassant => true,
            _ => false,
        }
    }
    #[inline(always)]
    pub fn get_captured_piece(self) -> PieceType {
        debug_assert!(self.is_capture());
        match self.move_type {
            GameMoveType::Capture(p) | GameMoveType::Promotion(_, Some(p)) => p,
            GameMoveType::EnPassant => PieceType::Pawn,
            _ => panic!("Captured piece type  called on a capture"),
        }
    }
    pub fn get_maybe_captured_piece(self) -> Option<PieceType> {
        match self.move_type {
            GameMoveType::Capture(p) | GameMoveType::Promotion(_, Some(p)) => Some(p),
            GameMoveType::EnPassant => Some(PieceType::Pawn),
            _ => None,
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
        generate_moves(game_state, false, &mut movelist);
        let mut res_str = String::new();
        if self.move_type == GameMoveType::Castle {
            if self.to == 2 || self.to == 58 {
                res_str.push_str("O-O-O");
            } else {
                res_str.push_str("O-O");
            }
        } else {
            res_str.push_str(if self.piece_type != PieceType::Pawn {
                self.piece_type.uppercase()
            } else {
                ""
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
                assert!(promo_piece.is_valid_promotion_piece());
                res_str.push_str(&format!("={}", promo_piece.uppercase()));
            }
        }
        let game_state = make_move(game_state, self);
        let agsi = generate_moves(&game_state, false, &mut movelist);
        if agsi.stm_incheck && movelist.move_list.is_empty() {
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
            assert!(s.is_valid_promotion_piece());
            res_str.push_str(s.lowercase());
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
pub struct Irreversible {
    hash: u64,
    en_passant: u64,
    half_moves: u16,
    castle_permissions: u8,
    phase: Phase,
    psqt: EvaluationScore,
}
impl Irreversible {
    pub fn new(
        hash: u64,
        en_passant: u64,
        half_moves: u16,
        castle_permissions: u8,
        phase: Phase,
        psqt: EvaluationScore,
    ) -> Self {
        Irreversible {
            hash,
            en_passant,
            half_moves,
            castle_permissions,
            phase,
            psqt,
        }
    }
}
#[derive(Clone)]
pub struct GameState {
    // 0 = White
    // 1 = Black
    color_to_move: usize,

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
    color_bb: [u64; 2],
    piece_bb: [u64; 6],

    irreversible: Irreversible,

    full_moves: usize,
}
//Getters and setters
impl GameState {
    pub fn get_piece_bb_array(&self) -> [u64; 6] {
        self.piece_bb
    }
    pub fn get_color_bb_array(&self) -> [u64; 2] {
        self.color_bb
    }
    pub fn get_full_moves(&self) -> usize {
        self.full_moves
    }
    pub fn get_color_to_move(&self) -> usize {
        self.color_to_move
    }
    pub fn set_color_to_move(&mut self, new: usize) {
        self.color_to_move = new
    }
    pub fn get_hash(&self) -> u64 {
        self.irreversible.hash
    }
    pub fn get_en_passant(&self) -> u64 {
        self.irreversible.en_passant
    }
    pub fn get_half_moves(&self) -> usize {
        self.irreversible.half_moves as usize
    }
    pub fn get_phase(&self) -> &Phase {
        &self.irreversible.phase
    }
    pub fn get_psqt(&self) -> EvaluationScore {
        self.irreversible.psqt
    }

    pub fn get_piece(&self, piece_type: PieceType, side: usize) -> u64 {
        self.piece_bb[piece_type as usize] & self.color_bb[side]
    }
    pub fn get_piece_bb(&self, piece_type: PieceType) -> u64 {
        self.piece_bb[piece_type as usize]
    }
    pub fn get_piece_amt(&self, piece_type: PieceType, side: usize) -> usize {
        self.get_piece(piece_type, side).count_ones() as usize
    }

    pub fn get_bishop_like_bb(&self, side: usize) -> u64 {
        self.get_piece(PieceType::Bishop, side) | self.get_piece(PieceType::Queen, side)
    }
    pub fn get_rook_like_bb(&self, side: usize) -> u64 {
        self.get_piece(PieceType::Rook, side) | self.get_piece(PieceType::Queen, side)
    }
    pub fn get_king_square(&self, side: usize) -> usize {
        self.get_piece(PieceType::King, side).trailing_zeros() as usize
    }
    pub fn get_pieces_from_side(&self, side: usize) -> u64 {
        self.color_bb[side]
    }
    pub fn get_pieces_from_side_without_king(&self, side: usize) -> u64 {
        self.get_pieces_from_side(side) & !self.get_piece_bb(PieceType::King)
    }
    pub fn get_all_pieces(&self) -> u64 {
        self.get_pieces_from_side(WHITE) | self.get_pieces_from_side(BLACK)
    }
    pub fn get_all_pieces_without_ctm_king(&self) -> u64 {
        self.get_pieces_from_side_without_king(self.color_to_move)
            | self.get_pieces_from_side(1 - self.color_to_move)
    }
    pub fn castle_white_kingside(&self) -> bool {
        self.irreversible.castle_permissions & CASTLE_WHITE_KS > 0
    }
    pub fn castle_white_queenside(&self) -> bool {
        self.irreversible.castle_permissions & CASTLE_WHITE_QS > 0
    }
    pub fn castle_black_kingside(&self) -> bool {
        self.irreversible.castle_permissions & CASTLE_BLACK_KS > 0
    }
    pub fn castle_black_queenside(&self) -> bool {
        self.irreversible.castle_permissions & CASTLE_BLACK_QS > 0
    }
    pub fn castle_permissions(&self) -> u8 {
        self.irreversible.castle_permissions
    }
}

//Utility functions
impl GameState {
    pub fn new(
        color_to_move: usize,
        piece_bb: [u64; 6],
        color_bb: [u64; 2],
        irreversible: Irreversible,
        full_moves: usize,
    ) -> Self {
        GameState {
            color_to_move,
            piece_bb,
            color_bb,
            irreversible,
            full_moves,
        }
    }
    pub fn relative_rank(side: usize, sq: usize) -> usize {
        if side == WHITE {
            sq / 8
        } else {
            7 - sq / 8
        }
    }
    pub fn get_piece_on(&self, shift: i32) -> &str {
        for side in 0..2 {
            for piece_type in PIECE_TYPES.iter() {
                if (self.get_piece(*piece_type, side) >> shift) & 1u64 > 0 {
                    if side == WHITE {
                        return piece_type.uppercase();
                    } else {
                        return piece_type.lowercase();
                    }
                }
            }
        }
        " "
    }
}
impl GameState {
    pub fn initialize_zobrist_hash(&mut self) {
        self.irreversible.hash = 0u64;
        if self.color_to_move == BLACK {
            self.irreversible.hash ^= ZOBRIST_KEYS.side_to_move;
        }
        self.irreversible.hash ^=
            ZOBRIST_KEYS.castle_permissions[self.castle_permissions() as usize];
        if self.get_en_passant() != 0u64 {
            let file = self.get_en_passant().trailing_zeros() as usize % 8;
            self.irreversible.hash ^= ZOBRIST_KEYS.en_passant[file];
        }
        for side in 0..2 {
            for pt in PIECE_TYPES.iter() {
                let mut piece = self.get_piece(*pt, side);
                while piece > 0 {
                    let idx = piece.trailing_zeros() as usize;
                    self.irreversible.hash ^= ZOBRIST_KEYS.pieces[side][*pt as usize][idx];
                    piece ^= square(idx);
                }
            }
        }
    }
    pub fn initialize_psqt(&mut self) {
        let p_w = crate::evaluation::psqt_evaluation::psqt(
            self,
            WHITE,
            #[cfg(feature = "texel-tuning")]
            &mut crate::evaluation::trace::LargeTrace::default(),
        );
        let p_b = crate::evaluation::psqt_evaluation::psqt(
            self,
            BLACK,
            #[cfg(feature = "texel-tuning")]
            &mut crate::evaluation::trace::LargeTrace::default(),
        );
        self.irreversible.psqt = p_w - p_b
    }
    pub fn initialize_phase(&mut self) {
        self.irreversible.phase = Phase::from_state(self);
    }
    pub fn initialize(&mut self) {
        self.initialize_zobrist_hash();
        self.initialize_psqt();
        self.initialize_phase();
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
        let mut piece_bb: [u64; 6] = [0u64; 6];
        let mut color_bb: [u64; 2] = [0u64; 2];
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
                            piece_bb[piece_type as usize] |= square(idx);
                            file += 1;
                        } else {
                            file += x.to_string().parse::<usize>().expect("Invalid Fen!");
                        }
                    }
                    None => panic!("Invalid FEN"),
                };
            }
        }

        //Side to move
        let color_to_move = match vec[1] {
            "w" => WHITE,
            "b" => BLACK,
            _ => panic!("Invalid FEN!"),
        };

        //Castling-Abilities
        let mut castle_permissions = 0u8;
        if vec[2].contains('K') {
            castle_permissions |= CASTLE_WHITE_KS
        }
        if vec[2].contains('Q') {
            castle_permissions |= CASTLE_WHITE_QS
        }
        if vec[2].contains('k') {
            castle_permissions |= CASTLE_BLACK_KS
        }
        if vec[2].contains('q') {
            castle_permissions |= CASTLE_BLACK_QS
        }
        //En passant target square
        let en_passant: u64 = if vec[3] != "-" {
            let mut idx: usize = 0usize;
            let file = vec[3].chars().next();
            let rank = vec[3].chars().nth(1);
            idx += char_to_file(file.expect("Invalid FEN!").to_ascii_lowercase());
            idx += 8 * char_to_rank(rank.expect("Invalid FEN!"));
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
        let mut res = GameState::new(
            color_to_move,
            piece_bb,
            color_bb,
            Irreversible::new(
                0u64,
                en_passant,
                half_moves,
                castle_permissions,
                Phase::default(),
                EvaluationScore(0, 0),
            ),
            full_moves,
        );
        res.initialize();
        res
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
        if self.castle_permissions() == 0 {
            res_str.push_str("-");
        } else {
            if self.castle_white_kingside() {
                res_str.push_str("K");
            }
            if self.castle_white_queenside() {
                res_str.push_str("Q");
            }
            if self.castle_black_kingside() {
                res_str.push_str("k");
            }
            if self.castle_black_queenside() {
                res_str.push_str("q");
            }
        }
        res_str.push_str(" ");

        if self.get_en_passant() == 0u64 {
            res_str.push_str("-");
        } else {
            let idx = self.get_en_passant().trailing_zeros() as usize;
            let rank = idx / 8;
            let file = idx % 8;
            res_str.push_str(&format!("{}{}", file_to_string(file), rank + 1));
        }
        res_str.push_str(" ");
        res_str.push_str(&format!("{} ", self.get_half_moves()));
        res_str.push_str(&format!("{}", self.get_full_moves()));
        res_str
    }

    pub fn standard() -> GameState {
        let color_to_move = 0usize;
        let mut piece_bb = [0u64; 6];
        let mut color_bb = [0u64; 2];
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
        for (pt, side, bb) in info.iter() {
            piece_bb[*pt as usize] |= bb;
            color_bb[*side] |= bb;
        }
        let mut res = GameState::new(
            color_to_move,
            piece_bb,
            color_bb,
            Irreversible::new(
                0u64,
                0u64,
                0,
                CASTLE_ALL,
                Phase::default(),
                EvaluationScore(0, 0),
            ),
            1,
        );
        res.initialize();
        res
    }

    #[inline(always)]
    pub fn has_non_pawns(&self, side: usize) -> bool {
        self.get_piece(PieceType::Bishop, side) != 0u64
            || self.get_piece(PieceType::Knight, side) != 0u64
            || self.get_piece(PieceType::Rook, side) != 0u64
            || self.get_piece(PieceType::Queen, side) != 0u64
    }

    //Calculates if a given move gives check. Does not necessarily return true even if move gives check
    //For instance, won't ever return true on a castling move that gives check.
    pub fn gives_check(&self, mv: GameMove) -> bool {
        if mv.move_type == GameMoveType::Castle {
            return false; // In theory a castle move can give_check, but it is too much hasssle to compute that
        }
        //We also ignore en passant discovered checks here
        let mut occ_board = self.get_all_pieces();
        occ_board ^= square(mv.from as usize);
        occ_board |= square(mv.to as usize);
        let king_position = self.get_king_square(1 - self.color_to_move);
        let bishop_like_attack = bishop_attack(king_position, occ_board);
        let rook_like_attack = rook_attack(king_position, occ_board);
        //Check discovered check
        if bishop_like_attack & self.get_bishop_like_bb(self.color_to_move) != 0u64
            || rook_like_attack & self.get_rook_like_bb(self.color_to_move) != 0u64
        {
            return true;
        }
        match mv.piece_type {
            PieceType::King => false,
            PieceType::Queen => {
                (bishop_like_attack | rook_like_attack) & square(mv.to as usize) != 0u64
            }
            PieceType::Knight => KNIGHT_ATTACKS[king_position] & square(mv.to as usize) != 0u64,
            PieceType::Bishop => bishop_like_attack & square(mv.to as usize) != 0u64,
            PieceType::Rook => rook_like_attack & square(mv.to as usize) != 0u64,
            PieceType::Pawn => match mv.move_type {
                GameMoveType::Quiet | GameMoveType::Capture(_) | GameMoveType::EnPassant => {
                    if self.color_to_move == WHITE {
                        (w_pawn_east_targets(square(mv.to as usize))
                            | w_pawn_west_targets(square(mv.to as usize)))
                            & square(king_position)
                            != 0u64
                    } else {
                        (b_pawn_east_targets(square(mv.to as usize))
                            | b_pawn_west_targets(square(mv.to as usize)))
                            & square(king_position)
                            != 0u64
                    }
                }
                GameMoveType::Promotion(p, _) => match p {
                    PieceType::Rook => rook_like_attack & square(mv.to as usize) != 0u64,
                    PieceType::Queen => {
                        (bishop_like_attack | rook_like_attack) & square(mv.to as usize) != 0u64
                    }
                    PieceType::Bishop => bishop_like_attack & square(mv.to as usize) != 0u64,
                    PieceType::Knight => {
                        KNIGHT_ATTACKS[king_position] & square(mv.to as usize) != 0u64
                    }
                    _ => panic!("Not a valid promotion piece."),
                },
                _ => panic!("Not a valid pawn move."),
            },
        }
    }

    pub fn is_valid_tt_move(&self, mv: GameMove) -> bool {
        if self.get_piece(mv.piece_type, self.color_to_move) & square(mv.from as usize) == 0u64 {
            return false;
        }
        if mv.piece_type == PieceType::Pawn
            && GameState::relative_rank(self.color_to_move, mv.to as usize) == 7
        {
            if let GameMoveType::Promotion(_, _) = mv.move_type {
            } else {
                return false;
            }
        } else if let GameMoveType::Promotion(_, _) = mv.move_type {
            if mv.piece_type != PieceType::Pawn
                || GameState::relative_rank(self.color_to_move, mv.to as usize) != 7
            {
                return false;
            }
        }

        if mv.move_type == GameMoveType::EnPassant {
            if (self.get_en_passant() & square(mv.to as usize)) == 0u64 {
                return false;
            }
        } else if mv.move_type == GameMoveType::Castle {
            if mv.piece_type != PieceType::King {
                return false;
            }
            let all_piece = self.get_all_pieces();
            if mv.to != square::G1 as u8
                && mv.to != square::C1 as u8
                && mv.to != square::G8 as u8
                && mv.to != square::C8 as u8
                || self.in_check()
            {
                return false;
            }
            let is_invalid_wk = || {
                mv.to == square::G1 as u8
                    && (!self.castle_white_kingside()
                        || all_piece & (square(square::F1) | square(square::G1)) > 0
                        || self.square_attacked(square::F1, all_piece, 0u64)
                        || self.square_attacked(square::G1, all_piece, 0u64))
            };
            let is_invalid_wq = || {
                mv.to == square::C1 as u8
                    && (!self.castle_white_queenside()
                        || all_piece
                            & (square(square::B1) | square(square::C1) | square(square::D1))
                            > 0
                        || self.square_attacked(square::C1, all_piece, 0u64)
                        || self.square_attacked(square::D1, all_piece, 0u64))
            };
            let is_invalid_bk = || {
                mv.to == square::G8 as u8
                    && (!self.castle_black_kingside()
                        || all_piece & (square(square::F8) | square(square::G8)) > 0
                        || self.square_attacked(square::F8, all_piece, 0u64)
                        || self.square_attacked(square::G8, all_piece, 0u64))
            };
            let is_invalid_bq = || {
                mv.to == square::C8 as u8
                    && (!self.castle_black_queenside()
                        || all_piece
                            & (square(square::B8) | square(square::C8) | square(square::D8))
                            > 0
                        || self.square_attacked(square::C8, all_piece, 0u64)
                        || self.square_attacked(square::D8, all_piece, 0u64))
            };
            if is_invalid_wk() || is_invalid_wq() || is_invalid_bk() || is_invalid_bq() {
                return false;
            }
        } else {
            let captured_piece = mv.get_maybe_captured_piece();
            if captured_piece.is_none() {
                if self.get_all_pieces() & square(mv.to as usize) != 0u64 {
                    return false;
                }
            } else if self.get_piece(captured_piece.unwrap(), 1 - self.color_to_move)
                & square(mv.to as usize)
                == 0u64
            {
                return false;
            }
        }
        let mut all_pieces = self.get_all_pieces();
        match mv.piece_type {
            PieceType::King => {
                if self.square_attacked(mv.to as usize, all_pieces ^ square(mv.from as usize), 0u64)
                    || mv.move_type != GameMoveType::Castle
                        && square(mv.to as usize) & (KING_ATTACKS[mv.from as usize]) == 0u64
                {
                    return false;
                }
            }
            PieceType::Bishop => {
                if square(mv.to as usize) & (bishop_attack(mv.from as usize, all_pieces)) == 0u64 {
                    return false;
                }
            }
            PieceType::Rook => {
                if square(mv.to as usize) & (rook_attack(mv.from as usize, all_pieces)) == 0u64 {
                    return false;
                }
            }
            PieceType::Queen => {
                if square(mv.to as usize)
                    & (bishop_attack(mv.from as usize, all_pieces)
                        | rook_attack(mv.from as usize, all_pieces))
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
                square((mv.to ^ 8) as usize)
            } else {
                square(mv.to as usize)
            };
            let king_square = if mv.piece_type == PieceType::King {
                mv.to as usize
            } else {
                self.get_king_square(self.color_to_move)
            };
            if bishop_attack(king_square, all_pieces)
                & self.get_bishop_like_bb(1 - self.color_to_move)
                & !cap_piece
                != 0u64
            {
                return false;
            }
            if rook_attack(king_square, all_pieces)
                & self.get_rook_like_bb(1 - self.color_to_move)
                & !cap_piece
                != 0u64
            {
                return false;
            }
            if KNIGHT_ATTACKS[king_square]
                & self.get_piece(PieceType::Knight, 1 - self.color_to_move)
                & !cap_piece
                != 0u64
            {
                return false;
            }
            if (pawn_east_targets(self.color_to_move, square(king_square))
                | pawn_west_targets(self.color_to_move, square(king_square)))
                & self.get_piece(PieceType::Pawn, 1 - self.color_to_move)
                & !cap_piece
                > 0
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
                res_str.push_str(self.get_piece_on(idx));
                res_str.push_str(" | ");
            }
            res_str.push_str("\n+---+---+---+---+---+---+---+---+\n");
        }
        res_str.push_str("Castle Rights: \n");
        res_str.push_str(&format!("CWK: {}\n", self.castle_white_kingside()));
        res_str.push_str(&format!("CWQ: {}\n", self.castle_white_queenside()));
        res_str.push_str(&format!("CBK: {}\n", self.castle_black_kingside()));
        res_str.push_str(&format!("CBQ: {}\n", self.castle_black_queenside()));
        res_str.push_str(&format!(
            "En Passant Possible: {:x}\n",
            self.get_en_passant()
        ));
        res_str.push_str(&format!("Half-Counter: {}\n", self.get_half_moves()));
        res_str.push_str(&format!("Full-Counter: {}\n", self.get_full_moves()));
        res_str.push_str(&format!("Side to Move: {}\n", self.get_color_to_move()));
        res_str.push_str(&format!("Hash: {}\n", self.get_hash()));
        res_str.push_str(&format!("FEN: {}\n", self.to_fen()));
        write!(formatter, "{}", res_str)
    }
}

impl Debug for GameState {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!("Color: {}\n", self.get_color_to_move()));
        for &color in [WHITE, BLACK].iter() {
            for &piece_type in PIECE_TYPES.iter() {
                res_str.push_str(&format!(
                    "{}{:?}: 0x{:x}u64\n",
                    if color == WHITE { "White" } else { "Black" },
                    piece_type,
                    self.get_piece(piece_type, color)
                ))
            }
        }
        res_str.push_str(&format!("CWK: {}\n", self.castle_white_kingside()));
        res_str.push_str(&format!("CWQ: {}\n", self.castle_white_queenside()));
        res_str.push_str(&format!("CBK: {}\n", self.castle_black_kingside()));
        res_str.push_str(&format!("CBQ: {}\n", self.castle_black_queenside()));
        res_str.push_str(&format!("En-Passant: 0x{:x}u64\n", self.get_en_passant()));
        res_str.push_str(&format!("half_moves: {}\n", self.get_half_moves()));
        res_str.push_str(&format!("full_moves: {}\n", self.get_full_moves()));
        res_str.push_str(&format!("Side to Move: {}\n", self.get_color_to_move()));
        res_str.push_str(&format!("Hash: {}\n", self.get_hash()));
        res_str.push_str(&format!("FEN: {}\n", self.to_fen()));
        write!(formatter, "{}", res_str)
    }
}
