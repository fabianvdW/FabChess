use super::EvaluationScore;
use crate::bitboards::bitboards::constants::square;
use crate::board_representation::game_state::{GameState, PieceType, PIECE_TYPES, WHITE};
use crate::evaluation::params::PSQT;

#[cfg(feature = "texel-tuning")]
use crate::bitboards::bitboards::mirror_square;
#[cfg(feature = "texel-tuning")]
use crate::evaluation::parameters::normal_parameters::IDX_PSQT;
#[cfg(feature = "texel-tuning")]
use crate::evaluation::trace::LargeTrace;

pub fn psqt(game_state: &GameState, side: usize, #[cfg(feature = "texel-tuning")] trace: &mut LargeTrace) -> EvaluationScore {
    let mut res = EvaluationScore::default();
    #[cfg(feature = "display-eval")]
    {
        println!("\nPSQT for {}:", if side == WHITE { "White" } else { "Black" });
    }
    for &pt in PIECE_TYPES.iter() {
        let mut piece_sum = EvaluationScore::default();
        let mut piece = game_state.get_piece(pt, side);
        while piece > 0 {
            #[allow(unused_mut)]
            let mut idx = piece.trailing_zeros() as usize;
            piece ^= square(idx);
            piece_sum += PSQT[pt as usize][side][idx / 8][idx % 8] * if side == WHITE { 1 } else { -1 };
            #[cfg(feature = "texel-tuning")]
            {
                if side != WHITE {
                    idx = mirror_square(idx);
                }
                trace.normal_coeffs[IDX_PSQT + 64 * pt as usize + idx] += if side == WHITE { 1 } else { -1 };
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
pub fn psqt_remove_piece(piece: PieceType, square: usize, side: usize, score: &mut EvaluationScore) {
    *score -= piece.to_psqt(side, square);
}
#[inline(always)]
pub fn psqt_add_piece(piece: PieceType, square: usize, side: usize, score: &mut EvaluationScore) {
    *score += piece.to_psqt(side, square);
}
