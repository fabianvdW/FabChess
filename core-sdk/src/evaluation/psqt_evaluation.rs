use super::EvaluationResult;
use super::EvaluationScore;
use crate::bitboards::bitboards::constants::square;
use crate::board_representation::game_state::{GameState, PieceType, PIECE_TYPES, WHITE};
use crate::evaluation::params::PSQT;

pub const BLACK_INDEX: [usize; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63, 48, 49, 50, 51, 52, 53, 54, 55, 40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39, 24, 25, 26, 27, 28, 29, 30, 31, 16, 17, 18, 19, 20, 21, 22, 23,
    8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7,
];

pub fn psqt(game_state: &GameState, side: usize, _eval: &mut EvaluationResult) -> EvaluationScore {
    let mut res = EvaluationScore::default();
    #[cfg(feature = "display-eval")]
    {
        println!(
            "\nPSQT for {}:",
            if side == WHITE { "White" } else { "Black" }
        );
    }
    for pt in PIECE_TYPES.iter() {
        let mut piece_sum = EvaluationScore::default();
        let mut piece = game_state.get_piece(*pt, side);
        while piece > 0 {
            #[allow(unused_mut)]
            let mut idx = piece.trailing_zeros() as usize;
            piece ^= square(idx);
            piece_sum +=
                PSQT[*pt as usize][side][idx / 8][idx % 8] * if side == WHITE { 1 } else { -1 };
            #[cfg(feature = "texel-tuning")]
            {
                if side != WHITE {
                    idx = BLACK_INDEX[idx];
                }
                _eval.trace.psqt[*pt as usize][idx / 8][idx % 8] +=
                    if side == WHITE { 1 } else { -1 };
            }
        }
        res += piece_sum;
        #[cfg(feature = "display-eval")]
        {
            println!("\t{:?}  : {}", *pt, piece_sum);
        }
    }
    #[cfg(feature = "display-eval")]
    {
        println!("Sum: {}", res);
    }
    res
}
#[inline(always)]
pub fn psqt_remove_piece(
    piece: PieceType,
    square: usize,
    side: usize,
    score: &mut EvaluationScore,
) {
    *score -= piece.to_psqt(side, square);
}
#[inline(always)]
pub fn psqt_add_piece(piece: PieceType, square: usize, side: usize, score: &mut EvaluationScore) {
    *score += piece.to_psqt(side, square);
}
