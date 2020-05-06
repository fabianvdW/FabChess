use crate::bitboards::bitboards::constants::{square, KING_ATTACKS};
use crate::bitboards::bitboards::knight_attacks;
use crate::board_representation::game_state::{
    GameMove, GameMoveType, GameState, PieceType, BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK,
    WHITE,
};
use crate::move_generation::movegen::{
    b_pawn_east_targets, b_pawn_west_targets, bishop_attacks, double_push_pawn_targets,
    pawn_east_targets, pawn_targets, pawn_west_targets, rook_attacks, single_push_pawn_targets,
    w_pawn_east_targets, w_pawn_west_targets,
};
use crate::move_generation::movelist::MoveList;

impl GameState {
    #[inline(always)]
    pub fn relative_rank(sq: usize, white: bool) -> usize {
        if white {
            sq / 8
        } else {
            7 - sq / 8
        }
    }

    pub fn in_check(&self) -> bool {
        self.irreversible.checkers > 0
    }
    pub fn gives_check(&self, mv: GameMove) -> bool {
        if mv.move_type == GameMoveType::Castle {
            return false; // In theory a castle move can give_check, but it is too much hasssle to compute that
        }
        //We also ignore en passant discovered checks here
        let mut occ_board = self.all_pieces();
        occ_board ^= square(mv.from as usize);
        occ_board |= square(mv.to as usize);
        let king_position = self.king_square(1 - self.color_to_move);
        let bishop_like_attack = bishop_attacks(king_position, occ_board);
        let rook_like_attack = rook_attacks(king_position, occ_board);
        //CHeck discovered check
        if bishop_like_attack
            & (self.pieces[BISHOP][self.color_to_move] | self.pieces[QUEEN][self.color_to_move])
            != 0u64
            || rook_like_attack
                & (self.pieces[ROOK][self.color_to_move] | self.pieces[QUEEN][self.color_to_move])
                != 0u64
        {
            return true;
        }
        match mv.piece_type {
            PieceType::King => false,
            PieceType::Queen => {
                (bishop_like_attack | rook_like_attack) & square(mv.to as usize) != 0u64
            }
            PieceType::Knight => {
                knight_attacks(square(king_position)) & square(mv.to as usize) != 0u64
            }
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
                        knight_attacks(square(king_position)) & square(mv.to as usize) != 0u64
                    }
                    _ => panic!("Not a valid promotion piece. Game_state.rs #1"),
                },
                _ => panic!("Not a valid pawn move. Game_state.rs #2"),
            },
        }
    }

    #[inline(always)]
    pub fn piecetype_on(&self, sq: usize, side: usize) -> PieceType {
        if self.pieces[PAWN][side] & square(sq) != 0u64 {
            PieceType::Pawn
        } else if self.pieces[KNIGHT][side] & square(sq) != 0u64 {
            PieceType::Knight
        } else if self.pieces[QUEEN][side] & square(sq) != 0u64 {
            PieceType::Queen
        } else if self.pieces[BISHOP][side] & square(sq) != 0u64 {
            PieceType::Bishop
        } else if self.pieces[ROOK][side] & square(sq) != 0u64 {
            PieceType::Rook
        } else {
            debug_assert!(self.pieces[KING][side] & square(sq) > 0);
            PieceType::King
        }
    }
    #[inline(always)]
    pub fn move_type_to(&self, to: usize) -> GameMoveType {
        if self.pieces[PAWN][1 - self.color_to_move] & square(to) != 0u64 {
            GameMoveType::Capture(PieceType::Pawn)
        } else if self.pieces[KNIGHT][1 - self.color_to_move] & square(to) != 0u64 {
            GameMoveType::Capture(PieceType::Knight)
        } else if self.pieces[QUEEN][1 - self.color_to_move] & square(to) != 0u64 {
            GameMoveType::Capture(PieceType::Queen)
        } else if self.pieces[BISHOP][1 - self.color_to_move] & square(to) != 0u64 {
            GameMoveType::Capture(PieceType::Bishop)
        } else if self.pieces[ROOK][1 - self.color_to_move] & square(to) != 0u64 {
            GameMoveType::Capture(PieceType::Rook)
        } else {
            debug_assert_eq!(self.pieces[KING][1 - self.color_to_move] & square(to), 0);
            GameMoveType::Quiet
        }
    }

    #[inline(always)]
    pub fn pieces_from_side(&self, side: usize) -> u64 {
        self.pieces_from_side_without_king(side) | self.pieces[KING][side]
    }

    #[inline(always)]
    pub fn pieces_from_side_without_king(&self, side: usize) -> u64 {
        self.pieces[PAWN][side]
            | self.pieces[KNIGHT][side]
            | self.pieces[BISHOP][side]
            | self.pieces[ROOK][side]
            | self.pieces[QUEEN][side]
    }

    #[inline(always)]
    pub fn all_pieces(&self) -> u64 {
        self.pieces_from_side(WHITE) | self.pieces_from_side(BLACK)
    }
    #[inline(always)]
    pub fn empty(&self) -> u64 {
        !self.all_pieces()
    }

    #[inline(always)]
    pub fn king_square(&self, side: usize) -> usize {
        self.pieces[KING][side].trailing_zeros() as usize
    }

    #[inline(always)]
    pub fn has_non_pawns(&self, side: usize) -> bool {
        self.pieces[BISHOP][side] != 0u64
            || self.pieces[KNIGHT][side] != 0u64
            || self.pieces[ROOK][side] != 0u64
            || self.pieces[QUEEN][side] != 0u64
    }

    #[inline(always)]
    pub fn is_valid_castle(&self, white: bool, kingside: bool, occ: &mut u64) -> bool {
        pub const CASTLE_SQUARES: [[(usize, usize, usize, usize); 2]; 2] = [
            [(5, 6, 5, 7), (2, 3, 0, 3)],
            [(61, 62, 61, 63), (58, 59, 56, 59)],
        ];
        let castle_squares =
            CASTLE_SQUARES[if white { 0 } else { 1 }][if kingside { 0 } else { 1 }];
        if self.square_attacked(castle_squares.0, *occ, 0u64)
            || self.square_attacked(castle_squares.1, *occ, 0u64)
        {
            return false;
        }
        *occ ^= square(castle_squares.2) ^ square(castle_squares.3);
        true
    }

    //Returns true if a pseudolegal move generated by generate_pseudolegal_moves is completly legal
    //Do not call this on any other move
    #[inline(always)]
    pub fn is_valid_move(&self, mv: GameMove) -> bool {
        //Check if our king would be in check after the move
        let king_idx = if mv.piece_type == PieceType::King {
            mv.to as usize
        } else {
            self.king_square(self.color_to_move)
        };
        let mut occ = self.all_pieces();
        let mut exclude = square(mv.to as usize);
        if mv.move_type == GameMoveType::EnPassant {
            //The ^8 Trick:
            //The fields 40-47 all contain 32+8, does xoring 8 removes 8 fields, getting the en-passented pawn
            //The fields 16-23 all DON't contain the 8-bit, thus xoring 8 adds 8 fields, getting the en-passented pawn
            occ ^= square((mv.to ^ 8) as usize);
            square((mv.to ^ 8) as usize);
            exclude |= square((mv.to ^ 8) as usize);
        //Remove enpassented pawn
        } else if mv.move_type == GameMoveType::Castle {
            let white = self.color_to_move == WHITE;
            let kingside = mv.to as usize == self.king_square(self.color_to_move) + 2;
            if !self.is_valid_castle(white, kingside, &mut occ) {
                return false;
            }
        }
        occ = (occ ^ square(mv.from as usize)) | square(mv.to as usize);
        !self.square_attacked(king_idx, occ, exclude)
    }

    //Returns true if the given square is attacked by the side not to move
    //occ: Blockers in the current position
    #[inline(always)]
    pub fn square_attacked(&self, sq: usize, occ: u64, exclude: u64) -> bool {
        let square = square(sq);
        KING_ATTACKS[sq] & self.pieces[KING][1 - self.color_to_move] & !exclude > 0
            || knight_attacks(square) & self.pieces[KNIGHT][1 - self.color_to_move] & !exclude > 0
            || bishop_attacks(sq, occ)
                & (self.pieces[BISHOP][1 - self.color_to_move]
                    | self.pieces[QUEEN][1 - self.color_to_move])
                & !exclude
                > 0
            || rook_attacks(sq, occ)
                & (self.pieces[ROOK][1 - self.color_to_move]
                    | self.pieces[QUEEN][1 - self.color_to_move])
                & !exclude
                > 0
            || pawn_targets(self.color_to_move, square)
                & self.pieces[PAWN][1 - self.color_to_move]
                & !exclude
                > 0
    }
    //Returns a bitboard of all the pieces attacking the square
    //occ: Blockers in the current position
    #[inline(always)]
    pub fn square_attackers(&self, sq: usize, occ: u64) -> u64 {
        let square = square(sq);
        KING_ATTACKS[sq] & self.pieces[KING][1 - self.color_to_move]
            | knight_attacks(square) & self.pieces[KNIGHT][1 - self.color_to_move]
            | bishop_attacks(sq, occ)
                & (self.pieces[BISHOP][1 - self.color_to_move]
                    | self.pieces[QUEEN][1 - self.color_to_move])
            | rook_attacks(sq, occ)
                & (self.pieces[ROOK][1 - self.color_to_move]
                    | self.pieces[QUEEN][1 - self.color_to_move])
            | pawn_targets(self.color_to_move, square) & self.pieces[PAWN][1 - self.color_to_move]
    }

    #[inline(always)]
    pub(crate) fn castle_target_square(&self, kingside: bool) -> u8 {
        if self.color_to_move == WHITE {
            if kingside {
                6
            } else {
                2
            }
        } else {
            if kingside {
                62
            } else {
                58
            }
        }
    }
}

