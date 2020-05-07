use super::params::*;
use super::EvaluationResult;
use super::EvaluationScore;
use crate::bitboards::bitboards::constants::square;
use crate::board_representation::game_state::{GameState, PieceType, BLACK, WHITE};

pub const BLACK_INDEX: [usize; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63, 48, 49, 50, 51, 52, 53, 54, 55, 40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39, 24, 25, 26, 27, 28, 29, 30, 31, 16, 17, 18, 19, 20, 21, 22, 23,
    8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7,
];

#[inline(always)]
pub fn psqt(white: bool, state: &GameState, _eval: &mut EvaluationResult) -> EvaluationScore {
    let mut pawn = EvaluationScore::default();
    let mut knight = EvaluationScore::default();
    let mut bishop = EvaluationScore::default();
    let mut rook = EvaluationScore::default();
    let mut queen = EvaluationScore::default();
    let king;

    let side = if white { WHITE } else { BLACK };

    //TODO: Vectorize
    let mut pawns = state.get_piece(PieceType::Pawn, side);
    while pawns != 0u64 {
        let mut idx = pawns.trailing_zeros() as usize;
        pawns ^= square(idx);
        if !white {
            idx = BLACK_INDEX[idx];
        }
        pawn += PSQT_PAWN[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_pawn[idx / 8][idx % 8] += if side == WHITE { 1 } else { -1 };
        }
    }

    let mut knights = state.get_piece(PieceType::Knight, side);
    while knights != 0u64 {
        let mut idx = knights.trailing_zeros() as usize;
        knights ^= square(idx);
        if !white {
            idx = BLACK_INDEX[idx]
        }
        knight += PSQT_KNIGHT[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_knight[idx / 8][idx % 8] += if side == WHITE { 1 } else { -1 };
        }
    }

    let mut bishops = state.get_piece(PieceType::Bishop, side);
    while bishops != 0u64 {
        let mut idx = bishops.trailing_zeros() as usize;
        bishops ^= square(idx);
        if !white {
            idx = BLACK_INDEX[idx];
        }
        bishop += PSQT_BISHOP[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_bishop[idx / 8][idx % 8] += if side == WHITE { 1 } else { -1 };
        }
    }

    let mut rooks = state.get_piece(PieceType::Rook, side);
    while rooks != 0u64 {
        let mut idx = rooks.trailing_zeros() as usize;
        rooks ^= square(idx);
        if !white {
            idx = BLACK_INDEX[idx];
        }
        rook += PSQT_ROOK[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_rook[idx / 8][idx % 8] += if side == WHITE { 1 } else { -1 };
        }
    }

    let mut queens = state.get_piece(PieceType::Queen, side);
    while queens != 0u64 {
        let mut idx = queens.trailing_zeros() as usize;
        queens ^= square(idx);
        if !white {
            idx = BLACK_INDEX[idx];
        }
        queen += PSQT_QUEEN[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_queen[idx / 8][idx % 8] += if side == WHITE { 1 } else { -1 };
        }
    }
    let mut king_idx = state.king_square(side);
    if !white {
        king_idx = BLACK_INDEX[king_idx];
    }
    king = PSQT_KING[king_idx / 8][king_idx % 8];
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.psqt_king[king_idx / 8][king_idx % 8] += if side == WHITE { 1 } else { -1 };
    }
    #[allow(clippy::let_and_return)]
    let sum = pawn + knight + bishop + rook + queen + king;
    #[cfg(feature = "display-eval")]
    {
        println!("\nPSQT for {}:", if white { "White" } else { "Black" });
        println!("\tPawns  : {}", pawn);
        println!("\tKnights: {}", knight);
        println!("\tBishops: {}", bishop);
        println!("\tRooks: {}", rook);
        println!("\tQueens: {}", queen);
        println!("\tKing   : {}", king);
        println!("Sum: {}", sum);
    }
    sum
}
#[inline(always)]
pub fn psqt_set_piece(state: &mut GameState, piece: PieceType, sq: usize, side: usize) {
    let (rank, file) = if side == WHITE {
        (sq / 8, sq % 8)
    } else {
        (BLACK_INDEX[sq] / 8, BLACK_INDEX[sq] % 8)
    };
    state.psqt += piece.to_psqt()[rank][file] * if side == WHITE { 1 } else { -1 };
}
pub fn psqt_unset_piece(state: &mut GameState, piece: PieceType, sq: usize, side: usize) {
    let (rank, file) = if side == WHITE {
        (sq / 8, sq % 8)
    } else {
        (BLACK_INDEX[sq] / 8, BLACK_INDEX[sq] % 8)
    };
    state.psqt -= piece.to_psqt()[rank][file] * if side == WHITE { 1 } else { -1 };
}
