use crate::board_representation::game_state::{
    GameMove, GameMoveType, GameState, PieceType, WHITE,
};
use crate::board_representation::zobrist_hashing::ZOBRIST_KEYS;
use crate::evaluation::psqt_evaluation::psqt_toggle_piece;

#[inline(always)]
pub fn toggle_piece(pieces: &mut [[u64; 2]; 6], piece: PieceType, square: u8, color: usize) {
    pieces[piece.to_index()][color] ^= 1u64 << square;
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
pub fn castle_hash(
    old: &GameState,
    new_wk: bool,
    new_wq: bool,
    new_bk: bool,
    new_bq: bool,
    hash: &mut u64,
) {
    if old.castle_white_kingside != new_wk {
        *hash ^= ZOBRIST_KEYS.castle_w_kingside;
    }
    if old.castle_white_queenside != new_wq {
        *hash ^= ZOBRIST_KEYS.castle_w_queenside;
    }
    if old.castle_black_kingside != new_bk {
        *hash ^= ZOBRIST_KEYS.castle_b_kingside;
    }
    if old.castle_black_queenside != new_bq {
        *hash ^= ZOBRIST_KEYS.castle_b_queenside;
    }
}

pub fn make_nullmove(g: &GameState) -> GameState {
    let color_to_move = 1 - g.color_to_move;
    let pieces = g.pieces;
    let en_passant = 0u64;
    let half_moves = g.half_moves + 1;
    let full_moves = g.full_moves + g.color_to_move;
    let mut hash = g.hash ^ ZOBRIST_KEYS.side_to_move;
    enpassant_hash(g.en_passant, en_passant, &mut hash);
    let mut res = GameState {
        color_to_move,
        pieces,
        castle_white_kingside: g.castle_white_kingside,
        castle_white_queenside: g.castle_white_queenside,
        castle_black_kingside: g.castle_black_kingside,
        castle_black_queenside: g.castle_black_queenside,
        en_passant,
        half_moves,
        full_moves,
        hash,
        psqt: g.psqt,
        phase: g.phase.clone(),
        checkers: 0u64,
    };
    res.initialize_checkers();
    res
}

#[inline(always)]
pub fn rook_castling(to: u8) -> (u8, u8) {
    if to == 58 {
        (56, 59)
    } else if to == 2 {
        (0, 3)
    } else if to == 62 {
        (63, 61)
    } else if to == 6 {
        (7, 5)
    } else {
        panic!("Invalid castling move!")
    }
}

//We expect the move to be FULLY legal before it can be made!
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
    toggle_piece(&mut pieces, mv.piece_type, mv.from, g.color_to_move);
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
        toggle_piece(&mut pieces, piece, square, color_to_move);
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
        toggle_piece(&mut pieces, mv.piece_type, mv.to, g.color_to_move);
        toggle_hash(mv.piece_type, mv.to, g.color_to_move, &mut hash);
        psqt_toggle_piece(
            &mut pieces,
            mv.piece_type,
            mv.to as usize,
            g.color_to_move,
            &mut psqt,
        );
        let (rook_from, rook_to) = rook_castling(mv.to);
        toggle_piece(&mut pieces, PieceType::Rook, rook_from, g.color_to_move);
        toggle_hash(PieceType::Rook, rook_from, g.color_to_move, &mut hash);
        psqt_toggle_piece(
            &mut pieces,
            PieceType::Rook,
            rook_from as usize,
            g.color_to_move,
            &mut psqt,
        );
        toggle_piece(&mut pieces, PieceType::Rook, rook_to, g.color_to_move);
        toggle_hash(PieceType::Rook, rook_to, g.color_to_move, &mut hash);
        psqt_toggle_piece(
            &mut pieces,
            PieceType::Rook,
            rook_to as usize,
            g.color_to_move,
            &mut psqt,
        );
    } else if let GameMoveType::Promotion(promo_piece, _) = mv.move_type {
        //If promotion, add promotion piece
        toggle_piece(&mut pieces, promo_piece, mv.to, g.color_to_move);
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
        toggle_piece(&mut pieces, mv.piece_type, mv.to, g.color_to_move);
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
    let (
        mut castle_white_kingside,
        mut castle_white_queenside,
        mut castle_black_kingside,
        mut castle_black_queenside,
    ) = (
        g.castle_white_kingside,
        g.castle_white_queenside,
        g.castle_black_kingside,
        g.castle_black_queenside,
    );
    if mv.move_type == GameMoveType::Castle || mv.piece_type == PieceType::King {
        if g.color_to_move == WHITE {
            castle_white_kingside = false;
            castle_white_queenside = false;
        } else {
            castle_black_kingside = false;
            castle_black_queenside = false;
        }
    } else if mv.piece_type == PieceType::Rook {
        if g.color_to_move == WHITE {
            if mv.from == 0 {
                castle_white_queenside = false;
            } else if mv.from == 7 {
                castle_white_kingside = false;
            }
        } else if mv.from == 56 {
            castle_black_queenside = false;
        } else if mv.from == 63 {
            castle_black_kingside = false;
        }
    }
    if captured_piece.is_some() {
        if mv.to == 0 {
            castle_white_queenside = false;
        } else if mv.to == 56 {
            castle_black_queenside = false;
        } else if mv.to == 7 {
            castle_white_kingside = false;
        } else if mv.to == 63 {
            castle_black_kingside = false;
        }
    }
    castle_hash(
        g,
        castle_white_kingside,
        castle_white_queenside,
        castle_black_kingside,
        castle_black_queenside,
        &mut hash,
    );
    //Step 4. Update en passant field
    let en_passant = if mv.move_type == GameMoveType::Quiet
        && mv.piece_type == PieceType::Pawn
        && (mv.to as isize - mv.from as isize).abs() == 16
    {
        if g.color_to_move == WHITE {
            1u64 << (mv.to - 8)
        } else {
            1u64 << (mv.to + 8)
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
    let mut res = GameState {
        color_to_move,
        pieces,
        castle_white_kingside,
        castle_white_queenside,
        castle_black_kingside,
        castle_black_queenside,
        en_passant,
        half_moves,
        full_moves,
        hash,
        psqt,
        phase,
        checkers: 0u64,
    };
    res.initialize_checkers();
    res
}