#[inline(always)]
pub fn generate_king(game_state: &GameState, movelist: &mut MoveList, mask: u64) {
    let king_idx = game_state.king_square(game_state.color_to_move);
    //Normal king attacks
    let valid_attacks = KING_ATTACKS[king_idx] & mask;
    movelist.add_bb(king_idx as u8, PieceType::King, valid_attacks, game_state);
    //Castle
    if !game_state.in_check() {
        let (ks, qs) = if game_state.color_to_move == WHITE {
            (
                game_state.irreversible.castle_white_kingside
                    && (game_state.all_pieces() & (square(5) | square(6)) == 0),
                game_state.irreversible.castle_white_queenside
                    && (game_state.all_pieces() & (square(1) | square(2) | square(3)) == 0),
            )
        } else {
            (
                game_state.irreversible.castle_black_kingside
                    && (game_state.all_pieces() & (square(62) | square(61))) == 0,
                game_state.irreversible.castle_black_queenside
                    && (game_state.all_pieces() & (square(57) | square(58) | square(59)) == 0),
            )
        };
        if ks && mask & square(game_state.castle_target_square(true) as usize) > 0 {
            movelist.add_move(GameMove {
                from: king_idx as u8,
                to: game_state.castle_target_square(true),
                move_type: GameMoveType::Castle,
                piece_type: PieceType::King,
            })
        }
        if qs && mask & square(game_state.castle_target_square(false) as usize) > 0 {
            movelist.add_move(GameMove {
                from: king_idx as u8,
                to: game_state.castle_target_square(false),
                move_type: GameMoveType::Castle,
                piece_type: PieceType::King,
            })
        }
    }
}

