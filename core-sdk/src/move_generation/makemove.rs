use crate::bitboards::bitboards::constants::{square, CASTLE_PERMISSION};
use crate::bitboards::bitboards::square;
use crate::board_representation::game_state::{ep_pawn_square, file_of, swap_side, GameMove, GameMoveType, GameState, Irreversible, PieceType};
use crate::board_representation::zobrist_hashing::ZOBRIST_KEYS;
use crate::evaluation::psqt_evaluation::{kp_add_piece, kp_move_king, kp_remove_piece, psqt_add_piece, psqt_remove_piece};
use crate::evaluation::EvaluationScore;

#[inline(always)]
pub fn toggle_piece(piece_bb: &mut [u64; 6], color_bb: &mut [u64; 2], piece: PieceType, sq: usize, color: usize, hash: &mut u64) {
    piece_bb[piece as usize] ^= square(sq);
    color_bb[color] ^= square(sq);
    *hash ^= piece.to_zobrist_key(color, sq);
}

#[inline(always)]
pub fn add_piece(piece_bb: &mut [u64; 6], color_bb: &mut [u64; 2], piece: PieceType, sq: usize, color: usize, hash: &mut u64, score: &mut EvaluationScore) {
    toggle_piece(piece_bb, color_bb, piece, sq, color, hash);
    psqt_add_piece(piece, sq, color, score);
}

#[inline(always)]
pub fn remove_piece(piece_bb: &mut [u64; 6], color_bb: &mut [u64; 2], piece: PieceType, sq: usize, color: usize, hash: &mut u64, score: &mut EvaluationScore) {
    toggle_piece(piece_bb, color_bb, piece, sq, color, hash);
    psqt_remove_piece(piece, sq, color, score);
}

#[inline(always)]
pub fn enpassant_hash(old: u64, new: u64, hash: &mut u64) {
    if old != 0u64 {
        *hash ^= ZOBRIST_KEYS.en_passant[file_of(old.trailing_zeros() as usize)];
    }
    if new != 0u64 {
        *hash ^= ZOBRIST_KEYS.en_passant[file_of(new.trailing_zeros() as usize)];
    }
}

#[inline(always)]
pub fn castle_hash(old: &GameState, new: u8, hash: &mut u64) {
    *hash ^= ZOBRIST_KEYS.castle_permissions[old.castle_permissions() as usize];
    *hash ^= ZOBRIST_KEYS.castle_permissions[new as usize];
}

pub fn make_nullmove(g: &GameState) -> GameState {
    let color_to_move = 1 - g.get_color_to_move();
    let piece_bb = g.get_piece_bb_array();
    let color_bb = g.get_color_bb_array();
    let en_passant = 0u64;
    let half_moves = g.get_half_moves() + 1;
    let full_moves = g.get_full_moves() + g.get_color_to_move();
    let mut hash = g.get_hash() ^ ZOBRIST_KEYS.side_to_move;
    enpassant_hash(g.get_en_passant(), en_passant, &mut hash);
    GameState::new(
        color_to_move,
        piece_bb,
        color_bb,
        Irreversible::new(hash, en_passant, half_moves as u16, g.castle_permissions(), g.get_phase().clone(), g.get_psqt()),
        full_moves,
    )
}

#[inline(always)]
pub fn rook_castling(to: usize) -> (usize, usize) {
    match to {
        square::C8 => (square::A8, square::D8),
        square::C1 => (square::A1, square::D1),
        square::G8 => (square::H8, square::F8),
        square::G1 => (square::H1, square::F1),
        _ => panic!("Invalid castling move!"),
    }
}

