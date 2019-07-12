use super::super::board_representation::zobrist_hashing::ZOBRIST_KEYS;
use crate::bitboards;
use crate::board_representation::game_state::{GameMove, GameMoveType, GameState, PieceType};
use crate::evaluation::psqt_evaluation::{
    psqt_incremental_add_piece, psqt_incremental_delete_piece, psqt_incremental_move_piece,
};

pub fn make_move(g: &GameState, mv: &GameMove) -> GameState {
    match &mv.move_type {
        GameMoveType::Quiet => make_quiet_move(&g, &mv),
        GameMoveType::Capture(piece) => make_capture_move(&g, &mv, &piece),
        GameMoveType::EnPassant => make_enpassant_move(&g, &mv),
        GameMoveType::Castle => make_castle_move(&g, &mv),
        GameMoveType::Promotion(_promoting_to, capturing) => {
            make_promotion_move(&g, &mv, &capturing)
        }
    }
}

#[inline(always)]
pub fn move_piece_hash(move_color: usize, mv: &GameMove, mut hash: u64) -> u64 {
    if move_color == 0 {
        match mv.piece_type {
            PieceType::Pawn => {
                hash ^= ZOBRIST_KEYS.w_pawns[mv.from] ^ ZOBRIST_KEYS.w_pawns[mv.to];
            }
            PieceType::Knight => {
                hash ^= ZOBRIST_KEYS.w_knights[mv.from] ^ ZOBRIST_KEYS.w_knights[mv.to];
            }
            PieceType::Bishop => {
                hash ^= ZOBRIST_KEYS.w_bishops[mv.from] ^ ZOBRIST_KEYS.w_bishops[mv.to];
            }
            PieceType::Rook => {
                hash ^= ZOBRIST_KEYS.w_rooks[mv.from] ^ ZOBRIST_KEYS.w_rooks[mv.to];
            }
            PieceType::Queen => {
                hash ^= ZOBRIST_KEYS.w_queens[mv.from] ^ ZOBRIST_KEYS.w_queens[mv.to];
            }
            PieceType::King => {
                hash ^= ZOBRIST_KEYS.w_king[mv.from] ^ ZOBRIST_KEYS.w_king[mv.to];
            }
        }
    } else {
        match mv.piece_type {
            PieceType::Pawn => {
                hash ^= ZOBRIST_KEYS.b_pawns[mv.from] ^ ZOBRIST_KEYS.b_pawns[mv.to];
            }
            PieceType::Knight => {
                hash ^= ZOBRIST_KEYS.b_knights[mv.from] ^ ZOBRIST_KEYS.b_knights[mv.to];
            }
            PieceType::Bishop => {
                hash ^= ZOBRIST_KEYS.b_bishops[mv.from] ^ ZOBRIST_KEYS.b_bishops[mv.to];
            }
            PieceType::Rook => {
                hash ^= ZOBRIST_KEYS.b_rooks[mv.from] ^ ZOBRIST_KEYS.b_rooks[mv.to];
            }
            PieceType::Queen => {
                hash ^= ZOBRIST_KEYS.b_queens[mv.from] ^ ZOBRIST_KEYS.b_queens[mv.to];
            }
            PieceType::King => {
                hash ^= ZOBRIST_KEYS.b_king[mv.from] ^ ZOBRIST_KEYS.b_king[mv.to];
            }
        }
    }
    hash
}

#[inline(always)]
pub fn enpassant_hash(old_en_passant: u64, new_en_passant: u64, mut hash: u64) -> u64 {
    if old_en_passant != 0u64 {
        hash ^= ZOBRIST_KEYS.en_passant[old_en_passant.trailing_zeros() as usize % 8];
    }
    if new_en_passant != 0u64 {
        hash ^= ZOBRIST_KEYS.en_passant[new_en_passant.trailing_zeros() as usize % 8];
    }
    hash
}

#[inline(always)]
pub fn castle_hash(
    ocwk: bool,
    ocwq: bool,
    ocbk: bool,
    ocbq: bool,
    ncwk: bool,
    ncwq: bool,
    ncbk: bool,
    ncbq: bool,
    mut hash: u64,
) -> u64 {
    if ocwk {
        if !ncwk {
            hash ^= ZOBRIST_KEYS.castle_w_kingside;
        }
    }
    if ocwq {
        if !ncwq {
            hash ^= ZOBRIST_KEYS.castle_w_queenside;
        }
    }
    if ocbk {
        if !ncbk {
            hash ^= ZOBRIST_KEYS.castle_b_kingside;
        }
    }
    if ocbq {
        if !ncbq {
            hash ^= ZOBRIST_KEYS.castle_b_queenside;
        }
    }
    hash
}

