use super::super::bitboards;
use super::super::board_representation::game_state::{self, GameMove, GameMoveType, PieceType};
use super::super::board_representation::zobrist_hashing::ZOBRIST_KEYS;
use super::magic::{self, Magic};
use crate::evaluation::psqt_evaluation::{
    psqt_incremental_add_piece, psqt_incremental_delete_piece, psqt_incremental_move_piece,
};

//Move GEn
//King- Piece-Wise by lookup
//Knight-Piece-Wise by lookup
//Bishop/Queen/Rook - Piece-Wise by lookup in Magic
//Pawn-SetWise by shift
#[inline(always)]
pub fn king_attack(square: usize) -> u64 {
    bitboards::KING_ATTACKS[square]
}
#[inline(always)]
pub fn bishop_attack(square: usize, all_pieces: u64) -> u64 {
    Magic::get_attacks(&magic::MAGICS_BISHOPS[square], all_pieces)
}
#[inline(always)]
pub fn rook_attack(square: usize, all_pieces: u64) -> u64 {
    Magic::get_attacks(&magic::MAGICS_ROOKS[square], all_pieces)
}
#[inline(always)]
pub fn knight_attack(square: usize) -> u64 {
    bitboards::KNIGHT_ATTACKS[square]
}
#[inline(always)]
pub fn w_single_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::north_one(pawns) & empty
}
#[inline(always)]
pub fn w_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::north_one(bitboards::north_one(pawns & bitboards::RANKS[1]) & empty) & empty
}
#[inline(always)]
pub fn b_single_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(pawns) & empty
}
#[inline(always)]
pub fn b_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(bitboards::south_one(pawns & bitboards::RANKS[6]) & empty) & empty
}

//NorthEast = +9
#[inline(always)]
pub fn w_pawn_east_targets(pawns: u64) -> u64 {
    bitboards::north_east_one(pawns)
}

//NorthWest = +7
#[inline(always)]
pub fn w_pawn_west_targets(pawns: u64) -> u64 {
    bitboards::north_west_one(pawns)
}

//SouthEast = -7
#[inline(always)]
pub fn b_pawn_east_targets(pawns: u64) -> u64 {
    bitboards::south_west_one(pawns)
}

//NorthWest = -9
#[inline(always)]
pub fn b_pawn_west_targets(pawns: u64) -> u64 {
    bitboards::south_east_one(pawns)
}

#[inline(always)]
pub fn add_moves(
    move_list: &mut Vec<GameMove>,
    from: usize,
    mut to_board: u64,
    piece_type: &PieceType,
    move_type: GameMoveType,
) {
    while to_board != 0u64 {
        let idx = to_board.trailing_zeros() as usize;
        let move_t_cl = move_type.clone();
        let pt_cl = piece_type.clone();
        move_list.push(GameMove {
            from,
            to: idx,
            move_type: move_t_cl,
            piece_type: pt_cl,
        });
        to_board ^= 1u64 << idx;
        //to_board&= to_board-1;
    }
}

#[inline(always)]
pub fn add_capture_moves(
    move_list: &mut Vec<GameMove>,
    from: usize,
    mut to_board: u64,
    piece_type: &PieceType,
    enemy_pawns: u64,
    enemy_knights: u64,
    enemy_bishops: u64,
    enemy_rooks: u64,
    enemy_queens: u64,
) {
    while to_board != 0u64 {
        let idx = to_board.trailing_zeros() as usize;
        let pt_cl = piece_type.clone();
        move_list.push(GameMove {
            from,
            to: idx,
            move_type: GameMoveType::Capture(find_captured_piece_type(
                idx,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            )),
            piece_type: pt_cl,
        });
        to_board ^= 1u64 << idx;
    }
}

