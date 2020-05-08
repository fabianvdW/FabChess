use crate::bitboards::bitboards::constants::{square, CASTLE_PERMISSION};
use crate::bitboards::bitboards::square;
use crate::board_representation::game_state::{
    GameMove, GameMoveType, GameState, PieceType, WHITE,
};
use crate::board_representation::zobrist_hashing::ZOBRIST_KEYS;
use crate::evaluation::psqt_evaluation::psqt_toggle_piece;

#[inline(always)]
pub fn toggle_piece(pieces: &mut [[u64; 2]; 6], piece: PieceType, sq: usize, color: usize) {
    pieces[piece as usize][color] ^= square(sq);
}

#[inline(always)]
pub fn toggle_hash(piece: PieceType, square: u8, color: usize, hash: &mut u64) {
    *hash ^= if color == WHITE {
        piece.to_zobrist_key().0
    } else {
        piece.to_zobrist_key().1
    }[square as usize];
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
pub fn castle_hash(old: &GameState, new: u8, hash: &mut u64) {
    *hash ^= ZOBRIST_KEYS.castle_permissions[old.castle_permissions() as usize];
    *hash ^= ZOBRIST_KEYS.castle_permissions[new as usize];
}

pub fn make_nullmove(g: &GameState) -> GameState {
    let color_to_move = 1 - g.color_to_move;
    let pieces = g.pieces;
    let en_passant = 0u64;
    let half_moves = g.half_moves + 1;
    let full_moves = g.full_moves + g.color_to_move;
    let mut hash = g.hash ^ ZOBRIST_KEYS.side_to_move;
    enpassant_hash(g.en_passant, en_passant, &mut hash);
    GameState {
        color_to_move,
        pieces,
        castle_permissions: g.castle_permissions(),
        en_passant,
        half_moves,
        full_moves,
        hash,
        psqt: g.psqt,
        phase: g.phase.clone(),
    }
}

#[inline(always)]
pub fn rook_castling(to: usize) -> (usize, usize) {
    if to == square::C8 {
        (square::A8, square::D8)
    } else if to == square::C1 {
        (square::A1, square::D1)
    } else if to == square::G8 {
        (square::H8, square::F8)
    } else if to == square::G1 {
        (square::H1, square::F1)
    } else {
        panic!("Invalid castling move!")
    }
}

pub fn make_move(g: &GameState, mv: GameMove) -> GameState {
    //Step 1. Update immediate fields
    let color_to_move = 1 - g.color_to_move;
    let full_moves = g.full_moves + g.color_to_move;
    //Step 2. Update pieces, hash and other incremental fields
    let mut pieces = g.pieces;
    let mut hash = g.hash ^ ZOBRIST_KEYS.side_to_move;
    let mut psqt = g.psqt;
    let mut phase = g.phase.clone();
    //Remove piece from original square
    toggle_piece(
        &mut pieces,
        mv.piece_type,
        mv.from as usize,
        g.color_to_move,
    );
    toggle_hash(mv.piece_type, mv.from, g.color_to_move, &mut hash);
    psqt_toggle_piece(
        &mut pieces,
        mv.piece_type,
        mv.from as usize,
        g.color_to_move,
        &mut psqt,
    );
    let captured_piece = match mv.move_type {
        GameMoveType::Capture(c) => Some(c),
        GameMoveType::EnPassant => Some(PieceType::Pawn),
        GameMoveType::Promotion(_, c) => {
            if let Some(c) = c {
                Some(c)
            } else {
                None
            }
        }
        _ => None,
    };
    //Delete piece if capture
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
        toggle_piece(&mut pieces, piece, square as usize, color_to_move);
        toggle_hash(piece, square, color_to_move, &mut hash);
        psqt_toggle_piece(
            &mut pieces,
            piece,
            square as usize,
            color_to_move,
            &mut psqt,
        );
        phase.delete_piece(piece);
    }
    //Move rook for castling
    if let GameMoveType::Castle = mv.move_type {
        toggle_piece(&mut pieces, mv.piece_type, mv.to as usize, g.color_to_move);
        toggle_hash(mv.piece_type, mv.to, g.color_to_move, &mut hash);
        psqt_toggle_piece(
            &mut pieces,
            mv.piece_type,
            mv.to as usize,
            g.color_to_move,
            &mut psqt,
        );
        let (rook_from, rook_to) = rook_castling(mv.to as usize);
        toggle_piece(&mut pieces, PieceType::Rook, rook_from, g.color_to_move);
        toggle_hash(PieceType::Rook, rook_from as u8, g.color_to_move, &mut hash);
        psqt_toggle_piece(
            &mut pieces,
            PieceType::Rook,
            rook_from as usize,
            g.color_to_move,
            &mut psqt,
        );
        toggle_piece(&mut pieces, PieceType::Rook, rook_to, g.color_to_move);
        toggle_hash(PieceType::Rook, rook_to as u8, g.color_to_move, &mut hash);
        psqt_toggle_piece(
            &mut pieces,
            PieceType::Rook,
            rook_to as usize,
            g.color_to_move,
            &mut psqt,
        );
    } else if let GameMoveType::Promotion(promo_piece, _) = mv.move_type {
        //If promotion, add promotion piece
        toggle_piece(&mut pieces, promo_piece, mv.to as usize, g.color_to_move);
        toggle_hash(promo_piece, mv.to, g.color_to_move, &mut hash);
        psqt_toggle_piece(
            &mut pieces,
            promo_piece,
            mv.to as usize,
            g.color_to_move,
            &mut psqt,
        );
        phase.add_piece(promo_piece);
    } else {
        //Add piece again at to
        toggle_piece(&mut pieces, mv.piece_type, mv.to as usize, g.color_to_move);
        toggle_hash(mv.piece_type, mv.to, g.color_to_move, &mut hash);
        psqt_toggle_piece(
            &mut pieces,
            mv.piece_type,
            mv.to as usize,
            g.color_to_move,
            &mut psqt,
        );
    }
    //Step 3. Update Castling Rights
    let castle_permissions = g.castle_permissions()
        & CASTLE_PERMISSION[mv.from as usize]
        & CASTLE_PERMISSION[mv.to as usize];
    castle_hash(g, castle_permissions, &mut hash);
    //Step 4. Update en passant field
    let en_passant = if mv.move_type == GameMoveType::Quiet
        && mv.piece_type == PieceType::Pawn
        && (mv.to as isize - mv.from as isize).abs() == 16
    {
        if g.color_to_move == WHITE {
            square((mv.to - 8) as usize)
        } else {
            square((mv.to + 8) as usize)
        }
    } else {
        0u64
    };
    enpassant_hash(g.en_passant, en_passant, &mut hash);
    //Step 5. Half moves
    let half_moves = if mv.move_type == GameMoveType::Quiet && mv.piece_type != PieceType::Pawn {
        g.half_moves + 1
    } else {
        0
    };
    GameState {
        color_to_move,
        pieces,
        castle_permissions,
        en_passant,
        half_moves,
        full_moves,
        hash,
        psqt,
        phase,
    }
}
