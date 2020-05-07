use crate::bitboards::bitboards::constants::{square, CASTLE_PERMISSION, FILES};
use crate::board_representation::game_state::{
    GameMove, GameMoveType, GameState, Irreversible, PieceType, WHITE,
};
use crate::board_representation::zobrist_hashing::ZOBRIST_KEYS;
use crate::evaluation::psqt_evaluation::{psqt_set_piece, psqt_unset_piece};

#[inline(always)]
pub fn toggle_hash(piece: PieceType, square: u8, color: usize, hash: &mut u64) {
    *hash ^= piece.to_zobrist_key(color, square as usize);
}
#[inline(always)]
pub fn enpassant_hash(old: u64, new: u64, hash: &mut u64) {
    if old != 0u64 {
        *hash ^= ZOBRIST_KEYS.en_passant[old.trailing_zeros() as usize % 8];
    }
    if new != 0u64 {
        *hash ^= ZOBRIST_KEYS.en_passant[new.trailing_zeros() as usize % 8];
    }
}
#[inline(always)]
pub fn castle_hash(old: &Irreversible, new: &mut Irreversible) {
    new.hash ^= ZOBRIST_KEYS.castle_permissions[old.castle_permissions() as usize];
    new.hash ^= ZOBRIST_KEYS.castle_permissions[new.castle_permissions() as usize];
}
#[inline(always)]
//Returns the rook positions for a castle
pub fn rook_castling(to: u8) -> (usize, usize) {
    if square(to as usize) & FILES[2] > 0 {
        ((to - 2) as usize, (to + 1) as usize)
    } else {
        ((to + 1) as usize, (to - 1) as usize)
    }
}

pub fn make_nullmove(g: &mut GameState) -> Irreversible {
    let irr = g.irreversible.clone();
    g.irreversible.en_passant = 0u64;
    g.irreversible.half_moves += 1;
    g.full_moves = g.full_moves + g.color_to_move;
    g.irreversible.hash ^= ZOBRIST_KEYS.side_to_move;
    g.color_to_move = 1 - g.color_to_move;
    enpassant_hash(
        irr.en_passant,
        g.irreversible.en_passant,
        &mut g.irreversible.hash,
    );
    g.initialize_checkers();
    irr
}
pub fn unmake_nullmove(g: &mut GameState, irr: Irreversible) {
    g.color_to_move = 1 - g.color_to_move;
    g.full_moves = g.full_moves - g.color_to_move;
    g.irreversible = irr;
}

//TODO: use this outside of search, perft where performance does not matter
pub fn copy_make(g: &GameState, mv: GameMove) -> GameState {
    let mut res = g.clone();
    make_move(&mut res, mv);
    res
}

