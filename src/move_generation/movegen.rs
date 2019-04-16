use super::super::bitboards;
use super::super::board_representation::game_state::{self, GameMove, GameMoveType, PieceType};
use super::magic::{self, Magic};
use super::super::board_representation::zobrist_hashing::ZOBRIST_KEYS;

//Move GEn
//King- Piece-Wise by lookup
//Knight-Piece-Wise by lookup
//Bishop/Queen/Rook - Piece-Wise by lookup in Magic
//Pawn-SetWise by shift
pub fn king_attack(square: usize) -> u64 {
    bitboards::KING_ATTACKS[square]
}

pub fn bishop_attack(square: usize, all_pieces: u64) -> u64 {
    Magic::get_attacks(&magic::MAGICS_BISHOPS[square], all_pieces)
}

pub fn rook_attack(square: usize, all_pieces: u64) -> u64 {
    Magic::get_attacks(&magic::MAGICS_ROOKS[square], all_pieces)
}

pub fn knight_attack(square: usize) -> u64 {
    bitboards::KNIGHT_ATTACKS[square]
}

pub fn w_single_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::north_one(pawns) & empty
}

pub fn w_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::north_one(bitboards::north_one(pawns & bitboards::RANKS[1]) & empty) & empty
}

pub fn b_single_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(pawns) & empty
}

pub fn b_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(bitboards::south_one(pawns & bitboards::RANKS[6]) & empty) & empty
}

//NorthEast = +9
pub fn w_pawn_east_targets(pawns: u64) -> u64 {
    bitboards::north_east_one(pawns)
}

//NorthWest = +7
pub fn w_pawn_west_targets(pawns: u64) -> u64 {
    bitboards::north_west_one(pawns)
}

//SouthEast = -7
pub fn b_pawn_east_targets(pawns: u64) -> u64 {
    bitboards::south_west_one(pawns)
}

//NorthWest = -9
pub fn b_pawn_west_targets(pawns: u64) -> u64 {
    bitboards::south_east_one(pawns)
}