#[inline(always)]
pub fn delete_piece_hash(
    delete_square: usize,
    delete_color: usize,
    captured_piece: &PieceType,
    mut hash: u64,
) -> u64 {
    if delete_color == 0 {
        hash ^= match captured_piece {
            PieceType::Pawn => ZOBRIST_KEYS.w_pawns,
            PieceType::Knight => ZOBRIST_KEYS.w_knights,
            PieceType::Bishop => ZOBRIST_KEYS.w_bishops,
            PieceType::Rook => ZOBRIST_KEYS.w_rooks,
            PieceType::Queen => ZOBRIST_KEYS.w_queens,
            PieceType::King => panic!("Can't capture king!"),
        }[delete_square];
    } else {
        hash ^= match captured_piece {
            PieceType::Pawn => ZOBRIST_KEYS.b_pawns,
            PieceType::Knight => ZOBRIST_KEYS.b_knights,
            PieceType::Bishop => ZOBRIST_KEYS.b_bishops,
            PieceType::Rook => ZOBRIST_KEYS.b_rooks,
            PieceType::Queen => ZOBRIST_KEYS.b_queens,
            PieceType::King => panic!("Can't capture king!"),
        }[delete_square];
    }
    hash
}

#[inline(always)]
pub fn move_piece(pieces: &mut [[u64; 2]; 6], mv: &GameMove, move_color: usize) {
    let index = match mv.piece_type {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => 5,
    };
    pieces[index][move_color] ^= bitboards::SQUARES[mv.from];
    pieces[index][move_color] |= bitboards::SQUARES[mv.to];
}

#[inline(always)]
pub fn delete_piece(
    pieces: &mut [[u64; 2]; 6],
    captured_piece: &PieceType,
    delete_square: usize,
    delete_color: usize,
) {
    pieces[match captured_piece {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => panic!("Can't capture king!"),
    }][delete_color] ^= 1u64 << delete_square;
}

pub fn check_castle_flags(
    ck: bool,
    cq: bool,
    mv: &GameMove,
    color_to_move: usize,
    pieces: [[u64; 2]; 6],
) -> (bool, bool) {
    match mv.piece_type {
        PieceType::King => (false, false),
        PieceType::Rook => {
            let mut new_ck = ck;
            if ck {
                if color_to_move == 0 {
                    if pieces[3][0] & bitboards::SQUARES[7] == 0 {
                        new_ck = false;
                    }
                } else {
                    if pieces[3][1] & bitboards::SQUARES[63] == 0 {
                        new_ck = false;
                    }
                }
            }
            let mut new_cq = cq;
            if cq {
                if color_to_move == 0 {
                    if pieces[3][0] & bitboards::SQUARES[0] == 0 {
                        new_cq = false;
                    }
                } else {
                    if pieces[3][1] & bitboards::SQUARES[56] == 0 {
                        new_cq = false;
                    }
                }
            }
            (new_ck, new_cq)
        }
        _ => (ck, cq),
    }
}

pub fn make_nullmove(g: &GameState) -> GameState {
    let color_to_move = 1 - g.color_to_move;
    let pieces = g.pieces.clone();
    let en_passant = 0u64;
    let half_moves = g.half_moves + 1;
    let full_moves = g.full_moves + g.color_to_move;
    let mut hash = g.hash ^ ZOBRIST_KEYS.side_to_move;
    hash = enpassant_hash(g.en_passant, en_passant, hash);
    GameState {
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
        psqt_mg: g.psqt_mg,
        psqt_eg: g.psqt_eg,
    }
}