impl PieceType {
    #[inline(always)]
    fn attacks(&self, from: usize, game_state: &GameState) -> u64 {
        match self {
            PieceType::King | PieceType::Pawn => panic!("Piece types not supported."),
            PieceType::Knight => knight_attacks(square(from)), //TODO: Test if KNIGHT_ATTACK[knight_idx as usize] is faster
            PieceType::Rook => rook_attacks(from, game_state.all_pieces()),
            PieceType::Queen => {
                bishop_attacks(from, game_state.all_pieces())
                    | rook_attacks(from, game_state.all_pieces())
            }
            PieceType::Bishop => bishop_attacks(from, game_state.all_pieces()),
        }
    }
}
#[inline(always)]
pub fn generate_others(
    game_state: &GameState,
    movelist: &mut MoveList,
    mask: u64,
    piece_type: PieceType,
) {
    let mut pieces = game_state.pieces[piece_type.to_index()][game_state.color_to_move];
    while pieces > 0 {
        let piece_idx = pieces.trailing_zeros();
        let attack = mask & piece_type.attacks(piece_idx as usize, game_state);
        movelist.add_bb(piece_idx as u8, piece_type, attack, game_state);
        pieces ^= square(piece_idx as usize);
    }
}
#[inline(always)]
pub fn generate_pawns_quiets(game_state: &GameState, movelist: &mut MoveList, mask: u64) {
    let pawns = game_state.pieces[PAWN][game_state.color_to_move];
    let empty = game_state.empty();
    let single_push_targets =
        single_push_pawn_targets(game_state.color_to_move, pawns, empty) & mask;
    let double_push_targets =
        double_push_pawn_targets(game_state.color_to_move, pawns, empty) & mask;
    let ctm = if game_state.color_to_move == WHITE {
        -1
    } else {
        1
    };
    movelist.add_pawn_bb(single_push_targets, ctm * 8, game_state);
    movelist.add_pawn_bb(double_push_targets, ctm * 16, game_state);
}
#[inline(always)]
pub fn generate_pawns_captures(game_state: &GameState, movelist: &mut MoveList, mask: u64) {
    let pawns = game_state.pieces[PAWN][game_state.color_to_move];
    let other = game_state.pieces_from_side(1 - game_state.color_to_move);
    let west_targets = pawn_west_targets(game_state.color_to_move, pawns) & mask;
    let east_targets = pawn_east_targets(game_state.color_to_move, pawns) & mask;

    let ctm = if game_state.color_to_move == WHITE {
        -1
    } else {
        1
    };
    movelist.add_pawn_bb(west_targets & other, ctm * 7, game_state);
    movelist.add_pawn_bb(east_targets & other, ctm * 9, game_state);

    if west_targets & game_state.irreversible.en_passant > 0 {
        movelist.add_move(GameMove {
            from: (game_state.irreversible.en_passant.trailing_zeros() as i8 + ctm * 7) as u8,
            to: game_state.irreversible.en_passant.trailing_zeros() as u8,
            piece_type: PieceType::Pawn,
            move_type: GameMoveType::EnPassant,
        })
    }
    //En-Passants
    if east_targets & game_state.irreversible.en_passant > 0 {
        movelist.add_move(GameMove {
            from: (game_state.irreversible.en_passant.trailing_zeros() as i8 + ctm * 9) as u8,
            to: game_state.irreversible.en_passant.trailing_zeros() as u8,
            piece_type: PieceType::Pawn,
            move_type: GameMoveType::EnPassant,
        })
    }
}