#[inline(always)]
pub fn add_moves(move_list: &mut Vec<GameMove>, from: usize, mut to_board: u64, piece_type: &PieceType, move_type: GameMoveType) {
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
pub fn add_capture_moves(move_list: &mut Vec<GameMove>, from: usize, mut to_board: u64, piece_type: &PieceType, enemy_pawns: u64, enemy_knights: u64, enemy_bishops: u64, enemy_rooks: u64, enemy_queens: u64) {
    while to_board != 0u64 {
        let idx = to_board.trailing_zeros() as usize;
        let pt_cl = piece_type.clone();
        move_list.push(GameMove {
            from,
            to: idx,
            move_type: GameMoveType::Capture(find_captured_piece_type(idx, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens)),
            piece_type: pt_cl,
        });
        to_board ^= 1u64 << idx;
    }
}

//Make moves
pub fn make_move(g: &game_state::GameState, mv: &game_state::GameMove) -> game_state::GameState {
    match &mv.move_type {
        GameMoveType::Quiet => make_quiet_move(&g, &mv),
        GameMoveType::Capture(piece) => make_capture_move(&g, &mv, &piece),
        GameMoveType::EnPassant => make_enpassant_move(&g, &mv),
        GameMoveType::Castle => make_castle_move(&g, &mv),
        GameMoveType::Promotion(_promoting_to, capturing) => make_promotion_move(&g, &mv, &capturing),
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
pub fn castle_hash(ocwk: bool, ocwq: bool, ocbk: bool, ocbq: bool, ncwk: bool, ncwq: bool, ncbk: bool, ncbq: bool, mut hash: u64) -> u64 {
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
pub fn delete_piece_hash(delete_square: usize, delete_color: usize, captured_piece: &PieceType, mut hash: u64) -> u64 {
    if delete_color == 0 {
        hash ^= match captured_piece {
            PieceType::Pawn => ZOBRIST_KEYS.w_pawns,
            PieceType::Knight => ZOBRIST_KEYS.w_knights,
            PieceType::Bishop => ZOBRIST_KEYS.w_bishops,
            PieceType::Rook => ZOBRIST_KEYS.w_rooks,
            PieceType::Queen => ZOBRIST_KEYS.w_queens,
            PieceType::King => panic!("Can't capture king!")
        }[delete_square];
    } else {
        hash ^= match captured_piece {
            PieceType::Pawn => ZOBRIST_KEYS.b_pawns,
            PieceType::Knight => ZOBRIST_KEYS.b_knights,
            PieceType::Bishop => ZOBRIST_KEYS.b_bishops,
            PieceType::Rook => ZOBRIST_KEYS.b_rooks,
            PieceType::Queen => ZOBRIST_KEYS.b_queens,
            PieceType::King => panic!("Can't capture king!")
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
pub fn delete_piece(pieces: &mut [[u64; 2]; 6], captured_piece: &PieceType, delete_square: usize, delete_color: usize) {
    pieces[match captured_piece {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => panic!("Can't capture king!")
    }][delete_color] ^= 1u64 << delete_square;
}

pub fn check_castle_flags(ck: bool, cq: bool, mv: &game_state::GameMove, color_to_move: usize, pieces: [[u64; 2]; 6]) -> (bool, bool) {
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
        _ => (ck, cq)
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
    }
}

pub fn make_quiet_move(g: &game_state::GameState, mv: &game_state::GameMove) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Make the move
    move_piece(&mut pieces, &mv, g.color_to_move);
    //Check new castle rights
    //The enemies castle right's can't change on a quiet move
    let (castle_white_kingside, castle_white_queenside) = if g.color_to_move == 0 {
        check_castle_flags(g.castle_white_kingside, g.castle_white_queenside, &mv, g.color_to_move, pieces)
    } else { (g.castle_white_kingside, g.castle_white_queenside) };
    let (castle_black_kingside, castle_black_queenside) = if g.color_to_move == 0 {
        (g.castle_black_kingside, g.castle_black_queenside)
    } else { check_castle_flags(g.castle_black_kingside, g.castle_black_queenside, &mv, g.color_to_move, pieces) };


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
    hash = castle_hash(g.castle_white_kingside, g.castle_white_queenside, g.castle_black_kingside, g.castle_black_queenside,
                       castle_white_kingside, castle_white_queenside, castle_black_kingside, castle_black_queenside, hash);
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
    }
}

pub fn make_capture_move(g: &game_state::GameState, mv: &game_state::GameMove, captured_piece: &PieceType) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Make the move
    move_piece(&mut pieces, &mv, g.color_to_move);
    //Delete to from enemy pieces
    delete_piece(&mut pieces, &captured_piece, mv.to, color_to_move);

    let (mut castle_white_kingside, mut castle_white_queenside) = if g.color_to_move == 0 {
        check_castle_flags(g.castle_white_kingside, g.castle_white_queenside, &mv, g.color_to_move, pieces)
    } else { (g.castle_white_kingside, g.castle_white_queenside) };
    let (mut castle_black_kingside, mut castle_black_queenside) = if g.color_to_move == 0 {
        (g.castle_black_kingside, g.castle_black_queenside)
    } else { check_castle_flags(g.castle_black_kingside, g.castle_black_queenside, &mv, g.color_to_move, pieces) };

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
    hash = castle_hash(g.castle_white_kingside, g.castle_white_queenside, g.castle_black_kingside, g.castle_black_queenside,
                       castle_white_kingside, castle_white_queenside, castle_black_kingside, castle_black_queenside, hash);
    hash = delete_piece_hash(mv.to, color_to_move, &captured_piece, hash);
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
    }
}

pub fn make_enpassant_move(g: &game_state::GameState, mv: &game_state::GameMove) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Make the move
    move_piece(&mut pieces, &mv, g.color_to_move);
    //Delete enemy pawn
    let delete_square = if g.color_to_move == 0 { mv.to - 8 } else { mv.to + 8 };
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
    hash = castle_hash(g.castle_white_kingside, g.castle_white_queenside, g.castle_black_kingside, g.castle_black_queenside,
                       castle_white_kingside, castle_white_queenside, castle_black_kingside, castle_black_queenside, hash);
    hash = delete_piece_hash(delete_square, color_to_move, &PieceType::Pawn, hash);
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
    }
}