pub fn make_move(g: &GameState, mv: GameMove) -> GameState {
    //Step 1. Update immediate fields
    let color = g.get_color_to_move();
    let full_moves = g.get_full_moves() + color;
    //Step 2. Update pieces, hash and other incremental fields
    let mut piece_bb = g.get_piece_bb_array();
    let mut color_bb = g.get_color_bb_array();
    let mut hash = g.get_hash() ^ ZOBRIST_KEYS.side_to_move;
    let mut psqt = g.get_psqt();
    let mut phase = g.get_phase().clone();
    let to = mv.to as usize;
    let from = mv.from as usize;
    let mut our_king_square = g.get_king_square(color);
    //Remove piece from original square
    if mv.piece_type == PieceType::King {
        //Update our KP tables
        kp_move_king(
            from,
            to,
            g.get_piece(PieceType::Pawn, color),
            g.get_piece(PieceType::Pawn, swap_side(color)),
            color,
            &mut psqt,
        );
        our_king_square = to;
    } else if mv.piece_type == PieceType::Pawn {
        //Move pawn for enemy king
        kp_remove_piece(swap_side(color), g.get_king_square(swap_side(color)), false, PieceType::Pawn, from, &mut psqt);
        kp_add_piece(swap_side(color), g.get_king_square(swap_side(color)), false, PieceType::Pawn, to, &mut psqt);
        //Move pawn for our king
        kp_remove_piece(color, our_king_square, true, PieceType::Pawn, from, &mut psqt);
        kp_add_piece(color, our_king_square, true, PieceType::Pawn, to, &mut psqt);
    }
    remove_piece(&mut piece_bb, &mut color_bb, mv.piece_type, from, color, &mut hash, &mut psqt);
    //Delete piece if capture
    if let Some(piece) = mv.get_maybe_captured_piece() {
        let square = to ^ (8 * (mv.move_type == GameMoveType::EnPassant) as usize);
        remove_piece(&mut piece_bb, &mut color_bb, piece, square, swap_side(color), &mut hash, &mut psqt);
        phase.delete_piece(piece);
        if piece == PieceType::Pawn {
            //Remove piece for our king
            kp_remove_piece(color, our_king_square, false, PieceType::Pawn, square as usize, &mut psqt);
            //Remove piece for enemy king
            kp_remove_piece(swap_side(color), g.get_king_square(swap_side(color)), true, PieceType::Pawn, square as usize, &mut psqt);
        }
    }
    //Move rook for castling
    if let GameMoveType::Castle = mv.move_type {
        add_piece(&mut piece_bb, &mut color_bb, mv.piece_type, to, color, &mut hash, &mut psqt);
        let (rook_from, rook_to) = rook_castling(to);
        remove_piece(&mut piece_bb, &mut color_bb, PieceType::Rook, rook_from, color, &mut hash, &mut psqt);
        add_piece(&mut piece_bb, &mut color_bb, PieceType::Rook, rook_to, color, &mut hash, &mut psqt);
    } else if let GameMoveType::Promotion(promo, _) = mv.move_type {
        //If promotion, add promotion piece
        add_piece(&mut piece_bb, &mut color_bb, promo, to, color, &mut hash, &mut psqt);
        phase.add_piece(promo);
    } else {
        //Add piece again at to
        add_piece(&mut piece_bb, &mut color_bb, mv.piece_type, to, color, &mut hash, &mut psqt);
    }
    //Step 3. Update Castling Rights
    let castle_permissions = g.castle_permissions() & CASTLE_PERMISSION[from] & CASTLE_PERMISSION[to];
    castle_hash(g, castle_permissions, &mut hash);
    //Step 4. Update en passant field
    let en_passant = if mv.move_type == GameMoveType::Quiet && mv.piece_type == PieceType::Pawn && (to as isize - from as isize).abs() == 16 {
        square(ep_pawn_square(to))
    } else {
        0u64
    };
    enpassant_hash(g.get_en_passant(), en_passant, &mut hash);
    //Step 5. Half moves
    let half_moves = if mv.move_type == GameMoveType::Quiet && mv.piece_type != PieceType::Pawn {
        g.get_half_moves() + 1
    } else {
        0
    };
    GameState::new(
        swap_side(color),
        piece_bb,
        color_bb,
        Irreversible::new(hash, en_passant, half_moves as u16, castle_permissions, phase, psqt),
        full_moves,
    )
}