//Make sure the movelist is cleared before you call this
#[inline(always)]
pub fn generate_pseudolegal_captures(game_state: &GameState, movelist: &mut MoveList) {
    let mut general_mask = game_state.pieces_from_side(1 - game_state.color_to_move);
    generate_king(game_state, movelist, general_mask);
    if game_state.irreversible.checkers.count_ones() <= 1 {
        if game_state.irreversible.checkers.count_ones() == 1 {
            general_mask = game_state.irreversible.checkers;
        }
        generate_pawns_captures(
            game_state,
            movelist,
            general_mask | game_state.irreversible.en_passant,
        );
        generate_others(game_state, movelist, general_mask, PieceType::Knight);
        generate_others(game_state, movelist, general_mask, PieceType::Bishop);
        generate_others(game_state, movelist, general_mask, PieceType::Rook);
        generate_others(game_state, movelist, general_mask, PieceType::Queen);
    }
}
//Make sure movelist is cleared before you call this
#[inline(always)]
pub fn generate_pseudolegal_quiets(game_state: &GameState, movelist: &mut MoveList) {
    let general_mask = game_state.empty();
    generate_king(game_state, movelist, general_mask);
    if game_state.irreversible.checkers.count_ones() <= 1 {
        generate_pawns_quiets(game_state, movelist, general_mask);
        generate_others(game_state, movelist, general_mask, PieceType::Knight);
        generate_others(game_state, movelist, general_mask, PieceType::Queen);
        generate_others(game_state, movelist, general_mask, PieceType::Bishop);
        generate_others(game_state, movelist, general_mask, PieceType::Rook);
    }
}

pub fn generate_pseudolegal_moves(game_state: &GameState, movelist: &mut MoveList) {
    movelist.move_list.clear();
    //Generate pseudolegal moves given a position
    let general_mask = !game_state.pieces_from_side(game_state.color_to_move);
    generate_king(game_state, movelist, general_mask);
    if game_state.irreversible.checkers.count_ones() <= 1 {
        //TODO : update general mask with updated stuff
        generate_pawns_quiets(game_state, movelist, general_mask);
        generate_pawns_captures(game_state, movelist, general_mask);
        generate_others(game_state, movelist, general_mask, PieceType::Knight);
        generate_others(game_state, movelist, general_mask, PieceType::Queen);
        generate_others(game_state, movelist, general_mask, PieceType::Bishop);
        generate_others(game_state, movelist, general_mask, PieceType::Rook);
    }
}

//This is a rather slow function and should only be used when speed is not really important. E.g. one time costs
pub fn generate_legal_moves(game_state: &GameState) -> MoveList {
    let mut res = MoveList::default();
    generate_pseudolegal_moves(game_state, &mut res);
    res.move_list.retain(|mv| game_state.is_valid_move(mv.0));
    res
}