pub fn make_castle_move(g: &game_state::GameState, mv: &game_state::GameMove) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Move the king
    move_piece(&mut pieces, &mv, g.color_to_move);

    //Move the rook
    let mut hash = g.hash ^ ZOBRIST_KEYS.side_to_move;
    let rook_zobrist = if g.color_to_move == 0 { ZOBRIST_KEYS.w_rooks } else { ZOBRIST_KEYS.b_rooks };
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

    let (castle_white_kingside, castle_white_queenside) = if g.color_to_move == 0 { (false, false) } else {
        (g.castle_white_kingside, g.castle_white_queenside)
    };
    let (castle_black_kingside, castle_black_queenside) = if g.color_to_move == 1 { (false, false) } else {
        (g.castle_black_kingside, g.castle_black_queenside)
    };

    let en_passant = 0u64;

    let half_moves = g.half_moves + 1;
    let full_moves = g.full_moves + g.color_to_move;
    hash = move_piece_hash(g.color_to_move, &mv, hash);
    hash = enpassant_hash(g.en_passant, en_passant, hash);
    hash = castle_hash(g.castle_white_kingside, g.castle_white_queenside, g.castle_black_kingside, g.castle_black_queenside,
                       castle_white_kingside, castle_white_queenside, castle_black_kingside, castle_black_queenside, hash);
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
    }
}