pub fn make_quiet_move(g: &GameState, mv: &GameMove) -> GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Make the move
    move_piece(&mut pieces, &mv, g.color_to_move);
    //Check new castle rights
    //The enemy's castle rights can't change on a quiet move
    let (castle_white_kingside, castle_white_queenside) = if g.color_to_move == 0 {
        check_castle_flags(
            g.castle_white_kingside,
            g.castle_white_queenside,
            &mv,
            g.color_to_move,
            pieces,
        )
    } else {
        (g.castle_white_kingside, g.castle_white_queenside)
    };
    let (castle_black_kingside, castle_black_queenside) = if g.color_to_move == 0 {
        (g.castle_black_kingside, g.castle_black_queenside)
    } else {
        check_castle_flags(
            g.castle_black_kingside,
            g.castle_black_queenside,
            &mv,
            g.color_to_move,
            pieces,
        )
    };

    let mut en_passant = 0u64;
    let mut half_moves = g.half_moves + 1;

    //Reset half moves if it's a pawn move and also check if it's a double pawn move, if so, set en passant flag
    match mv.piece_type {
        PieceType::Pawn => {
            half_moves = 0;
            if g.color_to_move == 0 && mv.to - mv.from == 16 {
                en_passant = bitboards::SQUARES[mv.to - 8];
            } else if g.color_to_move == 1 && mv.from - mv.to == 16 {
                en_passant = bitboards::SQUARES[mv.to + 8];
            }
        }
        _ => {}
    };

    //If black was to move, increase full moves by one
    let full_moves = g.full_moves + g.color_to_move;
    let mut hash = g.hash ^ ZOBRIST_KEYS.side_to_move;
    hash = move_piece_hash(g.color_to_move, &mv, hash);
    hash = enpassant_hash(g.en_passant, en_passant, hash);
    hash = castle_hash(
        g.castle_white_kingside,
        g.castle_white_queenside,
        g.castle_black_kingside,
        g.castle_black_queenside,
        castle_white_kingside,
        castle_white_queenside,
        castle_black_kingside,
        castle_black_queenside,
        hash,
    );
    let psqt = psqt_incremental_move_piece(
        &mv.piece_type,
        mv.from,
        mv.to,
        g.color_to_move == 1,
        g.psqt_mg,
        g.psqt_eg,
    );
    GameState {
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
        psqt_mg: psqt.0,
        psqt_eg: psqt.1,
    }
}

pub fn make_capture_move(g: &GameState, mv: &GameMove, captured_piece: &PieceType) -> GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Make the move
    move_piece(&mut pieces, &mv, g.color_to_move);
    //Delete destination-piece from enemy pieces
    delete_piece(&mut pieces, &captured_piece, mv.to, color_to_move);

    let (mut castle_white_kingside, mut castle_white_queenside) = if g.color_to_move == 0 {
        check_castle_flags(
            g.castle_white_kingside,
            g.castle_white_queenside,
            &mv,
            g.color_to_move,
            pieces,
        )
    } else {
        (g.castle_white_kingside, g.castle_white_queenside)
    };
    let (mut castle_black_kingside, mut castle_black_queenside) = if g.color_to_move == 0 {
        (g.castle_black_kingside, g.castle_black_queenside)
    } else {
        check_castle_flags(
            g.castle_black_kingside,
            g.castle_black_queenside,
            &mv,
            g.color_to_move,
            pieces,
        )
    };

    if g.color_to_move == 0 {
        //Check that black's rook didn't get captured
        if pieces[3][1] & bitboards::SQUARES[56] == 0 {
            castle_black_queenside = false;
        }
        if pieces[3][1] & bitboards::SQUARES[63] == 0 {
            castle_black_kingside = false;
        }
    } else {
        if pieces[3][0] & bitboards::SQUARES[0] == 0 {
            castle_white_queenside = false;
        }
        if pieces[3][0] & bitboards::SQUARES[7] == 0 {
            castle_white_kingside = false;
        }
    }
    let en_passant = 0u64;

    let half_moves = 0usize;
    let full_moves = g.full_moves + g.color_to_move;
    let mut hash = g.hash ^ ZOBRIST_KEYS.side_to_move;
    hash = move_piece_hash(g.color_to_move, &mv, hash);
    hash = enpassant_hash(g.en_passant, en_passant, hash);
    hash = castle_hash(
        g.castle_white_kingside,
        g.castle_white_queenside,
        g.castle_black_kingside,
        g.castle_black_queenside,
        castle_white_kingside,
        castle_white_queenside,
        castle_black_kingside,
        castle_black_queenside,
        hash,
    );
    hash = delete_piece_hash(mv.to, color_to_move, &captured_piece, hash);
    let psqt = psqt_incremental_move_piece(
        &mv.piece_type,
        mv.from,
        mv.to,
        g.color_to_move == 1,
        g.psqt_mg,
        g.psqt_eg,
    );
    let psqt =
        psqt_incremental_delete_piece(&captured_piece, mv.to, g.color_to_move != 1, psqt.0, psqt.1);
    GameState {
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
        psqt_mg: psqt.0,
        psqt_eg: psqt.1,
    }
}