//We expect the move to be FULLY legal before it can be made!
pub fn make_move(g: &mut GameState, mv: GameMove) -> Irreversible {
    let irr = g.irreversible.clone();
    let color_to_move = 1 - g.color_to_move;
    //Step 1. Update immediate fields
    g.full_moves = g.full_moves + g.color_to_move;
    //Step 2. Update pieces, hash and other incremental fields
    g.irreversible.hash ^= ZOBRIST_KEYS.side_to_move;
    //Remove piece from original square
    g.unset_piece(mv.piece_type, mv.from as usize, g.color_to_move);
    toggle_hash(
        mv.piece_type,
        mv.from,
        g.color_to_move,
        &mut g.irreversible.hash,
    );
    psqt_unset_piece(g, mv.piece_type, mv.from as usize, g.color_to_move);

    let captured_piece = mv.get_maybe_captured_piece();
    //Delete piece if capture
    if let Some(piece) = captured_piece {
        let square = mv.to ^ (8 * (mv.move_type == GameMoveType::EnPassant) as u8);
        g.unset_piece(piece, square as usize, color_to_move);
        toggle_hash(piece, square, color_to_move, &mut g.irreversible.hash);
        psqt_unset_piece(g, piece, square as usize, color_to_move);
        g.irreversible.phase.delete_piece(piece);
    }
    //Move rook for castling
    if mv.move_type == GameMoveType::Castle {
        //Add piece again at to
        g.set_piece(mv.piece_type, mv.to as usize, g.color_to_move);
        toggle_hash(
            mv.piece_type,
            mv.to,
            g.color_to_move,
            &mut g.irreversible.hash,
        );
        psqt_set_piece(g, mv.piece_type, mv.to as usize, g.color_to_move);
        let (rook_from, rook_to) = rook_castling(mv.to);
        g.unset_piece(PieceType::Rook, rook_from, g.color_to_move);
        toggle_hash(
            PieceType::Rook,
            rook_from as u8,
            g.color_to_move,
            &mut g.irreversible.hash,
        );
        psqt_unset_piece(g, PieceType::Rook, rook_from as usize, g.color_to_move);
        g.set_piece(PieceType::Rook, rook_to, g.color_to_move);
        toggle_hash(
            PieceType::Rook,
            rook_to as u8,
            g.color_to_move,
            &mut g.irreversible.hash,
        );
        psqt_set_piece(g, PieceType::Rook, rook_to as usize, g.color_to_move);
    } else if let GameMoveType::Promotion(promo_piece, _) = mv.move_type {
        //If promotion, add promotion piece
        g.set_piece(promo_piece, mv.to as usize, g.color_to_move);
        toggle_hash(
            promo_piece,
            mv.to,
            g.color_to_move,
            &mut g.irreversible.hash,
        );
        psqt_set_piece(g, promo_piece, mv.to as usize, g.color_to_move);
        g.irreversible.phase.add_piece(promo_piece);
    } else {
        //Add piece again at to
        g.set_piece(mv.piece_type, mv.to as usize, g.color_to_move);
        toggle_hash(
            mv.piece_type,
            mv.to,
            g.color_to_move,
            &mut g.irreversible.hash,
        );
        psqt_set_piece(g, mv.piece_type, mv.to as usize, g.color_to_move);
    }
    g.irreversible.castle_permissions &=
        CASTLE_PERMISSION[mv.from as usize] & CASTLE_PERMISSION[mv.to as usize];
    castle_hash(&irr, &mut g.irreversible);
    //Step 4. Update en passant field
    g.irreversible.en_passant = if mv.move_type == GameMoveType::Quiet
        && mv.piece_type == PieceType::Pawn
        && (mv.to as isize - mv.from as isize).abs() == 16
    {
        if g.color_to_move == WHITE {
            square(mv.to as usize - 8)
        } else {
            square(mv.to as usize + 8)
        }
    } else {
        0
    };
    enpassant_hash(
        irr.en_passant,
        g.irreversible.en_passant,
        &mut g.irreversible.hash,
    );
    //Step 5. Half moves
    g.irreversible.half_moves =
        if mv.move_type == GameMoveType::Quiet && mv.piece_type != PieceType::Pawn {
            g.irreversible.half_moves + 1
        } else {
            0
        };
    g.color_to_move = color_to_move;
    g.initialize_checkers();
    irr
}

pub fn unmake_move(g: &mut GameState, mv: GameMove, irr: Irreversible) {
    g.color_to_move = 1 - g.color_to_move;
    //Revert the move
    if mv.move_type == GameMoveType::Castle {
        g.unset_piece(mv.piece_type, mv.to as usize, g.color_to_move);
        let (rook_from, rook_to) = rook_castling(mv.to);
        g.set_piece(PieceType::Rook, rook_from, g.color_to_move);
        g.unset_piece(PieceType::Rook, rook_to, g.color_to_move);
    } else if let GameMoveType::Promotion(promo_piece, _) = mv.move_type {
        //If promotion, delete promotion piece
        g.unset_piece(promo_piece, mv.to as usize, g.color_to_move);
    } else {
        //Remove piece from to
        g.unset_piece(mv.piece_type, mv.to as usize, g.color_to_move);
    }
    let captured_piece = mv.get_maybe_captured_piece();
    //Add captured piece back
    if let Some(piece) = captured_piece {
        let square = if let GameMoveType::EnPassant = mv.move_type {
            if g.color_to_move == WHITE {
                mv.to - 8
            } else {
                mv.to + 8
            }
        } else {
            mv.to
        };
        g.set_piece(piece, square as usize, 1 - g.color_to_move);
    }
    g.full_moves = g.full_moves - g.color_to_move;

    //Add piece to original square
    g.set_piece(mv.piece_type, mv.from as usize, g.color_to_move);
    g.irreversible = irr;
}
