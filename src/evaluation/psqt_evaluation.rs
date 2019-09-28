use super::params::*;
use super::EvaluationResult;
use crate::board_representation::game_state::{
    PieceType, BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE,
};
#[cfg(feature = "display-eval")]
use crate::logging::log;

pub const BLACK_INDEX: [usize; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63, 48, 49, 50, 51, 52, 53, 54, 55, 40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39, 24, 25, 26, 27, 28, 29, 30, 31, 16, 17, 18, 19, 20, 21, 22, 23,
    8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7,
];

pub fn psqt(white: bool, pieces: &[[u64; 2]; 6], _eval: &mut EvaluationResult) -> (i16, i16) {
    let (mut pawn_mg, mut pawn_eg) = (0i16, 0i16);
    let (mut knight_mg, mut knight_eg) = (0i16, 0i16);
    let (mut bishop_mg, mut bishop_eg) = (0i16, 0i16);
    let (mut rook_mg, mut rook_eg) = (0i16, 0i16);
    let (mut queen_mg, mut queen_eg) = (0i16, 0i16);
    let (king_mg, king_eg);
    let side = if white { WHITE } else { BLACK };

    let mut pawns = pieces[PAWN][side];
    while pawns != 0u64 {
        let mut idx = pawns.trailing_zeros() as usize;
        pawns ^= 1u64 << idx;
        if !white {
            idx = BLACK_INDEX[idx];
        }
        pawn_mg += PSQT_PAWN_MG[idx / 8][idx % 8];
        pawn_eg += PSQT_PAWN_EG[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_pawn[side][idx / 8][idx % 8] += 1;
        }
    }

    let mut knights = pieces[KNIGHT][side];
    while knights != 0u64 {
        let mut idx = knights.trailing_zeros() as usize;
        knights ^= 1u64 << idx;
        if !white {
            idx = BLACK_INDEX[idx]
        }
        knight_mg += PSQT_KNIGHT_MG[idx / 8][idx % 8];
        knight_eg += PSQT_KNIGHT_EG[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_knight[side][idx / 8][idx % 8] += 1;
        }
    }

    let mut bishops = pieces[BISHOP][side];
    while bishops != 0u64 {
        let mut idx = bishops.trailing_zeros() as usize;
        bishops ^= 1u64 << idx;
        if !white {
            idx = BLACK_INDEX[idx];
        }
        bishop_mg += PSQT_BISHOP_MG[idx / 8][idx % 8];
        bishop_eg += PSQT_BISHOP_EG[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_bishop[side][idx / 8][idx % 8] += 1;
        }
    }

    let mut rooks = pieces[ROOK][side];
    while rooks != 0u64 {
        let mut idx = rooks.trailing_zeros() as usize;
        rooks ^= 1u64 << idx;
        if !white {
            idx = BLACK_INDEX[idx];
        }
        rook_mg += PSQT_ROOK_MG[idx / 8][idx % 8];
        rook_eg += PSQT_ROOK_EG[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_rook[side][idx / 8][idx % 8] += 1;
        }
    }

    let mut queens = pieces[QUEEN][side];
    while queens != 0u64 {
        let mut idx = queens.trailing_zeros() as usize;
        queens ^= 1u64 << idx;
        if !white {
            idx = BLACK_INDEX[idx];
        }
        queen_mg += PSQT_QUEEN_MG[idx / 8][idx % 8];
        queen_eg += PSQT_QUEEN_EG[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_queen[side][idx / 8][idx % 8] += 1;
        }
    }
    let mut king_idx = pieces[KING][side].trailing_zeros() as usize;
    if !white {
        king_idx = BLACK_INDEX[king_idx];
    }
    king_mg = PSQT_KING_MG[king_idx / 8][king_idx % 8];
    king_eg = PSQT_KING_EG[king_idx / 8][king_idx % 8];
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.psqt_king[side][king_idx / 8][king_idx % 8] += 1;
    }
    let mg_sum = pawn_mg + knight_mg + bishop_mg + rook_mg + queen_mg + king_mg;
    let eg_sum = pawn_eg + knight_eg + bishop_eg + rook_eg + queen_eg + king_eg;
    #[cfg(feature = "display-eval")]
    {
        log(&format!(
            "\nPSQT for {}:\n",
            if white { "White" } else { "Black" }
        ));
        log(&format!("\tPawns  : ({} , {})\n", pawn_mg, pawn_eg));
        log(&format!("\tKnights: ({} , {})\n", knight_mg, knight_eg));
        log(&format!("\tBishops: ({} , {})\n", bishop_mg, bishop_eg));
        log(&format!("\tRooks: ({} , {})\n", rook_mg, rook_eg));
        log(&format!("\tQueens: ({} , {})\n", queen_mg, queen_eg));
        log(&format!("\tKing   : ({} , {})\n", king_mg, king_eg));
        log(&format!("Sum: ({} , {})\n", mg_sum, eg_sum));
    }
    (mg_sum, eg_sum)
}