pub fn make_enpassant_move(g: &GameState, mv: &GameMove) -> GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Make the move
    move_piece(&mut pieces, &mv, g.color_to_move);
    //Delete enemy pawn
    let delete_square = if g.color_to_move == 0 {
        mv.to - 8
    } else {
        mv.to + 8
    };
    delete_piece(&mut pieces, &PieceType::Pawn, delete_square, color_to_move);

    let castle_white_kingside = g.castle_white_kingside;
    let castle_white_queenside = g.castle_white_queenside;
    let castle_black_kingside = g.castle_black_kingside;
    let castle_black_queenside = g.castle_black_queenside;

    let en_passant = 0u64;

    let half_moves = 0usize;
    let full_moves = g.full_moves + g.color_to_move;
    let mut hash = g.hash ^ ZOBRIST_KEYS.side_to_move;
    hash = move_piece_hash(g.color_to_move, &mv, hash);
    hash = enpassant_hash(g.en_passant, en_passant, hash);
    hash = castle_hash(
        g.castle_white_kingside,
        g.castle_white_queenside,
        g.castle_black_kingside,
        g.castle_black_queenside,
        castle_white_kingside,
        castle_white_queenside,
        castle_black_kingside,
        castle_black_queenside,
        hash,
    );
    hash = delete_piece_hash(delete_square, color_to_move, &PieceType::Pawn, hash);
    let psqt = psqt_incremental_move_piece(
        &mv.piece_type,
        mv.from,
        mv.to,
        g.color_to_move == 1,
        g.psqt_mg,
        g.psqt_eg,
    );
    let psqt = psqt_incremental_delete_piece(
        &PieceType::Pawn,
        delete_square,
        g.color_to_move != 1,
        psqt.0,
        psqt.1,
    );
    GameState {
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
        psqt_mg: psqt.0,
        psqt_eg: psqt.1,
    }
}

pub fn make_castle_move(g: &GameState, mv: &GameMove) -> GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Move the king
    move_piece(&mut pieces, &mv, g.color_to_move);

    //Move the rook
    let mut hash = g.hash ^ ZOBRIST_KEYS.side_to_move;
    let rook_zobrist = if g.color_to_move == 0 {
        ZOBRIST_KEYS.w_rooks
    } else {
        ZOBRIST_KEYS.b_rooks
    };
    if mv.to == 58 {
        pieces[3][1] ^= bitboards::SQUARES[56];
        pieces[3][1] |= bitboards::SQUARES[59];
        hash ^= rook_zobrist[56] ^ rook_zobrist[59];
    } else if mv.to == 2 {
        pieces[3][0] ^= bitboards::SQUARES[0];
        pieces[3][0] |= bitboards::SQUARES[3];
        hash ^= rook_zobrist[0] ^ rook_zobrist[3];
    } else if mv.to == 62 {
        pieces[3][1] ^= bitboards::SQUARES[63];
        pieces[3][1] |= bitboards::SQUARES[61];
        hash ^= rook_zobrist[63] ^ rook_zobrist[61];
    } else if mv.to == 6 {
        pieces[3][0] ^= bitboards::SQUARES[7];
        pieces[3][0] |= bitboards::SQUARES[5];
        hash ^= rook_zobrist[7] ^ rook_zobrist[5];
    }

    let (castle_white_kingside, castle_white_queenside) = if g.color_to_move == 0 {
        (false, false)
    } else {
        (g.castle_white_kingside, g.castle_white_queenside)
    };
    let (castle_black_kingside, castle_black_queenside) = if g.color_to_move == 1 {
        (false, false)
    } else {
        (g.castle_black_kingside, g.castle_black_queenside)
    };

    let en_passant = 0u64;

    let half_moves = g.half_moves + 1;
    let full_moves = g.full_moves + g.color_to_move;
    hash = move_piece_hash(g.color_to_move, &mv, hash);
    hash = enpassant_hash(g.en_passant, en_passant, hash);
    hash = castle_hash(
        g.castle_white_kingside,
        g.castle_white_queenside,
        g.castle_black_kingside,
        g.castle_black_queenside,
        castle_white_kingside,
        castle_white_queenside,
        castle_black_kingside,
        castle_black_queenside,
        hash,
    );
    let psqt = psqt_incremental_move_piece(
        &mv.piece_type,
        mv.from,
        mv.to,
        g.color_to_move == 1,
        g.psqt_mg,
        g.psqt_eg,
    );
    let psqt = psqt_incremental_move_piece(
        &PieceType::Rook,
        if mv.to == 58 {
            56
        } else if mv.to == 2 {
            0
        } else if mv.to == 62 {
            63
        } else {
            7
        },
        if mv.to == 58 {
            59
        } else if mv.to == 2 {
            3
        } else if mv.to == 62 {
            61
        } else {
            5
        },
        g.color_to_move == 1,
        psqt.0,
        psqt.1,
    );
    GameState {
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
        psqt_mg: psqt.0,
        psqt_eg: psqt.1,
    }
}