pub fn make_promotion_move(g: &game_state::GameState, mv: &game_state::GameMove, captured_piece: &Option<PieceType>) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    let mut hash = g.hash ^ ZOBRIST_KEYS.side_to_move;
    hash ^= if g.color_to_move == 0 { ZOBRIST_KEYS.w_pawns } else { ZOBRIST_KEYS.b_pawns }[mv.from];
    hash ^= match mv.move_type {
        GameMoveType::Promotion(PieceType::Queen, _) => {
            if g.color_to_move == 0 { ZOBRIST_KEYS.w_queens } else { ZOBRIST_KEYS.b_queens }
        }
        GameMoveType::Promotion(PieceType::Rook, _) => {
            if g.color_to_move == 0 { ZOBRIST_KEYS.w_rooks } else { ZOBRIST_KEYS.b_rooks }
        }
        GameMoveType::Promotion(PieceType::Knight, _) => {
            if g.color_to_move == 0 { ZOBRIST_KEYS.w_knights } else { ZOBRIST_KEYS.b_knights }
        }
        GameMoveType::Promotion(PieceType::Bishop, _) => {
            if g.color_to_move == 0 { ZOBRIST_KEYS.w_bishops } else { ZOBRIST_KEYS.b_bishops }
        }
        _ => panic!("Invalid Type")
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
        GameMoveType::Promotion(PieceType::Queen, _) => { 4 }
        GameMoveType::Promotion(PieceType::Knight, _) => { 1 }
        GameMoveType::Promotion(PieceType::Bishop, _) => { 2 }
        GameMoveType::Promotion(PieceType::Rook, _) => { 3 }
        _ => panic!("Invalid Type")
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
    hash = castle_hash(g.castle_white_kingside, g.castle_white_queenside, g.castle_black_kingside, g.castle_black_queenside,
                       castle_white_kingside, castle_white_queenside, castle_black_kingside, castle_black_queenside, hash);
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
        get_b_attacked_squares(enemy_king_idx, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks
                               , all_pieces_without_my_king)
    } else {
        get_w_attacked_squares(enemy_king_idx, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks
                               , all_pieces_without_my_king)
    };
    let possible_king_moves = king_attack(my_king_idx) & !unsafe_white_squares & not_my_pieces;
    add_moves(&mut move_list, my_king_idx, possible_king_moves & not_enemy_pieces, &PieceType::King, GameMoveType::Quiet);
    add_capture_moves(&mut move_list, my_king_idx, possible_king_moves & enemy_pieces, &PieceType::King, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens);
    let (king_attackers_board, checking_piece_is_slider, checking_piece_slider_is_bishop) = if unsafe_white_squares & my_king == 0 {
        (0u64, false, false)
    } else {
        if color_to_move == 0 {
            attackers_from_black(my_king, my_king_idx, enemy_pawns, enemy_knights
                                 , enemy_bishops, enemy_rooks, all_pieces)
        } else {
            attackers_from_white(my_king, my_king_idx, enemy_pawns, enemy_knights
                                 , enemy_bishops, enemy_rooks, all_pieces)
        }
    };

    let num_checkers = king_attackers_board.count_ones();
    //Double check
    if num_checkers > 1 { //Then only king moves are possible anyway
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
                    push_mask = get_bishop_ray(bishop_attack(checking_piece_square, 0u64), my_king_idx, checking_piece_square);
                } else {
                    push_mask = get_rook_ray(rook_attack(checking_piece_square, 0u64), my_king_idx, checking_piece_square);
                }
            } else {
                push_mask = 0u64;
            }
        }
    }

    //Pinned pieces
    {//Pinned by rook
        let rook_attacks_from_my_king_postion = rook_attack(my_king_idx, all_pieces);
        //See one layer through my pieces
        //If a rook is found seeing through one piece, the piece is pinned
        let xray_rooks = xray_rook_attacks(rook_attacks_from_my_king_postion, all_pieces, my_pieces, my_king_idx);
        //Go through all directions
        //Find the rooks with xray
        let mut rooks_on_xray = xray_rooks & enemy_rooks;
        while rooks_on_xray != 0 {
            let rook_position = rooks_on_xray.trailing_zeros() as usize;
            rooks_on_xray &= rooks_on_xray - 1;
            let ray = get_rook_ray(rook_attacks_from_my_king_postion | xray_rooks, rook_position, my_king_idx);
            let pinned_piece = ray & my_pieces;
            let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
            if pinned_piece & my_rooks != 0 {
                let mut piece_type = PieceType::Rook;
                if pinned_piece & my_queens != 0 {
                    my_bishops ^= pinned_piece;
                    piece_type = PieceType::Queen;
                }
                my_rooks ^= pinned_piece;
                add_moves(&mut move_list, pinned_piece_position, ray & !pinned_piece & push_mask, &piece_type, GameMoveType::Quiet);
                //Don't forget that we can capture the rook aswell
                add_capture_moves(&mut move_list, pinned_piece_position, bitboards::SQUARES[rook_position] & capture_mask, &piece_type, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens);
                continue;
            }
            if pinned_piece & my_pawns != 0 {
                my_pawns ^= pinned_piece;
                let pawn_push_once = if color_to_move == 0 { w_single_push_pawn_targets(pinned_piece, empty) } else { b_single_push_pawn_targets(pinned_piece, empty) } & ray;
                let pawn_push_twice = if color_to_move == 0 { w_double_push_pawn_targets(pinned_piece, empty) } else { b_double_push_pawn_targets(pinned_piece, empty) } & ray;
                add_moves(&mut move_list, pinned_piece_position, (pawn_push_once | pawn_push_twice) & push_mask, &PieceType::Pawn, GameMoveType::Quiet);
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
        let xray_bishops = xray_bishop_attacks(bishop_attacks_from_my_king_position, all_pieces, my_pieces, my_king_idx);
        let mut bishops_on_xray = xray_bishops & enemy_bishops;
        while bishops_on_xray != 0 {
            let bishop_position = bishops_on_xray.trailing_zeros() as usize;
            bishops_on_xray &= bishops_on_xray - 1;
            let ray = get_bishop_ray(bishop_attacks_from_my_king_position | xray_bishops, bishop_position, my_king_idx);
            let pinned_piece = ray & my_pieces;
            let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
            if pinned_piece & my_bishops != 0 {
                let mut piece_type = PieceType::Bishop;
                if pinned_piece & my_queens != 0 {
                    my_rooks ^= pinned_piece;
                    piece_type = PieceType::Queen;
                }
                my_bishops ^= pinned_piece;

                add_moves(&mut move_list, pinned_piece_position, ray & !pinned_piece & push_mask, &piece_type, GameMoveType::Quiet);
                add_capture_moves(&mut move_list, pinned_piece_position, bitboards::SQUARES[bishop_position] & capture_mask, &piece_type, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens);
                continue;
            }
            if pinned_piece & my_pawns != 0 {
                my_pawns ^= pinned_piece;
                let pawn_targets = if color_to_move == 0 { w_pawn_east_targets(pinned_piece) } else { b_pawn_east_targets(pinned_piece) } | if color_to_move == 0 { w_pawn_west_targets(pinned_piece) } else { b_pawn_west_targets(pinned_piece) };
                let pawn_captures = pawn_targets & bitboards::SQUARES[bishop_position] & capture_mask;
                let pawn_normal_captures = pawn_captures & !bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
                let pawn_promotion_captures = pawn_captures & bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
                let source_shift = pawn_promotion_captures.trailing_zeros() as isize - pinned_piece_position as isize;
                //println!("{}",misc::to_string_board(pawn_targets));
                let pawn_enpassants = pawn_targets & g.en_passant & capture_mask & (bitboards::SQUARES[bishop_position] | ray);
                add_capture_moves(&mut move_list, pinned_piece_position, pawn_normal_captures, &PieceType::Pawn, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens);
                add_promotion_capture(pawn_promotion_captures, &color_to_move, &mut move_list, (source_shift * if color_to_move == 0 { 1 } else { -1 }) as usize, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens);
                add_moves(&mut move_list, pinned_piece_position, pawn_enpassants, &PieceType::Pawn, GameMoveType::EnPassant);
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
            let my_pawns_single_push = if color_to_move == 0 { w_single_push_pawn_targets(my_pawns, empty) } else { b_single_push_pawn_targets(my_pawns, empty) } & push_mask;
            let my_pawns_no_promotion = my_pawns_single_push & !bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            let my_pawns_promotion = my_pawns_single_push & bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            add_quiet_pawn_single_pushes(my_pawns_no_promotion, &color_to_move, &mut move_list);
            add_promotion_push(my_pawns_promotion, &color_to_move, &mut move_list, 8usize);
        }
        //Double push
        {
            let my_pawns_double_push = if color_to_move == 0 { w_double_push_pawn_targets(my_pawns, empty) } else { b_double_push_pawn_targets(my_pawns, empty) } & push_mask;
            add_quiet_pawn_double_pushes(my_pawns_double_push, &color_to_move, &mut move_list);
        }
        //Capture west
        {
            let my_pawns_west_targets = if color_to_move == 0 { w_pawn_west_targets(my_pawns) } else { b_pawn_west_targets(my_pawns) };
            let my_pawns_west_normal_captures = my_pawns_west_targets & capture_mask & enemy_pieces;
            //Checking for promotion on capture
            let my_pawns_no_promotion = my_pawns_west_normal_captures & !bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            let my_pawns_promotion = my_pawns_west_normal_captures & bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            //Capture
            add_pawn_capture(my_pawns_no_promotion, &color_to_move, &mut move_list, 7usize, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens);
            //Promotion capture
            add_promotion_capture(my_pawns_promotion, &color_to_move, &mut move_list, 7usize, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens);


            //En passant
            //We can capture en passant, if its in capture mask aswell
            let my_pawns_west_enpassants = my_pawns_west_targets & g.en_passant & if color_to_move == 0 { capture_mask << 8 } else { capture_mask >> 8 };
            add_en_passants(my_pawns_west_enpassants, &color_to_move, &mut move_list, 7usize, all_pieces_without_my_king, enemy_rooks, my_king_idx);
        }
        //Capture east
        {
            let my_pawns_east_targets = if color_to_move == 0 { w_pawn_east_targets(my_pawns) } else { b_pawn_east_targets(my_pawns) };
            let my_pawns_east_normal_captures = my_pawns_east_targets & capture_mask & enemy_pieces;
            //Checking for promotion on capture
            let my_pawns_no_promotion = my_pawns_east_normal_captures & !bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            let my_pawns_promotion = my_pawns_east_normal_captures & bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            add_pawn_capture(my_pawns_no_promotion, &color_to_move, &mut move_list, 9usize, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens);
            add_promotion_capture(my_pawns_promotion, &color_to_move, &mut move_list, 9usize, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens);
            //En passants
            let my_pawns_east_enpassants = my_pawns_east_targets & g.en_passant & if color_to_move == 0 { capture_mask << 8 } else { capture_mask >> 8 };
            add_en_passants(my_pawns_east_enpassants, &color_to_move, &mut move_list, 9usize, all_pieces_without_my_king, enemy_rooks, my_king_idx);
        }
    }
    //Knights
    {
        while my_knights != 0u64 {
            let index = if color_to_move == 0 { 63usize - my_knights.leading_zeros() as usize } else { my_knights.trailing_zeros() as usize };
            let my_knight_attacks = knight_attack(index) & not_my_pieces;
            let my_knight_captures = my_knight_attacks & enemy_pieces & capture_mask;
            add_capture_moves(&mut move_list, index, my_knight_captures, &PieceType::Knight, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens);
            let my_knight_quiets = my_knight_attacks & !enemy_pieces & push_mask;
            add_moves(&mut move_list, index, my_knight_quiets, &PieceType::Knight, GameMoveType::Quiet);
            my_knights ^= 1u64 << index;
        }
    }
    //Bishops
    {
        while my_bishops != 0u64 {
            let index = if color_to_move == 0 { 63usize - my_bishops.leading_zeros() as usize } else { my_bishops.trailing_zeros() as usize };
            let piece = 1u64 << index;
            let my_bishop_attack = bishop_attack(index, all_pieces) & not_my_pieces;
            let my_bishop_capture = my_bishop_attack & enemy_pieces & capture_mask;
            let piece_type = if piece & my_queens != 0 { PieceType::Queen } else { PieceType::Bishop };
            add_capture_moves(&mut move_list, index, my_bishop_capture, &piece_type, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens);
            let my_bishop_quiet = my_bishop_attack & !enemy_pieces & push_mask;
            add_moves(&mut move_list, index, my_bishop_quiet, &piece_type, GameMoveType::Quiet);
            my_bishops ^= piece;
        }
    }
    //Rooks
    {
        while my_rooks != 0u64 {
            let index = if color_to_move == 0 { 63usize - my_rooks.leading_zeros() as usize } else { my_rooks.trailing_zeros() as usize };
            let piece = 1u64 << index;
            let my_rook_attack = rook_attack(index, all_pieces) & not_my_pieces;
            let my_rook_capture = my_rook_attack & enemy_pieces & capture_mask;
            let piece_type = if piece & my_queens != 0 { PieceType::Queen } else { PieceType::Rook };
            add_capture_moves(&mut move_list, index, my_rook_capture, &piece_type, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens);
            let my_rook_quiets = my_rook_attack & !enemy_pieces & push_mask;
            add_moves(&mut move_list, index, my_rook_quiets, &piece_type, GameMoveType::Quiet);
            my_rooks ^= piece;
        }
    }
    //Castles
    if num_checkers == 0 {
        if g.color_to_move == 0 {
            //Make sure there is no piece in between and safe squares
            if g.castle_white_kingside {
                if (all_pieces | unsafe_white_squares) & (bitboards::SQUARES[5] | bitboards::SQUARES[6]) == 0 {
                    move_list.push(GameMove {
                        from: my_king_idx,
                        to: 6usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
            if g.castle_white_queenside {
                if ((all_pieces | unsafe_white_squares) & (bitboards::SQUARES[2] | bitboards::SQUARES[3]) | all_pieces & bitboards::SQUARES[1]) == 0 {
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
                if (all_pieces | unsafe_white_squares) & (bitboards::SQUARES[61] | bitboards::SQUARES[62]) == 0 {
                    move_list.push(GameMove {
                        from: my_king_idx,
                        to: 62usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
            if g.castle_black_queenside {
                if ((all_pieces | unsafe_white_squares) & (bitboards::SQUARES[58] | bitboards::SQUARES[59]) | all_pieces & bitboards::SQUARES[57]) == 0 {
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
pub fn find_captured_piece_type(to: usize, e_pawns: u64, e_knights: u64, e_bishops: u64, e_rooks: u64, e_queens: u64) -> PieceType {
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
pub fn add_quiet_pawn_single_pushes(mut single_push_board: u64, color_to_move: &usize, move_list: &mut Vec<GameMove>) {
    while single_push_board != 0u64 {
        let idx = single_push_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - 8 } else { idx + 8 },
            to: idx,
            move_type: GameMoveType::Quiet,
            piece_type: PieceType::Pawn,
        });
        single_push_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_quiet_pawn_double_pushes(mut double_push_board: u64, color_to_move: &usize, move_list: &mut Vec<GameMove>) {
    while double_push_board != 0u64 {
        let idx = double_push_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - 16 } else { idx + 16 },
            to: idx,
            move_type: GameMoveType::Quiet,
            piece_type: PieceType::Pawn,
        });
        double_push_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_promotion_push(mut promotion_board: u64, color_to_move: &usize, move_list: &mut Vec<GameMove>, source_shift: usize) {
    while promotion_board != 0u64 {
        let idx = promotion_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Queen, None),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Rook, None),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Bishop, None),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Knight, None),
            piece_type: PieceType::Pawn,
        });
        promotion_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_promotion_capture(mut promotion_board: u64, color_to_move: &usize, move_list: &mut Vec<GameMove>, source_shift: usize, enemy_pawns: u64, enemy_knights: u64, enemy_bishops: u64, enemy_rooks: u64, enemy_queens: u64) {
    while promotion_board != 0u64 {
        let idx = promotion_board.trailing_zeros() as usize;
        let x: Option<PieceType> = Some(find_captured_piece_type(idx, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens));
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Queen, x.clone()),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Rook, x.clone()),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Bishop, x.clone()),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Knight, x),
            piece_type: PieceType::Pawn,
        });
        promotion_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_pawn_capture(mut capture_board: u64, color_to_move: &usize, move_list: &mut Vec<GameMove>, source_shift: usize, enemy_pawns: u64, enemy_knights: u64, enemy_bishops: u64, enemy_rooks: u64, enemy_queens: u64) {
    while capture_board != 0u64 {
        let idx = capture_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Capture(find_captured_piece_type(idx, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens)),
            piece_type: PieceType::Pawn,
        });
        capture_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_en_passants(mut enpassant_board: u64, color_to_move: &usize, move_list: &mut Vec<GameMove>, source_shift: usize, all_pieces_without_my_king: u64, enemy_rooks: u64, my_king_idx: usize) {
    while enpassant_board != 0u64 {
        let index = enpassant_board.trailing_zeros() as usize;
        enpassant_board ^= 1u64 << index;
        //Check if rare case didn't happen
        //Remove t-7,t-8 or t+7,t+8
        let all_pieces_without_en_passants = all_pieces_without_my_king & !bitboards::SQUARES[if *color_to_move == 0 { index - source_shift } else { index + source_shift }]
            & !bitboards::SQUARES[if *color_to_move == 0 { index - 8 } else { index + 8 }];
        if rook_attack(my_king_idx, all_pieces_without_en_passants) & (!bitboards::FILES[my_king_idx % 8]) & enemy_rooks != 0 {
            continue;
        }
        move_list.push(GameMove {
            from: if *color_to_move == 0 { index - source_shift } else { index + source_shift },
            to: index,
            move_type: GameMoveType::EnPassant,
            piece_type: PieceType::Pawn,
        });
    }
}

pub fn xray_rook_attacks(attacks: u64, occupied_square: u64, one_time_blockers: u64, rook_square: usize) -> u64 {
    return attacks ^ rook_attack(rook_square, occupied_square ^ (one_time_blockers & attacks));
}

pub fn xray_bishop_attacks(attacks: u64, occupied_square: u64, one_time_blockers: u64, bishop_square: usize) -> u64 {
    return attacks ^ bishop_attack(bishop_square, occupied_square ^ (one_time_blockers & attacks));
}

//Gets the ray of one rook into a specific direction
pub fn get_rook_ray(rook_attacks_in_all_directions: u64, target_square: usize, rook_square: usize) -> u64 {
    let diff = target_square as isize - rook_square as isize;
    let target_rank = target_square / 8;
    let target_file = target_square % 8;
    let rook_rank = rook_square / 8;
    let rook_file = rook_square % 8;
    if diff > 0 {
        //Same vertical
        if target_rank == rook_rank {
            return bitboards::FILES_LESS_THAN[target_file] & bitboards::FILES_GREATER_THAN[rook_file] & rook_attacks_in_all_directions;
        } else {
            return bitboards::RANKS_LESS_THAN[target_rank] & bitboards::RANKS_GREATER_THAN[rook_rank] & rook_attacks_in_all_directions;
        }
    } else {
        if target_rank == rook_rank {
            return bitboards::FILES_GREATER_THAN[target_file] & bitboards::FILES_LESS_THAN[rook_file] & rook_attacks_in_all_directions;
        } else {
            return bitboards::RANKS_GREATER_THAN[target_rank] & bitboards::RANKS_LESS_THAN[rook_rank] & rook_attacks_in_all_directions;
        }
    }
}

//Gets the rof of one bishop into a specific direction
pub fn get_bishop_ray(bishop_attack_in_all_directions: u64, target_square: usize, bishop_square: usize) -> u64 {
    let diff = target_square as isize - bishop_square as isize;
    let target_rank = target_square / 8;
    let target_file = target_square % 8;
    let bishop_rank = bishop_square / 8;
    let bishop_file = bishop_square % 8;
    if diff > 0 {
        if diff % 9 == 0 {
            return bitboards::FILES_LESS_THAN[target_file] & bitboards::FILES_GREATER_THAN[bishop_file]
                & bitboards::RANKS_LESS_THAN[target_rank] & bitboards::RANKS_GREATER_THAN[bishop_rank]
                & bishop_attack_in_all_directions;
        } else {
            return bitboards::FILES_GREATER_THAN[target_file] & bitboards::FILES_LESS_THAN[bishop_file]
                & bitboards::RANKS_LESS_THAN[target_rank] & bitboards::RANKS_GREATER_THAN[bishop_rank]
                & bishop_attack_in_all_directions;
        }
    } else {
        if diff % -9 == 0 {
            return bitboards::FILES_GREATER_THAN[target_file] & bitboards::FILES_LESS_THAN[bishop_file]
                & bitboards::RANKS_GREATER_THAN[target_rank] & bitboards::RANKS_LESS_THAN[bishop_rank]
                & bishop_attack_in_all_directions;
        } else {
            return bitboards::FILES_LESS_THAN[target_file] & bitboards::FILES_GREATER_THAN[bishop_file]
                & bitboards::RANKS_GREATER_THAN[target_rank] & bitboards::RANKS_LESS_THAN[bishop_rank]
                & bishop_attack_in_all_directions;
        }
    }
}

pub fn attackers_from_white(square_board: u64, square: usize, white_pawns: u64, white_knights: u64, white_bishops: u64, white_rooks: u64, blockers: u64) -> (u64, bool, bool) {
    let mut attackers = 0u64;
    let mut slider_flag = false;
    let mut bishop_slider = false;
    attackers |= knight_attack(square) & white_knights;
    attackers |= (b_pawn_west_targets(square_board) | b_pawn_east_targets(square_board)) & white_pawns;
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

pub fn attackers_from_black(square_board: u64, square: usize, black_pawns: u64, black_knights: u64, black_bishops: u64, black_rooks: u64, blockers: u64) -> (u64, bool, bool) {
    let mut attackers = 0u64;
    let mut slider_flag = false;
    let mut bishop_slider = false;
    attackers |= knight_attack(square) & black_knights;
    attackers |= (w_pawn_west_targets(square_board) | w_pawn_east_targets(square_board)) & black_pawns;
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

pub fn get_w_attacked_squares(white_king_idx: usize, white_pawns: u64, mut white_knights: u64, mut white_bishops: u64, mut white_rooks: u64, blocked_squares: u64) -> u64 {
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

pub fn get_b_attacked_squares(black_king_idx: usize, black_pawns: u64, mut black_knights: u64, mut black_bishops: u64, mut black_rooks: u64, blocked_squares: u64) -> u64 {
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
