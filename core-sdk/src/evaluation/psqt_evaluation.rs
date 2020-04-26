use super::params::*;
use super::EvaluationResult;
use super::EvaluationScore;
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

pub fn psqt(white: bool, pieces: &[[u64; 2]; 6], _eval: &mut EvaluationResult) -> EvaluationScore {
    let mut pawn = EvaluationScore::default();
    let mut knight = EvaluationScore::default();
    let mut bishop = EvaluationScore::default();
    let mut rook = EvaluationScore::default();
    let mut queen = EvaluationScore::default();
    let king;

    let side = if white { WHITE } else { BLACK };

    let mut pawns = pieces[PAWN][side];
    while pawns != 0u64 {
        let mut idx = pawns.trailing_zeros() as usize;
        pawns ^= 1u64 << idx;
        if !white {
            idx = BLACK_INDEX[idx];
        }
        pawn += PSQT_PAWN[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_pawn[idx / 8][idx % 8] += if side == WHITE { 1 } else { -1 };
        }
    }

    let mut knights = pieces[KNIGHT][side];
    while knights != 0u64 {
        let mut idx = knights.trailing_zeros() as usize;
        knights ^= 1u64 << idx;
        if !white {
            idx = BLACK_INDEX[idx]
        }
        knight += PSQT_KNIGHT[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_knight[idx / 8][idx % 8] += if side == WHITE { 1 } else { -1 };
        }
    }

    let mut bishops = pieces[BISHOP][side];
    while bishops != 0u64 {
        let mut idx = bishops.trailing_zeros() as usize;
        bishops ^= 1u64 << idx;
        if !white {
            idx = BLACK_INDEX[idx];
        }
        bishop += PSQT_BISHOP[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_bishop[idx / 8][idx % 8] += if side == WHITE { 1 } else { -1 };
        }
    }

    let mut rooks = pieces[ROOK][side];
    while rooks != 0u64 {
        let mut idx = rooks.trailing_zeros() as usize;
        rooks ^= 1u64 << idx;
        if !white {
            idx = BLACK_INDEX[idx];
        }
        rook += PSQT_ROOK[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_rook[idx / 8][idx % 8] += if side == WHITE { 1 } else { -1 };
        }
    }

    let mut queens = pieces[QUEEN][side];
    while queens != 0u64 {
        let mut idx = queens.trailing_zeros() as usize;
        queens ^= 1u64 << idx;
        if !white {
            idx = BLACK_INDEX[idx];
        }
        queen += PSQT_QUEEN[idx / 8][idx % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.psqt_queen[idx / 8][idx % 8] += if side == WHITE { 1 } else { -1 };
        }
    }
    let mut king_idx = pieces[KING][side].trailing_zeros() as usize;
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
        log(&format!(
            "\nPSQT for {}:\n",
            if white { "White" } else { "Black" }
        ));
        log(&format!("\tPawns  : {}\n", pawn));
        log(&format!("\tKnights: {}\n", knight));
        log(&format!("\tBishops: {}\n", bishop));
        log(&format!("\tRooks: {}\n", rook));
        log(&format!("\tQueens: {}\n", queen));
        log(&format!("\tKing   : {}\n", king));
        log(&format!("Sum: {}\n", sum));
    }
    sum
}

#[inline(always)]
pub fn psqt_toggle_piece(
    pieces: &mut [[u64; 2]; 6],
    piece: PieceType,
    square: usize,
    side: usize,
    score: &mut EvaluationScore,
) {
    let temp = pieces[piece.to_index()][side];
    let (rank, file) = if side == WHITE {
        (square / 8, square % 8)
    } else {
        (BLACK_INDEX[square] / 8, BLACK_INDEX[square] % 8)
    };
    let mut new_score = piece.to_psqt()[rank][file];
    if (temp & 1u64 << square) == 0u64 {
        new_score *= -1;
    }
    new_score *= if side == WHITE { 1 } else { -1 };
    *score += new_score;
}