pub fn make_promotion_move(
    g: &GameState,
    mv: &GameMove,
    captured_piece: &Option<PieceType>,
) -> GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    let mut hash = g.hash ^ ZOBRIST_KEYS.side_to_move;
    hash ^= if g.color_to_move == 0 {
        ZOBRIST_KEYS.w_pawns
    } else {
        ZOBRIST_KEYS.b_pawns
    }[mv.from];
    hash ^= match mv.move_type {
        GameMoveType::Promotion(PieceType::Queen, _) => {
            if g.color_to_move == 0 {
                ZOBRIST_KEYS.w_queens
            } else {
                ZOBRIST_KEYS.b_queens
            }
        }
        GameMoveType::Promotion(PieceType::Rook, _) => {
            if g.color_to_move == 0 {
                ZOBRIST_KEYS.w_rooks
            } else {
                ZOBRIST_KEYS.b_rooks
            }
        }
        GameMoveType::Promotion(PieceType::Knight, _) => {
            if g.color_to_move == 0 {
                ZOBRIST_KEYS.w_knights
            } else {
                ZOBRIST_KEYS.b_knights
            }
        }
        GameMoveType::Promotion(PieceType::Bishop, _) => {
            if g.color_to_move == 0 {
                ZOBRIST_KEYS.w_bishops
            } else {
                ZOBRIST_KEYS.b_bishops
            }
        }
        _ => panic!("Invalid Type"),
    }[mv.to];
    move_piece(&mut pieces, &mv, g.color_to_move);
    //Delete enemy piece if any on there
    match captured_piece {
        Some(piece) => {
            delete_piece(&mut pieces, &piece, mv.to, color_to_move);
            hash = delete_piece_hash(mv.to, color_to_move, &piece, hash);
        }
        None => {}
    }
    //Delete my pawn
    pieces[0][g.color_to_move] ^= bitboards::SQUARES[mv.to];
    //Add piece respectively
    pieces[match mv.move_type {
        GameMoveType::Promotion(PieceType::Queen, _) => 4,
        GameMoveType::Promotion(PieceType::Knight, _) => 1,
        GameMoveType::Promotion(PieceType::Bishop, _) => 2,
        GameMoveType::Promotion(PieceType::Rook, _) => 3,
        _ => panic!("Invalid Type"),
    }][g.color_to_move] |= bitboards::SQUARES[mv.to];

    let mut castle_white_kingside = g.castle_white_kingside;
    let mut castle_white_queenside = g.castle_white_queenside;
    let mut castle_black_kingside = g.castle_black_kingside;
    let mut castle_black_queenside = g.castle_black_queenside;

    if g.color_to_move == 0 {
        //Check that black's rook didn't get captured
        if pieces[3][1] & bitboards::SQUARES[56] == 0 {
            castle_black_queenside = false;
        }
        if pieces[3][1] & bitboards::SQUARES[63] == 0 {
            castle_black_kingside = false;
        }
    } else {
        if pieces[3][0] & bitboards::SQUARES[0] == 0 {
            castle_white_queenside = false;
        }
        if pieces[3][0] & bitboards::SQUARES[7] == 0 {
            castle_white_kingside = false;
        }
    }

    let en_passant = 0u64;

    let half_moves = 0usize;
    let full_moves = g.full_moves + g.color_to_move;
    hash = enpassant_hash(g.en_passant, en_passant, hash);
    hash = castle_hash(
        g.castle_white_kingside,
        g.castle_white_queenside,
        g.castle_black_kingside,
        g.castle_black_queenside,
        castle_white_kingside,
        castle_white_queenside,
        castle_black_kingside,
        castle_black_queenside,
        hash,
    );
    let psqt = psqt_incremental_delete_piece(
        &mv.piece_type,
        mv.from,
        g.color_to_move == 1,
        g.psqt_mg,
        g.psqt_eg,
    );
    let mut psqt = psqt_incremental_add_piece(
        &(match mv.move_type {
            GameMoveType::Promotion(typ, _) => typ,
            _ => panic!("Invalid move type in make move promotion"),
        }),
        mv.to,
        g.color_to_move == 1,
        psqt.0,
        psqt.1,
    );
    if let Some(piece) = captured_piece {
        psqt = psqt_incremental_delete_piece(piece, mv.to, g.color_to_move != 1, psqt.0, psqt.1)
    }
    GameState {
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
        psqt_mg: psqt.0,
        psqt_eg: psqt.1,
    }
}