//Make moves
pub fn make_move(g: &game_state::GameState, mv: &game_state::GameMove) -> game_state::GameState {
    //println!("Making move: {:?}", mv);
    //println!("Move is made on state:\n {}", g);
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
pub fn move_piece_hash(move_color: usize, mv: &game_state::GameMove, mut hash: u64) -> u64 {
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
pub fn move_piece(pieces: &mut [[u64; 2]; 6], mv: &game_state::GameMove, move_color: usize) {
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
    //pieces[index][move_color] ^= 1u64<<mv.from;
    //pieces[index][move_color] |= 1u64<<mv.to;
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
    mv: &game_state::GameMove,
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

pub fn make_nullmove(g: &game_state::GameState) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let pieces = g.pieces.clone();
    let en_passant = 0u64;
    let half_moves = g.half_moves + 1;
    let full_moves = g.full_moves + g.color_to_move;
    let mut hash = g.hash ^ ZOBRIST_KEYS.side_to_move;
    hash = enpassant_hash(g.en_passant, en_passant, hash);
    game_state::GameState {
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

pub fn make_quiet_move(
    g: &game_state::GameState,
    mv: &game_state::GameMove,
) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Make the move
    move_piece(&mut pieces, &mv, g.color_to_move);
    //Check new castle rights
    //The enemies castle right's can't change on a quiet move
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

    //Reset half moves if its a pawn move and also check if its a double pawn move, if so, set en passant flag
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
    game_state::GameState {
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

pub fn make_capture_move(
    g: &game_state::GameState,
    mv: &game_state::GameMove,
    captured_piece: &PieceType,
) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Make the move
    move_piece(&mut pieces, &mv, g.color_to_move);
    //Delete to from enemy pieces
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
        //Check that blacks rook didn't get captured
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
    game_state::GameState {
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

pub fn make_enpassant_move(
    g: &game_state::GameState,
    mv: &game_state::GameMove,
) -> game_state::GameState {
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
    game_state::GameState {
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

pub fn make_castle_move(
    g: &game_state::GameState,
    mv: &game_state::GameMove,
) -> game_state::GameState {
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
    game_state::GameState {
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
    g: &game_state::GameState,
    mv: &game_state::GameMove,
    captured_piece: &Option<PieceType>,
) -> game_state::GameState {
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
    //Add piece respectivly
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
        //Check that blacks rook didn't get captured
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
    game_state::GameState {
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

pub fn generate_moves(g: &game_state::GameState) -> (Vec<GameMove>, bool) {
    //Following this guide:
    // https://peterellisjones.com/posts/generating-legal-chess-moves-efficiently/

    let mut move_list: Vec<GameMove> = Vec::with_capacity(60);
    let color_to_move = g.color_to_move;
    let enemy_color = 1 - color_to_move;

    //Get my pieces
    let my_king = g.pieces[5][color_to_move];
    let my_king_idx = my_king.trailing_zeros() as usize;

    let mut my_pawns = g.pieces[0][color_to_move];

    let mut my_knights = g.pieces[1][color_to_move];

    let mut my_bishops = g.pieces[2][color_to_move] | g.pieces[4][color_to_move];

    let mut my_rooks = g.pieces[3][color_to_move] | g.pieces[4][color_to_move];
    //Need this to xor out queens later, when we look at pinned pieces
    let my_queens = g.pieces[4][color_to_move];

    //Get enemy pieces
    let enemy_king = g.pieces[5][enemy_color];
    let enemy_king_idx = enemy_king.trailing_zeros() as usize;

    let enemy_pawns = g.pieces[0][enemy_color];

    let enemy_knights = g.pieces[1][enemy_color];

    let enemy_bishops = g.pieces[2][enemy_color] | g.pieces[4][enemy_color];

    let enemy_rooks = g.pieces[3][enemy_color] | g.pieces[4][enemy_color];

    let enemy_queens = g.pieces[4][enemy_color];

    let my_pieces = my_king | my_pawns | my_knights | my_bishops | my_rooks;
    let not_my_pieces = !my_pieces;
    let enemy_sliders = enemy_bishops | enemy_rooks;
    let enemy_pieces = enemy_pawns | enemy_knights | enemy_sliders | enemy_king;
    let not_enemy_pieces = !enemy_pieces;
    let all_pieces_without_my_king = enemy_pieces | (my_pieces & !my_king);
    let all_pieces = all_pieces_without_my_king | my_king;
    let empty = !all_pieces;

    let unsafe_white_squares = if color_to_move == 0 {
        get_b_attacked_squares(
            enemy_king_idx,
            enemy_pawns,
            enemy_knights,
            enemy_bishops,
            enemy_rooks,
            all_pieces_without_my_king,
        )
    } else {
        get_w_attacked_squares(
            enemy_king_idx,
            enemy_pawns,
            enemy_knights,
            enemy_bishops,
            enemy_rooks,
            all_pieces_without_my_king,
        )
    };
    let possible_king_moves = king_attack(my_king_idx) & !unsafe_white_squares & not_my_pieces;
    add_moves(
        &mut move_list,
        my_king_idx,
        possible_king_moves & not_enemy_pieces,
        &PieceType::King,
        GameMoveType::Quiet,
    );
    add_capture_moves(
        &mut move_list,
        my_king_idx,
        possible_king_moves & enemy_pieces,
        &PieceType::King,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
    );
    let (king_attackers_board, checking_piece_is_slider, checking_piece_slider_is_bishop) =
        if unsafe_white_squares & my_king == 0 {
            (0u64, false, false)
        } else {
            if color_to_move == 0 {
                attackers_from_black(
                    my_king,
                    my_king_idx,
                    enemy_pawns,
                    enemy_knights,
                    enemy_bishops,
                    enemy_rooks,
                    all_pieces,
                )
            } else {
                attackers_from_white(
                    my_king,
                    my_king_idx,
                    enemy_pawns,
                    enemy_knights,
                    enemy_bishops,
                    enemy_rooks,
                    all_pieces,
                )
            }
        };

    let num_checkers = king_attackers_board.count_ones();
    //Double check
    if num_checkers > 1 {
        //Then only king moves are possible anyway
        return (move_list, true);
    }
    //Calculate capture and push mask
    let mut capture_mask = 0xFFFFFFFFFFFFFFFFu64;
    let mut push_mask = 0xFFFFFFFFFFFFFFFFu64;
    //Single-Check
    {
        if num_checkers == 1 {
            capture_mask = king_attackers_board;
            if checking_piece_is_slider {
                let checking_piece_square = king_attackers_board.trailing_zeros() as usize;
                if checking_piece_slider_is_bishop {
                    push_mask = get_bishop_ray(my_king_idx, checking_piece_square);
                } else {
                    push_mask = get_rook_ray(my_king_idx, checking_piece_square);
                }
            } else {
                push_mask = 0u64;
            }
        }
    }

    //Pinned pieces
    {
        //Pinned by rook
        let rook_attacks_from_my_king_postion = rook_attack(my_king_idx, all_pieces);
        //See one layer through my pieces
        //If a rook is found seeing through one piece, the piece is pinned
        let xray_rooks = xray_rook_attacks(
            rook_attacks_from_my_king_postion,
            all_pieces,
            my_pieces,
            my_king_idx,
        );
        //Go through all directions
        //Find the rooks with xray
        let mut rooks_on_xray = xray_rooks & enemy_rooks;
        while rooks_on_xray != 0 {
            let rook_position = rooks_on_xray.trailing_zeros() as usize;
            rooks_on_xray &= rooks_on_xray - 1;
            let ray = get_rook_ray(my_king_idx, rook_position);
            let pinned_piece = ray & my_pieces;
            let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
            if pinned_piece & my_rooks != 0 {
                let mut piece_type = PieceType::Rook;
                if pinned_piece & my_queens != 0 {
                    my_bishops ^= pinned_piece;
                    piece_type = PieceType::Queen;
                }
                my_rooks ^= pinned_piece;
                add_moves(
                    &mut move_list,
                    pinned_piece_position,
                    ray & !pinned_piece & push_mask,
                    &piece_type,
                    GameMoveType::Quiet,
                );
                //Don't forget that we can capture the rook aswell
                add_capture_moves(
                    &mut move_list,
                    pinned_piece_position,
                    bitboards::SQUARES[rook_position] & capture_mask,
                    &piece_type,
                    enemy_pawns,
                    enemy_knights,
                    enemy_bishops,
                    enemy_rooks,
                    enemy_queens,
                );
                continue;
            }
            if pinned_piece & my_pawns != 0 {
                my_pawns ^= pinned_piece;
                let pawn_push_once = if color_to_move == 0 {
                    w_single_push_pawn_targets(pinned_piece, empty)
                } else {
                    b_single_push_pawn_targets(pinned_piece, empty)
                } & ray;
                let pawn_push_twice = if color_to_move == 0 {
                    w_double_push_pawn_targets(pinned_piece, empty)
                } else {
                    b_double_push_pawn_targets(pinned_piece, empty)
                } & ray;
                add_moves(
                    &mut move_list,
                    pinned_piece_position,
                    (pawn_push_once | pawn_push_twice) & push_mask,
                    &PieceType::Pawn,
                    GameMoveType::Quiet,
                );
                continue;
            }
            if pinned_piece & my_knights != 0 {
                my_knights ^= pinned_piece;
                continue;
            }
            if pinned_piece & my_bishops != 0 {
                my_bishops ^= pinned_piece;
            }
        }
        //Pinned by bishop
        let bishop_attacks_from_my_king_position = bishop_attack(my_king_idx, all_pieces);
        let xray_bishops = xray_bishop_attacks(
            bishop_attacks_from_my_king_position,
            all_pieces,
            my_pieces,
            my_king_idx,
        );
        let mut bishops_on_xray = xray_bishops & enemy_bishops;
        while bishops_on_xray != 0 {
            let bishop_position = bishops_on_xray.trailing_zeros() as usize;
            bishops_on_xray &= bishops_on_xray - 1;
            let ray = get_bishop_ray(my_king_idx, bishop_position);
            let pinned_piece = ray & my_pieces;
            let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
            if pinned_piece & my_bishops != 0 {
                let mut piece_type = PieceType::Bishop;
                if pinned_piece & my_queens != 0 {
                    my_rooks ^= pinned_piece;
                    piece_type = PieceType::Queen;
                }
                my_bishops ^= pinned_piece;

                add_moves(
                    &mut move_list,
                    pinned_piece_position,
                    ray & !pinned_piece & push_mask,
                    &piece_type,
                    GameMoveType::Quiet,
                );
                add_capture_moves(
                    &mut move_list,
                    pinned_piece_position,
                    bitboards::SQUARES[bishop_position] & capture_mask,
                    &piece_type,
                    enemy_pawns,
                    enemy_knights,
                    enemy_bishops,
                    enemy_rooks,
                    enemy_queens,
                );
                continue;
            }
            if pinned_piece & my_pawns != 0 {
                my_pawns ^= pinned_piece;
                let pawn_targets = if color_to_move == 0 {
                    w_pawn_east_targets(pinned_piece)
                } else {
                    b_pawn_east_targets(pinned_piece)
                } | if color_to_move == 0 {
                    w_pawn_west_targets(pinned_piece)
                } else {
                    b_pawn_west_targets(pinned_piece)
                };
                let pawn_captures =
                    pawn_targets & bitboards::SQUARES[bishop_position] & capture_mask;
                let pawn_normal_captures =
                    pawn_captures & !bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
                let pawn_promotion_captures =
                    pawn_captures & bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
                let source_shift = pawn_promotion_captures.trailing_zeros() as isize
                    - pinned_piece_position as isize;
                //println!("{}",misc::to_string_board(pawn_targets));
                let pawn_enpassants = pawn_targets
                    & g.en_passant
                    & capture_mask
                    & (bitboards::SQUARES[bishop_position] | ray);
                add_capture_moves(
                    &mut move_list,
                    pinned_piece_position,
                    pawn_normal_captures,
                    &PieceType::Pawn,
                    enemy_pawns,
                    enemy_knights,
                    enemy_bishops,
                    enemy_rooks,
                    enemy_queens,
                );
                add_promotion_capture(
                    pawn_promotion_captures,
                    &color_to_move,
                    &mut move_list,
                    (source_shift * if color_to_move == 0 { 1 } else { -1 }) as usize,
                    enemy_pawns,
                    enemy_knights,
                    enemy_bishops,
                    enemy_rooks,
                    enemy_queens,
                );
                add_moves(
                    &mut move_list,
                    pinned_piece_position,
                    pawn_enpassants,
                    &PieceType::Pawn,
                    GameMoveType::EnPassant,
                );
                continue;
            }
            if pinned_piece & my_knights != 0 {
                my_knights ^= pinned_piece;
                continue;
            }
            if pinned_piece & my_rooks != 0 {
                my_rooks ^= pinned_piece;
            }
        }
    }
    //Calculate normal moves
    //Pawns
    {
        //Single push
        {
            let my_pawns_single_push = if color_to_move == 0 {
                w_single_push_pawn_targets(my_pawns, empty)
            } else {
                b_single_push_pawn_targets(my_pawns, empty)
            } & push_mask;
            let my_pawns_no_promotion =
                my_pawns_single_push & !bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            let my_pawns_promotion =
                my_pawns_single_push & bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            add_pawn_single_pushes(my_pawns_no_promotion, &color_to_move, &mut move_list);
            add_promotion_push(my_pawns_promotion, &color_to_move, &mut move_list);
        }
        //Double push
        {
            let my_pawns_double_push = if color_to_move == 0 {
                w_double_push_pawn_targets(my_pawns, empty)
            } else {
                b_double_push_pawn_targets(my_pawns, empty)
            } & push_mask;
            add_pawn_double_pushes(my_pawns_double_push, &color_to_move, &mut move_list);
        }
        //Capture west
        {
            let my_pawns_west_targets = if color_to_move == 0 {
                w_pawn_west_targets(my_pawns)
            } else {
                b_pawn_west_targets(my_pawns)
            };
            let my_pawns_west_normal_captures = my_pawns_west_targets & capture_mask & enemy_pieces;
            //Checking for promotion on capture
            let my_pawns_no_promotion = my_pawns_west_normal_captures
                & !bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            let my_pawns_promotion = my_pawns_west_normal_captures
                & bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            //Capture
            add_pawn_capture(
                my_pawns_no_promotion,
                &color_to_move,
                &mut move_list,
                7usize,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            );
            //Promotion capture
            add_promotion_capture(
                my_pawns_promotion,
                &color_to_move,
                &mut move_list,
                7usize,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            );

            //En passant
            //We can capture en passant, if its in capture mask aswell
            let my_pawns_west_enpassants = my_pawns_west_targets
                & g.en_passant
                & if color_to_move == 0 {
                    capture_mask << 8
                } else {
                    capture_mask >> 8
                };
            add_en_passants(
                my_pawns_west_enpassants,
                &color_to_move,
                &mut move_list,
                7usize,
                all_pieces_without_my_king,
                enemy_rooks,
                my_king_idx,
            );
        }
        //Capture east
        {
            let my_pawns_east_targets = if color_to_move == 0 {
                w_pawn_east_targets(my_pawns)
            } else {
                b_pawn_east_targets(my_pawns)
            };
            let my_pawns_east_normal_captures = my_pawns_east_targets & capture_mask & enemy_pieces;
            //Checking for promotion on capture
            let my_pawns_no_promotion = my_pawns_east_normal_captures
                & !bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            let my_pawns_promotion = my_pawns_east_normal_captures
                & bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            add_pawn_capture(
                my_pawns_no_promotion,
                &color_to_move,
                &mut move_list,
                9usize,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            );
            add_promotion_capture(
                my_pawns_promotion,
                &color_to_move,
                &mut move_list,
                9usize,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            );
            //En passants
            let my_pawns_east_enpassants = my_pawns_east_targets
                & g.en_passant
                & if color_to_move == 0 {
                    capture_mask << 8
                } else {
                    capture_mask >> 8
                };
            add_en_passants(
                my_pawns_east_enpassants,
                &color_to_move,
                &mut move_list,
                9usize,
                all_pieces_without_my_king,
                enemy_rooks,
                my_king_idx,
            );
        }
    }
    //Knights
    {
        while my_knights != 0u64 {
            let index = if color_to_move == 0 {
                63usize - my_knights.leading_zeros() as usize
            } else {
                my_knights.trailing_zeros() as usize
            };
            let my_knight_attacks = knight_attack(index) & not_my_pieces;
            let my_knight_captures = my_knight_attacks & enemy_pieces & capture_mask;
            add_capture_moves(
                &mut move_list,
                index,
                my_knight_captures,
                &PieceType::Knight,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            );
            let my_knight_quiets = my_knight_attacks & !enemy_pieces & push_mask;
            add_moves(
                &mut move_list,
                index,
                my_knight_quiets,
                &PieceType::Knight,
                GameMoveType::Quiet,
            );
            my_knights ^= 1u64 << index;
        }
    }
    //Bishops
    {
        while my_bishops != 0u64 {
            let index = if color_to_move == 0 {
                63usize - my_bishops.leading_zeros() as usize
            } else {
                my_bishops.trailing_zeros() as usize
            };
            let piece = 1u64 << index;
            let my_bishop_attack = bishop_attack(index, all_pieces) & not_my_pieces;
            let my_bishop_capture = my_bishop_attack & enemy_pieces & capture_mask;
            let piece_type = if piece & my_queens != 0 {
                PieceType::Queen
            } else {
                PieceType::Bishop
            };
            add_capture_moves(
                &mut move_list,
                index,
                my_bishop_capture,
                &piece_type,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            );
            let my_bishop_quiet = my_bishop_attack & !enemy_pieces & push_mask;
            add_moves(
                &mut move_list,
                index,
                my_bishop_quiet,
                &piece_type,
                GameMoveType::Quiet,
            );
            my_bishops ^= piece;
        }
    }
    //Rooks
    {
        while my_rooks != 0u64 {
            let index = if color_to_move == 0 {
                63usize - my_rooks.leading_zeros() as usize
            } else {
                my_rooks.trailing_zeros() as usize
            };
            let piece = 1u64 << index;
            let my_rook_attack = rook_attack(index, all_pieces) & not_my_pieces;
            let my_rook_capture = my_rook_attack & enemy_pieces & capture_mask;
            let piece_type = if piece & my_queens != 0 {
                PieceType::Queen
            } else {
                PieceType::Rook
            };
            add_capture_moves(
                &mut move_list,
                index,
                my_rook_capture,
                &piece_type,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            );
            let my_rook_quiets = my_rook_attack & !enemy_pieces & push_mask;
            add_moves(
                &mut move_list,
                index,
                my_rook_quiets,
                &piece_type,
                GameMoveType::Quiet,
            );
            my_rooks ^= piece;
        }
    }
    //Castles
    if num_checkers == 0 {
        if g.color_to_move == 0 {
            //Make sure there is no piece in between and safe squares
            if g.castle_white_kingside {
                if (all_pieces | unsafe_white_squares)
                    & (bitboards::SQUARES[5] | bitboards::SQUARES[6])
                    == 0
                {
                    move_list.push(GameMove {
                        from: my_king_idx,
                        to: 6usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
            if g.castle_white_queenside {
                if ((all_pieces | unsafe_white_squares)
                    & (bitboards::SQUARES[2] | bitboards::SQUARES[3])
                    | all_pieces & bitboards::SQUARES[1])
                    == 0
                {
                    move_list.push(GameMove {
                        from: my_king_idx,
                        to: 2usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
        } else {
            if g.castle_black_kingside {
                if (all_pieces | unsafe_white_squares)
                    & (bitboards::SQUARES[61] | bitboards::SQUARES[62])
                    == 0
                {
                    move_list.push(GameMove {
                        from: my_king_idx,
                        to: 62usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
            if g.castle_black_queenside {
                if ((all_pieces | unsafe_white_squares)
                    & (bitboards::SQUARES[58] | bitboards::SQUARES[59])
                    | all_pieces & bitboards::SQUARES[57])
                    == 0
                {
                    move_list.push(GameMove {
                        from: my_king_idx,
                        to: 58usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
        }
    }
    (move_list, num_checkers > 0)
}

#[inline(always)]
pub fn find_captured_piece_type(
    to: usize,
    e_pawns: u64,
    e_knights: u64,
    e_bishops: u64,
    e_rooks: u64,
    e_queens: u64,
) -> PieceType {
    let to_board = 1u64 << to;
    if e_pawns & to_board != 0u64 {
        PieceType::Pawn
    } else if e_knights & to_board != 0u64 {
        PieceType::Knight
    //Find queens before bishops and rooks since it's in all three boards.
    } else if e_queens & to_board != 0u64 {
        PieceType::Queen
    } else if e_bishops & to_board != 0u64 {
        PieceType::Bishop
    } else if e_rooks & to_board != 0u64 {
        PieceType::Rook
    } else {
        panic!("Shoudln't get here");
    }
}

#[inline(always)]
pub fn add_pawn_single_pushes(
    mut single_push_board: u64,
    color_to_move: &usize,
    move_list: &mut Vec<GameMove>,
) {
    while single_push_board != 0u64 {
        let idx = single_push_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - 8
            } else {
                idx + 8
            },
            to: idx,
            move_type: GameMoveType::Quiet,
            piece_type: PieceType::Pawn,
        });
        single_push_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_pawn_double_pushes(
    mut double_push_board: u64,
    color_to_move: &usize,
    move_list: &mut Vec<GameMove>,
) {
    while double_push_board != 0u64 {
        let idx = double_push_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - 16
            } else {
                idx + 16
            },
            to: idx,
            move_type: GameMoveType::Quiet,
            piece_type: PieceType::Pawn,
        });
        double_push_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_promotion_push(
    mut promotion_board: u64,
    color_to_move: &usize,
    move_list: &mut Vec<GameMove>,
) {
    while promotion_board != 0u64 {
        let idx = promotion_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - 8
            } else {
                idx + 8
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Queen, None),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - 8
            } else {
                idx + 8
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Rook, None),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - 8
            } else {
                idx + 8
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Bishop, None),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - 8
            } else {
                idx + 8
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Knight, None),
            piece_type: PieceType::Pawn,
        });
        promotion_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_promotion_capture(
    mut promotion_board: u64,
    color_to_move: &usize,
    move_list: &mut Vec<GameMove>,
    source_shift: usize,
    enemy_pawns: u64,
    enemy_knights: u64,
    enemy_bishops: u64,
    enemy_rooks: u64,
    enemy_queens: u64,
) {
    while promotion_board != 0u64 {
        let idx = promotion_board.trailing_zeros() as usize;
        let x: Option<PieceType> = Some(find_captured_piece_type(
            idx,
            enemy_pawns,
            enemy_knights,
            enemy_bishops,
            enemy_rooks,
            enemy_queens,
        ));
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - source_shift
            } else {
                idx + source_shift
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Queen, x.clone()),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - source_shift
            } else {
                idx + source_shift
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Rook, x.clone()),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - source_shift
            } else {
                idx + source_shift
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Bishop, x.clone()),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - source_shift
            } else {
                idx + source_shift
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Knight, x),
            piece_type: PieceType::Pawn,
        });
        promotion_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_pawn_capture(
    mut capture_board: u64,
    color_to_move: &usize,
    move_list: &mut Vec<GameMove>,
    source_shift: usize,
    enemy_pawns: u64,
    enemy_knights: u64,
    enemy_bishops: u64,
    enemy_rooks: u64,
    enemy_queens: u64,
) {
    while capture_board != 0u64 {
        let idx = capture_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - source_shift
            } else {
                idx + source_shift
            },
            to: idx,
            move_type: GameMoveType::Capture(find_captured_piece_type(
                idx,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            )),
            piece_type: PieceType::Pawn,
        });
        capture_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_en_passants(
    mut enpassant_board: u64,
    color_to_move: &usize,
    move_list: &mut Vec<GameMove>,
    source_shift: usize,
    all_pieces_without_my_king: u64,
    enemy_rooks: u64,
    my_king_idx: usize,
) {
    while enpassant_board != 0u64 {
        let index = enpassant_board.trailing_zeros() as usize;
        enpassant_board ^= 1u64 << index;
        //Check if rare case didn't happen
        //Remove t-7,t-8 or t+7,t+8
        let all_pieces_without_en_passants = all_pieces_without_my_king
            & !bitboards::SQUARES[if *color_to_move == 0 {
                index - source_shift
            } else {
                index + source_shift
            }]
            & !bitboards::SQUARES[if *color_to_move == 0 {
                index - 8
            } else {
                index + 8
            }];
        if rook_attack(my_king_idx, all_pieces_without_en_passants)
            & (!bitboards::FILES[my_king_idx % 8])
            & enemy_rooks
            != 0
        {
            continue;
        }
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                index - source_shift
            } else {
                index + source_shift
            },
            to: index,
            move_type: GameMoveType::EnPassant,
            piece_type: PieceType::Pawn,
        });
    }
}

#[inline(always)]
pub fn xray_rook_attacks(
    rook_attacks: u64,
    occupied_squares: u64,
    my_pieces: u64,
    rook_square: usize,
) -> u64 {
    return rook_attacks ^ rook_attack(rook_square, occupied_squares ^ (my_pieces & rook_attacks));
}
#[inline(always)]
pub fn xray_bishop_attacks(
    bishop_attacks: u64,
    occupied_squares: u64,
    my_pieces: u64,
    bishop_square: usize,
) -> u64 {
    return bishop_attacks
        ^ bishop_attack(
            bishop_square,
            occupied_squares ^ (my_pieces & bishop_attacks),
        );
}
#[inline(always)]
pub fn get_rook_ray(king_square: usize, rook_square: usize) -> u64 {
    bitboards::ROOK_RAYS[king_square][rook_square]
}

#[inline(always)]
pub fn get_bishop_ray(king_square: usize, bishop_square: usize) -> u64 {
    bitboards::BISHOP_RAYS[king_square][bishop_square]
}

pub fn attackers_from_white(
    square_board: u64,
    square: usize,
    white_pawns: u64,
    white_knights: u64,
    white_bishops: u64,
    white_rooks: u64,
    blockers: u64,
) -> (u64, bool, bool) {
    let mut attackers = 0u64;
    let mut slider_flag = false;
    let mut bishop_slider = false;
    attackers |= knight_attack(square) & white_knights;
    attackers |=
        (b_pawn_west_targets(square_board) | b_pawn_east_targets(square_board)) & white_pawns;
    let bishop_attacks = bishop_attack(square, blockers) & white_bishops;
    attackers |= bishop_attacks;
    if bishop_attacks != 0 {
        slider_flag = true;
        bishop_slider = true;
    }
    let rook_attacks = rook_attack(square, blockers) & white_rooks;
    attackers |= rook_attacks;
    if rook_attacks != 0 {
        slider_flag = true;
    }
    (attackers, slider_flag, bishop_slider)
}

pub fn attackers_from_black(
    square_board: u64,
    square: usize,
    black_pawns: u64,
    black_knights: u64,
    black_bishops: u64,
    black_rooks: u64,
    blockers: u64,
) -> (u64, bool, bool) {
    let mut attackers = 0u64;
    let mut slider_flag = false;
    let mut bishop_slider = false;
    attackers |= knight_attack(square) & black_knights;
    attackers |=
        (w_pawn_west_targets(square_board) | w_pawn_east_targets(square_board)) & black_pawns;
    let bishop_attacks = bishop_attack(square, blockers) & black_bishops;
    attackers |= bishop_attacks;
    if bishop_attacks != 0 {
        slider_flag = true;
        bishop_slider = true;
    }
    let rook_attacks = rook_attack(square, blockers) & black_rooks;
    attackers |= rook_attacks;
    if rook_attacks != 0 {
        slider_flag = true;
    }
    (attackers, slider_flag, bishop_slider)
}

pub fn get_w_attacked_squares(
    white_king_idx: usize,
    white_pawns: u64,
    mut white_knights: u64,
    mut white_bishops: u64,
    mut white_rooks: u64,
    blocked_squares: u64,
) -> u64 {
    let mut res = 0u64;
    res |= king_attack(white_king_idx);
    res |= w_pawn_west_targets(white_pawns) | w_pawn_east_targets(white_pawns);
    while white_knights != 0u64 {
        let sq = 63usize - white_knights.leading_zeros() as usize;
        res |= knight_attack(sq);
        white_knights ^= 1u64 << sq;
    }
    while white_bishops != 0u64 {
        let sq = 63usize - white_bishops.leading_zeros() as usize;
        res |= bishop_attack(sq, blocked_squares);
        white_bishops ^= 1u64 << sq;
    }
    while white_rooks != 0u64 {
        let sq = 63usize - white_rooks.leading_zeros() as usize;
        res |= rook_attack(sq, blocked_squares);
        white_rooks ^= 1u64 << sq;
    }
    res
}

pub fn get_b_attacked_squares(
    black_king_idx: usize,
    black_pawns: u64,
    mut black_knights: u64,
    mut black_bishops: u64,
    mut black_rooks: u64,
    blocked_squares: u64,
) -> u64 {
    let mut res = 0u64;
    res |= king_attack(black_king_idx);
    res |= b_pawn_west_targets(black_pawns) | b_pawn_east_targets(black_pawns);
    while black_knights != 0u64 {
        let sq = black_knights.trailing_zeros() as usize;
        res |= knight_attack(sq);
        black_knights ^= 1u64 << sq;
    }
    while black_bishops != 0u64 {
        let sq = black_bishops.trailing_zeros() as usize;
        res |= bishop_attack(sq, blocked_squares);
        black_bishops ^= 1u64 << sq;
    }
    while black_rooks != 0u64 {
        let sq = black_rooks.trailing_zeros() as usize;
        res |= rook_attack(sq, blocked_squares);
        black_rooks ^= 1u64 << sq;
    }
    res
}

pub struct AdditionalBitBoards {
    stm_pieces_without_king: u64,
    stm_pieces: u64,
    enemy_pieces: u64,
    all_pieces_without_stmking: u64,
    all_pieces: u64,

    stm_pawns_westattack: u64,
    stm_pawns_eastattack: u64,
    enemy_pawns_westattack: u64,
    enemy_pawns_eastattack: u64,

    stm_king_attacks: u64,
    enemy_king_attacks: u64,

    stm_unsafe_squares: u64,
    all_checkers: u64,
}

/*#[inline(always)]
pub fn calculate_additionalbitboards_g(
    g: &game_state::GameState,
) -> AdditionalBitBoards {
    let stm_color = g.color_to_move;
    let enemy_color = 1 - stm_color;
    let stm_color_iswhite: bool = g.color_to_move == 0;

    let stm_pawns: u64 = g.pieces[0][stm_color];
    let stm_knights: u64 = g.pieces[1][stm_color];
    let stm_bishops: u64 = g.pieces[2][stm_color];
    let stm_rooks: u64 = g.pieces[3][stm_color];
    let stm_queens: u64 = g.pieces[4][stm_color];
    let stm_king: u64 = g.pieces[5][stm_color];

    let enemy_pawns: u64 = g.pieces[0][enemy_color];
    let enemy_knights: u64 = g.pieces[1][enemy_color];
    let enemy_bishops: u64 = g.pieces[2][enemy_color];
    let enemy_rooks: u64 = g.pieces[3][enemy_color];
    let enemy_queens: u64 = g.pieces[4][enemy_color];
    let enemy_king: u64 = g.pieces[5][enemy_color];
    calculate_additionalbitboards(
        stm_pawns,
        enemy_pawns,
        stm_knights,
        enemy_knights,
        stm_bishops,
        enemy_bishops,
        stm_rooks,
        enemy_rooks,
        stm_queens,
        enemy_queens,
        stm_king,
        enemy_king,
        stm_color_iswhite
    )
}*/

pub fn calculate_additionalbitboards(
    stm_pawns: u64,
    enemy_pawns: u64,
    mut stm_knights: u64,
    mut enemy_knights: u64,
    mut stm_bishops: u64,
    mut enemy_bishops: u64,
    mut stm_rooks: u64,
    mut enemy_rooks: u64,
    mut stm_queens: u64,
    mut enemy_queens: u64,
    stm_king: u64,
    enemy_king: u64,
    stm_color_iswhite: bool,
) -> AdditionalBitBoards {
    let stm_pieces_without_king: u64 =
        stm_pawns | stm_knights | stm_bishops | stm_rooks | stm_queens;
    let stm_pieces: u64 = stm_pieces_without_king | stm_king;
    let enemy_pieces =
        enemy_pawns | enemy_knights | enemy_bishops | enemy_rooks | enemy_queens | enemy_king;
    let all_pieces_without_stmking = enemy_pieces | stm_pieces_without_king;
    let all_pieces = all_pieces_without_stmking | stm_king;

    let (mut stm_unsafe_squares, mut all_checkers) = (0u64, 0u64);

    //Pawns
    let (
        stm_pawns_westattack,
        stm_pawns_eastattack,
        enemy_pawns_westattack,
        enemy_pawns_eastattack,
    ) = if stm_color_iswhite {
        (
            w_pawn_west_targets(stm_pawns),
            w_pawn_east_targets(stm_pawns),
            b_pawn_west_targets(enemy_pawns),
            b_pawn_east_targets(enemy_pawns),
        )
    } else {
        (
            b_pawn_west_targets(stm_pawns),
            b_pawn_east_targets(stm_pawns),
            w_pawn_west_targets(enemy_pawns),
            w_pawn_east_targets(enemy_pawns),
        )
    };

    stm_unsafe_squares |= enemy_pawns_westattack | enemy_pawns_eastattack;
    all_checkers |= if stm_color_iswhite {
        w_pawn_west_targets(stm_king & enemy_pawns_westattack)
    } else {
        b_pawn_west_targets(stm_king & enemy_pawns_westattack)
    } | if stm_color_iswhite {
        w_pawn_east_targets(stm_king & enemy_pawns_eastattack)
    } else {
        b_pawn_east_targets(stm_king & enemy_pawns_eastattack)
    };

    //Knights
    while enemy_knights != 0u64 {
        let enemy_knightindex = enemy_knights.trailing_zeros() as usize;
        let enemy_knight = 1u64 << enemy_knightindex;
        let enemy_knightattacks = knight_attack(enemy_knightindex);
        stm_unsafe_squares |= enemy_knightattacks;
        if stm_king & enemy_knightattacks != 0u64 {
            all_checkers |= enemy_knight;
        }
        enemy_knights ^= enemy_knight;
    }

    //Bishops
    while enemy_bishops != 0u64 {
        let enemy_bishopindex = enemy_bishops.trailing_zeros() as usize;
        let enemy_bishop = 1u64 << enemy_bishopindex;
        let enemy_bishopattacks = bishop_attack(enemy_bishopindex, all_pieces_without_stmking);
        stm_unsafe_squares |= enemy_bishopattacks;
        if stm_king & enemy_bishopattacks != 0u64 {
            all_checkers |= enemy_bishop;
        }
        enemy_bishops ^= enemy_bishop;
    }

    //Rooks
    while enemy_rooks != 0u64 {
        let enemy_rookindex = enemy_rooks.trailing_zeros() as usize;
        let enemy_rook = 1u64 << enemy_rookindex;
        let enemy_rookattacks = rook_attack(enemy_rookindex, all_pieces_without_stmking);
        stm_unsafe_squares |= enemy_rookattacks;
        if stm_king & enemy_rookattacks != 0u64 {
            all_checkers |= enemy_rook;
        }
        enemy_rooks ^= enemy_rook;
    }

    //Queens
    while enemy_queens != 0u64 {
        let enemy_queenindex = enemy_queens.trailing_zeros() as usize;
        let enemy_queen = 1u64 << enemy_queenindex;
        let enemy_queenattacks = bishop_attack(enemy_queenindex, all_pieces_without_stmking)
            | rook_attack(enemy_queenindex, all_pieces_without_stmking);
        stm_unsafe_squares |= enemy_queenattacks;
        if stm_king & enemy_queenattacks != 0u64 {
            all_checkers |= enemy_queen;
        }
        enemy_queens ^= enemy_queen;
    }
    //Kings
    let stm_king_attacks = king_attack(stm_king.trailing_zeros() as usize) & !stm_pieces;
    let enemy_king_attacks = king_attack(enemy_king.trailing_zeros() as usize);
    stm_unsafe_squares |= enemy_king_attacks;
    AdditionalBitBoards {
        stm_pieces_without_king,
        stm_pieces,
        enemy_pieces,
        all_pieces_without_stmking,
        all_pieces,
        stm_pawns_westattack,
        stm_pawns_eastattack,
        enemy_pawns_westattack,
        enemy_pawns_eastattack,
        stm_king_attacks,
        enemy_king_attacks,
        stm_unsafe_squares,
        all_checkers,
    }
}

pub struct AdditionalGameStateInformation {
    stm_incheck: bool,
    stm_haslegalmove: bool,
    additional_bitboards: AdditionalBitBoards,
}
#[inline(always)]
pub fn add_pin_moves_to_movelist(
    legal_moves: &mut Vec<GameMove>,
    only_captures: bool,
    ray_to_king: u64,
    push_mask: u64,
    capture_mask: u64,
    enemy_pinner: u64,
    pinned_piece_position: usize,
    moving_piece_type: PieceType,
    pinner_position: usize,
    enemy_queens: u64,
    other_pinner_piece_type: PieceType,
) -> bool {
    let pin_quiet_targets = ray_to_king & push_mask & !(1u64 << pinned_piece_position);
    let pin_capture_possible = (capture_mask & enemy_pinner) != 0u64;
    let haslegalmove = pin_capture_possible || pin_quiet_targets != 0u64;
    if !only_captures {
        add_moves_to_movelist(
            legal_moves,
            pinned_piece_position,
            pin_quiet_targets,
            moving_piece_type.clone(),
            GameMoveType::Quiet,
        );
    }
    if pin_capture_possible {
        add_move_to_movelist(
            legal_moves,
            pinned_piece_position,
            pinner_position,
            moving_piece_type.clone(),
            GameMoveType::Capture(if enemy_pinner & enemy_queens != 0u64 {
                PieceType::Queen
            } else {
                other_pinner_piece_type
            }),
        );
    }
    haslegalmove
}
#[inline(always)]
pub fn add_king_moves_to_movelist(
    legal_moves: &mut Vec<GameMove>,
    only_captures: bool,
    stm_legal_kingmoves: u64,
    stm_king_index: usize,
    enemy_pawns: u64,
    enemy_knights: u64,
    enemy_bishops: u64,
    enemy_rooks: u64,
    enemy_queens: u64,
    enemy_pieces: u64,
) {
    let mut captures = stm_legal_kingmoves & enemy_pieces;
    let quiets = stm_legal_kingmoves & !captures;
    while captures != 0u64 {
        let capture_index = captures.trailing_zeros() as usize;
        add_move_to_movelist(
            legal_moves,
            stm_king_index,
            capture_index,
            PieceType::King,
            GameMoveType::Capture(find_captured_piece_type(
                capture_index,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            )),
        );
        captures ^= 1u64 << capture_index;
    }
    if !only_captures {
        add_moves_to_movelist(
            legal_moves,
            stm_king_index,
            quiets,
            PieceType::King,
            GameMoveType::Quiet,
        );
    }
}

#[inline(always)]
pub fn add_pawn_moves_to_movelist(
    legal_moves: &mut Vec<GameMove>,
    mut target_board: u64,
    shift: usize,
    stm_color_iswhite: bool,
    enemy_pawns: u64,
    enemy_knights: u64,
    enemy_bishops: u64,
    enemy_rooks: u64,
    enemy_queens: u64,
    is_capture: bool,
    is_promotion: bool,
    pinned_pieces: u64,
) -> bool {
    let mut stm_haslegalmove = false;
    while target_board != 0u64 {
        let pawn_index = target_board.trailing_zeros() as usize;
        let pawn = 1u64 << pawn_index;
        let from_index = if stm_color_iswhite {
            pawn_index - shift
        } else {
            pawn_index + shift
        };
        let from_board = 1u64 << from_index;
        if from_board & pinned_pieces == 0u64 {
            stm_haslegalmove = true;
            let mv_type = if is_capture {
                GameMoveType::Capture(find_captured_piece_type(
                    pawn_index,
                    enemy_pawns,
                    enemy_knights,
                    enemy_bishops,
                    enemy_rooks,
                    enemy_queens,
                ))
            } else {
                GameMoveType::Quiet
            };
            if is_promotion {
                add_promotion_move_to_movelist(legal_moves, from_index, pawn_index, mv_type);
            } else {
                add_move_to_movelist(
                    legal_moves,
                    from_index,
                    pawn_index,
                    PieceType::Pawn,
                    mv_type,
                )
            }
        }
        target_board ^= pawn;
    }
    stm_haslegalmove
}
#[inline(always)]
pub fn add_normal_moves_to_movelist(
    legal_moves: &mut Vec<GameMove>,
    piece_type: PieceType,
    mut piece_board: u64,
    pinned_pieces: u64,
    enemy_pawns: u64,
    enemy_knights: u64,
    enemy_bishops: u64,
    enemy_rooks: u64,
    enemy_queens: u64,
    enemy_pieces: u64,
    empty_squares: u64,
    all_pieces: u64,
    push_mask: u64,
    capture_mask: u64,
    only_captures: bool,
) -> bool {
    let mut stm_haslegalmove = false;
    let mut index = 0;
    while piece_board != 0u64 {
        let piece_index = piece_board.trailing_zeros() as usize;
        let piece = 1u64 << piece_index;
        if piece & pinned_pieces == 0u64 {
            let piece_target = if let PieceType::Knight = piece_type {
                knight_attack(piece_index)
            } else if let PieceType::Bishop = piece_type {
                bishop_attack(piece_index, all_pieces)
            } else if let PieceType::Rook = piece_type {
                rook_attack(piece_index, all_pieces)
            } else if let PieceType::Queen = piece_type {
                bishop_attack(piece_index, all_pieces) | rook_attack(piece_index, all_pieces)
            } else {
                panic!("Shouldn't get here")
            };
            let mut captures = piece_target & capture_mask & enemy_pieces;
            stm_haslegalmove |= captures != 0u64;
            while captures != 0u64 {
                let capture_index = captures.trailing_zeros() as usize;
                add_move_to_movelist(
                    legal_moves,
                    piece_index,
                    capture_index,
                    piece_type.clone(),
                    GameMoveType::Capture(find_captured_piece_type(
                        capture_index,
                        enemy_pawns,
                        enemy_knights,
                        enemy_bishops,
                        enemy_rooks,
                        enemy_queens,
                    )),
                );
                captures ^= 1u64 << capture_index;
            }

            if !only_captures || !stm_haslegalmove {
                let quiets = piece_target & push_mask & empty_squares;
                stm_haslegalmove |= quiets != 0u64;
                if !only_captures {
                    add_moves_to_movelist(
                        legal_moves,
                        piece_index,
                        quiets,
                        piece_type.clone(),
                        GameMoveType::Quiet,
                    );
                }
            }
        }
        index = index + 1;
        piece_board ^= piece;
    }
    stm_haslegalmove
}
#[inline(always)]
pub fn add_promotion_move_to_movelist(
    legal_moves: &mut Vec<GameMove>,
    from_square: usize,
    to_square: usize,
    move_type: GameMoveType,
) {
    let new_types = if let GameMoveType::Capture(x) = move_type {
        (
            GameMoveType::Promotion(PieceType::Queen, Some(x.clone())),
            GameMoveType::Promotion(PieceType::Rook, Some(x.clone())),
            GameMoveType::Promotion(PieceType::Bishop, Some(x.clone())),
            GameMoveType::Promotion(PieceType::Knight, Some(x.clone())),
        )
    } else {
        (
            GameMoveType::Promotion(PieceType::Queen, None),
            GameMoveType::Promotion(PieceType::Rook, None),
            GameMoveType::Promotion(PieceType::Bishop, None),
            GameMoveType::Promotion(PieceType::Knight, None),
        )
    };
    add_move_to_movelist(
        legal_moves,
        from_square,
        to_square,
        PieceType::Pawn,
        new_types.0,
    );
    add_move_to_movelist(
        legal_moves,
        from_square,
        to_square,
        PieceType::Pawn,
        new_types.1,
    );
    add_move_to_movelist(
        legal_moves,
        from_square,
        to_square,
        PieceType::Pawn,
        new_types.2,
    );
    add_move_to_movelist(
        legal_moves,
        from_square,
        to_square,
        PieceType::Pawn,
        new_types.3,
    );
}
#[inline(always)]
pub fn add_moves_to_movelist(
    legal_moves: &mut Vec<GameMove>,
    from_square: usize,
    mut target_board: u64,
    piece_type: PieceType,
    move_type: GameMoveType,
) {
    while target_board != 0u64 {
        let target_square = target_board.trailing_zeros() as usize;
        add_move_to_movelist(
            legal_moves,
            from_square,
            target_square,
            piece_type.clone(),
            move_type.clone(),
        );
        target_board ^= 1u64 << target_square;
    }
}
#[inline(always)]
pub fn add_move_to_movelist(
    legal_moves: &mut Vec<GameMove>,
    from_square: usize,
    to_square: usize,
    piece_type: PieceType,
    move_type: GameMoveType,
) {
    legal_moves.push(GameMove {
        from: from_square,
        to: to_square,
        move_type: move_type,
        piece_type: piece_type,
    })
}

pub fn generate_moves2(
    g: &game_state::GameState,
    only_captures: bool,
) -> (Vec<GameMove>, AdditionalGameStateInformation) {
    let mut legal_moves: Vec<GameMove> = Vec::with_capacity(38);

    //**********************************************************************
    //0.General Bitboards and Variable Initialization
    let stm_color = g.color_to_move;
    let enemy_color = 1 - stm_color;
    let stm_color_iswhite: bool = g.color_to_move == 0;

    let mut stm_pawns: u64 = g.pieces[0][stm_color];
    let stm_knights: u64 = g.pieces[1][stm_color];
    let stm_bishops: u64 = g.pieces[2][stm_color];
    let stm_rooks: u64 = g.pieces[3][stm_color];
    let stm_queens: u64 = g.pieces[4][stm_color];
    let stm_king: u64 = g.pieces[5][stm_color];
    let stm_king_index: usize = stm_king.trailing_zeros() as usize;

    let enemy_pawns: u64 = g.pieces[0][enemy_color];
    let enemy_knights: u64 = g.pieces[1][enemy_color];
    let enemy_bishops: u64 = g.pieces[2][enemy_color];
    let enemy_rooks: u64 = g.pieces[3][enemy_color];
    let enemy_queens: u64 = g.pieces[4][enemy_color];
    let enemy_king: u64 = g.pieces[5][enemy_color];

    let mut stm_haslegalmove = false;
    //----------------------------------------------------------------------
    //**********************************************************************
    //1.Calculate additional needed bitboards
    let abb = calculate_additionalbitboards(
        stm_pawns,
        enemy_pawns,
        stm_knights,
        enemy_knights,
        stm_bishops,
        enemy_bishops,
        stm_rooks,
        enemy_rooks,
        stm_queens,
        enemy_queens,
        stm_king,
        enemy_king,
        stm_color_iswhite,
    );
    //----------------------------------------------------------------------
    //**********************************************************************
    //2. Safe King moves
    let stm_legal_kingmoves = abb.stm_king_attacks & !abb.stm_unsafe_squares;
    stm_haslegalmove |= stm_legal_kingmoves != 0u64;
    add_king_moves_to_movelist(
        &mut legal_moves,
        only_captures,
        stm_legal_kingmoves,
        stm_king_index,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        abb.enemy_pieces,
    );
    //----------------------------------------------------------------------
    //**********************************************************************
    //3. Check & Check Evasions
    let checkers = abb.all_checkers.count_ones() as usize;
    let stm_incheck = checkers > 0;

    let mut capture_mask = 0xFFFFFFFFFFFFFFFFu64;
    let mut push_mask = 0xFFFFFFFFFFFFFFFFu64;
    if checkers > 1 {
        //Double check, only safe king moves are legal
        return (
            legal_moves,
            AdditionalGameStateInformation {
                stm_incheck,
                stm_haslegalmove,
                additional_bitboards: abb,
            },
        );
    } else if checkers == 1 {
        //Only a single checker
        capture_mask = abb.all_checkers;
        //If it's a slider, we can also push in it's way
        if abb.all_checkers & (enemy_bishops | enemy_rooks | enemy_queens) != 0u64 {
            let checker_square = abb.all_checkers.trailing_zeros() as usize;
            if abb.all_checkers & (bitboards::FREEFIELD_ROOK_ATTACKS[stm_king_index]) != 0u64 {
                //Checker is rook-like
                push_mask = get_rook_ray(stm_king_index, checker_square);
            } else {
                //Checker is bishop-like
                push_mask = get_bishop_ray(stm_king_index, checker_square);
            }
        } else {
            //else, we can't do push (quiet) moves
            push_mask = 0u64;
        }
    }

    let empty_squares = !abb.all_pieces;
    //----------------------------------------------------------------------
    //**********************************************************************
    //4. Pins and pinned pieces
    let mut pinned_pieces = 0u64;
    //4.1 Rook-Like pins
    let stm_rook_attacks_from_king = rook_attack(stm_king_index, abb.all_pieces);
    let stm_xray_rook_attacks_from_king = xray_rook_attacks(
        stm_rook_attacks_from_king,
        abb.all_pieces,
        abb.stm_pieces,
        stm_king_index,
    );
    let mut enemy_rooks_on_xray = stm_xray_rook_attacks_from_king & (enemy_rooks | enemy_queens);
    while enemy_rooks_on_xray != 0u64 {
        let enemy_rook_position = enemy_rooks_on_xray.trailing_zeros() as usize;
        let enemy_rook = 1u64 << enemy_rook_position;
        let ray_to_king = get_rook_ray(stm_king_index, enemy_rook_position);
        let pinned_piece = ray_to_king & abb.stm_pieces;
        let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
        pinned_pieces |= pinned_piece;
        if pinned_piece & stm_queens != 0u64 {
            //Add possible queen pushes
            stm_haslegalmove |= add_pin_moves_to_movelist(
                &mut legal_moves,
                only_captures,
                ray_to_king,
                push_mask,
                capture_mask,
                enemy_rook,
                pinned_piece_position,
                PieceType::Queen,
                enemy_rook_position,
                enemy_queens,
                PieceType::Rook,
            );
        } else if pinned_piece & stm_rooks != 0u64 {
            //Add possible rook pushes
            stm_haslegalmove |= add_pin_moves_to_movelist(
                &mut legal_moves,
                only_captures,
                ray_to_king,
                push_mask,
                capture_mask,
                enemy_rook,
                pinned_piece_position,
                PieceType::Rook,
                enemy_rook_position,
                enemy_queens,
                PieceType::Rook,
            );
        } else if pinned_piece & stm_pawns != 0u64 {
            //Add possible pawn pushes
            stm_pawns ^= pinned_piece;
            let stm_pawn_pin_single_push = if stm_color_iswhite {
                w_single_push_pawn_targets(pinned_piece, empty_squares)
            } else {
                b_single_push_pawn_targets(pinned_piece, empty_squares)
            } & ray_to_king
                & push_mask;
            let stm_pawn_pin_double_push = if stm_color_iswhite {
                w_double_push_pawn_targets(pinned_piece, empty_squares)
            } else {
                b_double_push_pawn_targets(pinned_piece, empty_squares)
            } & ray_to_king
                & push_mask;
            stm_haslegalmove |= (stm_pawn_pin_single_push | stm_pawn_pin_double_push) != 0u64;
            if !only_captures {
                add_moves_to_movelist(
                    &mut legal_moves,
                    pinned_piece_position,
                    stm_pawn_pin_single_push | stm_pawn_pin_double_push,
                    PieceType::Pawn,
                    GameMoveType::Quiet,
                )
            }
        }
        enemy_rooks_on_xray ^= enemy_rook;
    }
    //4.2 Bishop-Like pins
    let stm_bishop_attacks_from_king = bishop_attack(stm_king_index, abb.all_pieces);
    let stm_xray_bishop_attacks_from_king = xray_bishop_attacks(
        stm_bishop_attacks_from_king,
        abb.all_pieces,
        abb.stm_pieces,
        stm_king_index,
    );
    let mut enemy_bishop_on_xray =
        stm_xray_bishop_attacks_from_king & (enemy_bishops | enemy_queens);
    while enemy_bishop_on_xray != 0u64 {
        let enemy_bishop_position = enemy_bishop_on_xray.trailing_zeros() as usize;
        let enemy_bishop = 1u64 << enemy_bishop_position;
        let ray_to_king = get_bishop_ray(stm_king_index, enemy_bishop_position);
        let pinned_piece = ray_to_king & abb.stm_pieces;
        let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
        pinned_pieces |= pinned_piece;
        if pinned_piece & stm_queens != 0u64 {
            //Add possible queen pushes
            stm_haslegalmove |= add_pin_moves_to_movelist(
                &mut legal_moves,
                only_captures,
                ray_to_king,
                push_mask,
                capture_mask,
                enemy_bishop,
                pinned_piece_position,
                PieceType::Queen,
                enemy_bishop_position,
                enemy_queens,
                PieceType::Bishop,
            );
        } else if pinned_piece & stm_bishops != 0u64 {
            //Add possible bishop pushes
            stm_haslegalmove |= add_pin_moves_to_movelist(
                &mut legal_moves,
                only_captures,
                ray_to_king,
                push_mask,
                capture_mask,
                enemy_bishop,
                pinned_piece_position,
                PieceType::Bishop,
                enemy_bishop_position,
                enemy_queens,
                PieceType::Bishop,
            );
        } else if pinned_piece & stm_pawns != 0u64 {
            //Add possible pawn captures
            stm_pawns ^= pinned_piece;
            //TODO MAKE SURE THIS REMOVED PINNED PAWN DOESN'T CAPTURE FROM ABB.WEST OR ABB.EAST TARGETS
            let stm_pawn_pin_target = if stm_color_iswhite {
                w_pawn_east_targets(pinned_piece) | w_pawn_west_targets(pinned_piece)
            } else {
                b_pawn_east_targets(pinned_piece) | b_pawn_west_targets(pinned_piece)
            };
            //Normal captures
            let stm_pawn_pin_captures = stm_pawn_pin_target & capture_mask & enemy_bishop;
            let stm_pawn_pin_promotion_capture =
                stm_pawn_pin_captures & bitboards::RANKS[if stm_color_iswhite { 7 } else { 0 }];
            if stm_pawn_pin_promotion_capture != 0u64 {
                stm_haslegalmove = true;
                add_promotion_move_to_movelist(
                    &mut legal_moves,
                    pinned_piece_position,
                    enemy_bishop_position,
                    GameMoveType::Capture(if enemy_bishop & enemy_queens != 0u64 {
                        PieceType::Queen
                    } else {
                        PieceType::Bishop
                    }),
                );
            }
            let stm_pawn_pin_nonpromotion_capture =
                stm_pawn_pin_captures & !stm_pawn_pin_promotion_capture;
            if stm_pawn_pin_nonpromotion_capture != 0u64 {
                stm_haslegalmove = true;
                add_move_to_movelist(
                    &mut legal_moves,
                    pinned_piece_position,
                    enemy_bishop_position,
                    PieceType::Pawn,
                    GameMoveType::Capture(if enemy_bishop & enemy_queens != 0u64 {
                        PieceType::Queen
                    } else {
                        PieceType::Bishop
                    }),
                );
            }
            //En-Passants
            let stm_pawn_pin_enpassant =
                stm_pawn_pin_target & g.en_passant & capture_mask & ray_to_king;
            if stm_pawn_pin_enpassant != 0u64 {
                stm_haslegalmove = true;
                add_move_to_movelist(
                    &mut legal_moves,
                    pinned_piece_position,
                    stm_pawn_pin_target.trailing_zeros() as usize,
                    PieceType::Pawn,
                    GameMoveType::EnPassant,
                );
            }
        }
        enemy_bishop_on_xray ^= enemy_bishop;
    }

    //----------------------------------------------------------------------
    //**********************************************************************
    //5. Pawn pushes, captures, and promotions (captures, capture-enpassant,capture-promotion,normal-promotion)
    //5.1 Single push (Promotions and pushes)
    let stm_pawns_single_push = if stm_color_iswhite {
        w_single_push_pawn_targets(stm_pawns, empty_squares)
    } else {
        b_single_push_pawn_targets(stm_pawns, empty_squares)
    } & push_mask;
    stm_haslegalmove |= stm_pawns_single_push != 0u64;
    let stm_pawn_promotions =
        stm_pawns_single_push & bitboards::RANKS[if stm_color_iswhite { 7 } else { 0 }];
    add_pawn_moves_to_movelist(
        &mut legal_moves,
        stm_pawn_promotions,
        8,
        stm_color_iswhite,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        false,
        true,
        pinned_pieces,
    );
    if !only_captures {
        let stm_pawns_quiet_single_push = stm_pawns_single_push & !stm_pawn_promotions;
        add_pawn_moves_to_movelist(
            &mut legal_moves,
            stm_pawns_quiet_single_push,
            8,
            stm_color_iswhite,
            enemy_pawns,
            enemy_knights,
            enemy_bishops,
            enemy_rooks,
            enemy_queens,
            false,
            false,
            pinned_pieces,
        );
    }
    //5.2 Double push
    if !only_captures || !stm_haslegalmove {
        let stm_pawns_double_push = if stm_color_iswhite {
            w_double_push_pawn_targets(stm_pawns, empty_squares)
        } else {
            b_double_push_pawn_targets(stm_pawns, empty_squares)
        } & push_mask;
        stm_haslegalmove |= stm_pawns_double_push != 0u64;
        if !only_captures {
            add_pawn_moves_to_movelist(
                &mut legal_moves,
                stm_pawns_double_push,
                16,
                stm_color_iswhite,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
                false,
                false,
                pinned_pieces,
            );
        }
    }
    //5.3 West captures (normal capture, promotion capture, en-passant)
    let stm_pawn_west_captures = abb.stm_pawns_westattack & capture_mask & abb.enemy_pieces;
    //Split up in promotion and non-promotion captures
    let stm_pawn_west_promotion_capture =
        stm_pawn_west_captures & bitboards::RANKS[if stm_color_iswhite { 7 } else { 0 }];
    stm_haslegalmove |= add_pawn_moves_to_movelist(
        &mut legal_moves,
        stm_pawn_west_promotion_capture,
        7,
        stm_color_iswhite,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        true,
        true,
        pinned_pieces,
    );
    let stm_pawn_west_nonpromotion_capture =
        stm_pawn_west_captures & !stm_pawn_west_promotion_capture;
    stm_haslegalmove |= add_pawn_moves_to_movelist(
        &mut legal_moves,
        stm_pawn_west_nonpromotion_capture,
        7,
        stm_color_iswhite,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        true,
        false,
        pinned_pieces,
    );
    //En-Passants
    let stm_pawn_west_enpassants = abb.stm_pawns_westattack
        & g.en_passant
        & if stm_color_iswhite {
            capture_mask << 8
        } else {
            capture_mask >> 8
        };
    if stm_pawn_west_enpassants != 0u64
        && if stm_color_iswhite {
            stm_pawn_west_enpassants >> 7
        } else {
            stm_pawn_west_enpassants << 7
        } & pinned_pieces
            == 0u64
    {
        let pawn_index = stm_pawn_west_enpassants.trailing_zeros() as usize;
        let (pawn_from, removed_piece_index) = if stm_color_iswhite {
            (pawn_index - 7, pawn_index - 8)
        } else {
            (pawn_index + 7, pawn_index + 8)
        };
        let all_pieces_without_en_passants =
            abb.all_pieces & !(1u64 << pawn_from) & !(1u64 << removed_piece_index);
        if rook_attack(stm_king_index, all_pieces_without_en_passants)
            & bitboards::RANKS[stm_king_index / 8]
            & enemy_rooks
            == 0u64
        {
            stm_haslegalmove = true;
            add_move_to_movelist(
                &mut legal_moves,
                pawn_from,
                pawn_index,
                PieceType::Pawn,
                GameMoveType::EnPassant,
            );
        }
    }
    //5.4 East captures (normal capture, promotion capture, en-passant)
    let stm_pawn_east_captures = abb.stm_pawns_eastattack & capture_mask & abb.enemy_pieces;
    //Split up in promotion and non-promotion captures
    let stm_pawn_east_promotion_capture =
        stm_pawn_east_captures & bitboards::RANKS[if stm_color_iswhite { 7 } else { 0 }];
    stm_haslegalmove |= add_pawn_moves_to_movelist(
        &mut legal_moves,
        stm_pawn_east_promotion_capture,
        9,
        stm_color_iswhite,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        true,
        true,
        pinned_pieces,
    );
    let stm_pawn_east_nonpromotion_capture =
        stm_pawn_east_captures & !stm_pawn_east_promotion_capture;
    stm_haslegalmove |= add_pawn_moves_to_movelist(
        &mut legal_moves,
        stm_pawn_east_nonpromotion_capture,
        9,
        stm_color_iswhite,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        true,
        false,
        pinned_pieces,
    );
    //En-Passants
    let stm_pawn_east_enpassants = abb.stm_pawns_eastattack
        & g.en_passant
        & if stm_color_iswhite {
            capture_mask << 8
        } else {
            capture_mask >> 8
        };
    if stm_pawn_east_enpassants != 0u64
        && if stm_color_iswhite {
            stm_pawn_east_enpassants >> 9
        } else {
            stm_pawn_east_enpassants << 9
        } & pinned_pieces
            == 0u64
    {
        let pawn_index = stm_pawn_east_enpassants.trailing_zeros() as usize;
        let (pawn_from, removed_piece_index) = if stm_color_iswhite {
            (pawn_index - 9, pawn_index - 8)
        } else {
            (pawn_index + 9, pawn_index + 8)
        };
        let all_pieces_without_en_passants =
            abb.all_pieces & !(1u64 << pawn_from) & !(1u64 << removed_piece_index);
        if rook_attack(stm_king_index, all_pieces_without_en_passants)
            & bitboards::RANKS[stm_king_index / 8]
            & enemy_rooks
            == 0u64
        {
            stm_haslegalmove = true;
            add_move_to_movelist(
                &mut legal_moves,
                pawn_from,
                pawn_index,
                PieceType::Pawn,
                GameMoveType::EnPassant,
            );
        }
    }

    //----------------------------------------------------------------------
    //**********************************************************************
    //6. All other legal moves (knights,bishops,rooks,queens)
    //6.1 Knights
    stm_haslegalmove |= add_normal_moves_to_movelist(
        &mut legal_moves,
        PieceType::Knight,
        stm_knights,
        pinned_pieces,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        abb.enemy_pieces,
        empty_squares,
        abb.all_pieces,
        push_mask,
        capture_mask,
        only_captures,
    );
    //6.2 Bishops
    stm_haslegalmove |= add_normal_moves_to_movelist(
        &mut legal_moves,
        PieceType::Bishop,
        stm_bishops,
        pinned_pieces,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        abb.enemy_pieces,
        empty_squares,
        abb.all_pieces,
        push_mask,
        capture_mask,
        only_captures,
    );
    //6.3 Rooks
    stm_haslegalmove |= add_normal_moves_to_movelist(
        &mut legal_moves,
        PieceType::Rook,
        stm_rooks,
        pinned_pieces,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        abb.enemy_pieces,
        empty_squares,
        abb.all_pieces,
        push_mask,
        capture_mask,
        only_captures,
    );
    //6.4 Queens
    stm_haslegalmove |= add_normal_moves_to_movelist(
        &mut legal_moves,
        PieceType::Queen,
        stm_queens,
        pinned_pieces,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        abb.enemy_pieces,
        empty_squares,
        abb.all_pieces,
        push_mask,
        capture_mask,
        only_captures,
    );
    //----------------------------------------------------------------------
    //**********************************************************************
    //7. Castling
    if (!only_captures || !stm_haslegalmove) && checkers == 0 {
        if stm_color_iswhite {
            if g.castle_white_kingside {
                if (abb.all_pieces | abb.stm_unsafe_squares)
                    & (bitboards::SQUARES[5] | bitboards::SQUARES[6])
                    == 0u64
                {
                    stm_haslegalmove = true;
                    if !only_captures {
                        legal_moves.push(GameMove {
                            from: stm_king_index,
                            to: 6usize,
                            move_type: GameMoveType::Castle,
                            piece_type: PieceType::King,
                        });
                    }
                }
            }
            if g.castle_white_queenside {
                if ((abb.all_pieces | abb.stm_unsafe_squares)
                    & (bitboards::SQUARES[2] | bitboards::SQUARES[3])
                    | abb.all_pieces & bitboards::SQUARES[1])
                    == 0u64
                {
                    stm_haslegalmove = true;
                    if !only_captures {
                        legal_moves.push(GameMove {
                            from: stm_king_index,
                            to: 2usize,
                            move_type: GameMoveType::Castle,
                            piece_type: PieceType::King,
                        });
                    }
                }
            }
        } else {
            if g.castle_black_kingside {
                if (abb.all_pieces | abb.stm_unsafe_squares)
                    & (bitboards::SQUARES[61] | bitboards::SQUARES[62])
                    == 0u64
                {
                    stm_haslegalmove = true;
                    if !only_captures {
                        legal_moves.push(GameMove {
                            from: stm_king_index,
                            to: 62usize,
                            move_type: GameMoveType::Castle,
                            piece_type: PieceType::King,
                        });
                    }
                }
            }
            if g.castle_black_queenside {
                if ((abb.all_pieces | abb.stm_unsafe_squares)
                    & (bitboards::SQUARES[58] | bitboards::SQUARES[59])
                    | abb.all_pieces & bitboards::SQUARES[57])
                    == 0u64
                {
                    stm_haslegalmove = true;
                    if !only_captures {
                        legal_moves.push(GameMove {
                            from: stm_king_index,
                            to: 58usize,
                            move_type: GameMoveType::Castle,
                            piece_type: PieceType::King,
                        });
                    }
                }
            }
        }
    }
    //----------------------------------------------------------------------
    let agi = AdditionalGameStateInformation {
        stm_incheck,
        stm_haslegalmove,
        additional_bitboards: abb,
    };
    (legal_moves, agi)
}