#[inline(always)]
pub fn psqt_incremental_move_piece(
    piece: PieceType,
    mut from_square: usize,
    mut to_square: usize,
    is_black: bool,
    psqt_mg: i16,
    psqt_eg: i16,
) -> (i16, i16) {
    if is_black {
        from_square = BLACK_INDEX[from_square];
        to_square = BLACK_INDEX[to_square];
    }
    let mut psqt_mg_plus: i16 = 0;
    let mut psqt_eg_plus: i16 = 0;
    if let PieceType::Pawn = piece {
        psqt_mg_plus += PSQT_PAWN_MG[to_square / 8][to_square % 8]
            - PSQT_PAWN_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_PAWN_EG[to_square / 8][to_square % 8]
            - PSQT_PAWN_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Knight = piece {
        psqt_mg_plus += PSQT_KNIGHT_MG[to_square / 8][to_square % 8]
            - PSQT_KNIGHT_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_KNIGHT_EG[to_square / 8][to_square % 8]
            - PSQT_KNIGHT_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Bishop = piece {
        psqt_mg_plus += PSQT_BISHOP_MG[to_square / 8][to_square % 8]
            - PSQT_BISHOP_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_BISHOP_EG[to_square / 8][to_square % 8]
            - PSQT_BISHOP_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Rook = piece {
        psqt_mg_plus += PSQT_ROOK_MG[to_square / 8][to_square % 8]
            - PSQT_ROOK_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_ROOK_EG[to_square / 8][to_square % 8]
            - PSQT_ROOK_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Queen = piece {
        psqt_mg_plus += PSQT_QUEEN_MG[to_square / 8][to_square % 8]
            - PSQT_QUEEN_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_QUEEN_EG[to_square / 8][to_square % 8]
            - PSQT_QUEEN_EG[from_square / 8][from_square % 8];
    } else if let PieceType::King = piece {
        psqt_mg_plus += PSQT_KING_MG[to_square / 8][to_square % 8]
            - PSQT_KING_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_KING_EG[to_square / 8][to_square % 8]
            - PSQT_KING_EG[from_square / 8][from_square % 8];
    }
    if is_black {
        psqt_mg_plus *= -1;
        psqt_eg_plus *= -1;
    }
    (psqt_mg + psqt_mg_plus, psqt_eg + psqt_eg_plus)
}

#[inline(always)]
pub fn psqt_incremental_delete_piece(
    piece: PieceType,
    mut from_square: usize,
    is_black: bool,
    psqt_mg: i16,
    psqt_eg: i16,
) -> (i16, i16) {
    if is_black {
        from_square = BLACK_INDEX[from_square];
    }
    let mut psqt_mg_plus = 0;
    let mut psqt_eg_plus = 0;
    if let PieceType::Pawn = piece {
        psqt_mg_plus += -PSQT_PAWN_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += -PSQT_PAWN_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Knight = piece {
        psqt_mg_plus += -PSQT_KNIGHT_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += -PSQT_KNIGHT_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Bishop = piece {
        psqt_mg_plus += -PSQT_BISHOP_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += -PSQT_BISHOP_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Rook = piece {
        psqt_mg_plus += -PSQT_ROOK_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += -PSQT_ROOK_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Queen = piece {
        psqt_mg_plus += -PSQT_QUEEN_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += -PSQT_QUEEN_EG[from_square / 8][from_square % 8];
    } else if let PieceType::King = piece {
        psqt_mg_plus += -PSQT_KING_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += -PSQT_KING_EG[from_square / 8][from_square % 8];
    }
    if is_black {
        psqt_mg_plus *= -1;
        psqt_eg_plus *= -1;
    }
    (psqt_mg + psqt_mg_plus, psqt_eg + psqt_eg_plus)
}

#[inline(always)]
pub fn psqt_incremental_add_piece(
    piece: PieceType,
    mut from_square: usize,
    is_black: bool,
    psqt_mg: i16,
    psqt_eg: i16,
) -> (i16, i16) {
    if is_black {
        from_square = BLACK_INDEX[from_square];
    }
    let mut psqt_mg_plus = 0;
    let mut psqt_eg_plus = 0;
    if let PieceType::Pawn = piece {
        psqt_mg_plus += PSQT_PAWN_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_PAWN_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Knight = piece {
        psqt_mg_plus += PSQT_KNIGHT_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_KNIGHT_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Bishop = piece {
        psqt_mg_plus += PSQT_BISHOP_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_BISHOP_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Rook = piece {
        psqt_mg_plus += PSQT_ROOK_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_ROOK_EG[from_square / 8][from_square % 8];
    } else if let PieceType::Queen = piece {
        psqt_mg_plus += PSQT_QUEEN_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_QUEEN_EG[from_square / 8][from_square % 8];
    } else if let PieceType::King = piece {
        psqt_mg_plus += PSQT_KING_MG[from_square / 8][from_square % 8];
        psqt_eg_plus += PSQT_KING_EG[from_square / 8][from_square % 8];
    }
    if is_black {
        psqt_mg_plus *= -1;
        psqt_eg_plus *= -1;
    }
    (psqt_mg + psqt_mg_plus, psqt_eg + psqt_eg_plus)
}
