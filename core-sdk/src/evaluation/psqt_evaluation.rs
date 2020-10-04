use super::EvaluationScore;
use crate::bitboards::bitboards::constants::square;
use crate::board_representation::game_state::{GameState, PieceType, PIECE_TYPES, WHITE};
use crate::evaluation::params::{KING_ENEMY_PAWN, PSQT};

#[cfg(feature = "tuning")]
use crate::board_representation::game_state::white_pov;
#[cfg(feature = "tuning")]
use crate::evaluation::parameters::normal_parameters::IDX_KING_ENEMY_PAWN;
#[cfg(feature = "tuning")]
use crate::evaluation::parameters::normal_parameters::IDX_PSQT;
#[cfg(feature = "tuning")]
use crate::evaluation::trace::LargeTrace;

pub fn psqt(game_state: &GameState, side: usize, #[cfg(feature = "tuning")] trace: &mut LargeTrace) -> EvaluationScore {
    #[cfg(feature = "display-eval")]
    {
        println!("\nPSQT for {}:", if side == WHITE { "White" } else { "Black" });
    }

    let mut res = EvaluationScore::default();

    for &pt in PIECE_TYPES.iter() {
        let mut piece_sum = EvaluationScore::default();
        let mut piece = game_state.get_piece(pt, side);

        while piece > 0 {
            let idx = piece.trailing_zeros() as usize;
            piece ^= square(idx);
            piece_sum += PSQT[pt as usize][side][idx] * if side == WHITE { 1 } else { -1 };

            #[cfg(feature = "tuning")]
            {
                trace.normal_coeffs[IDX_PSQT + 64 * pt as usize + white_pov(idx, side)] += if side == WHITE { 1 } else { -1 };
            }
        }
        res += piece_sum;

        #[cfg(feature = "display-eval")]
        {
            println!("\t{:?}  : {}", pt, piece_sum);
        }
    }

    //KP table
    let mut king_enemy_pawn_sum = EvaluationScore::default();
    let mut enemy_pawns = game_state.get_piece(PieceType::Pawn, 1 - side);
    while enemy_pawns > 0 {
        let idx = enemy_pawns.trailing_zeros() as usize;
        enemy_pawns ^= square(idx);
        king_enemy_pawn_sum += KING_ENEMY_PAWN[side][game_state.get_king_square(side)][idx] * if side == WHITE { 1 } else { -1 };
        #[cfg(feature = "tuning")]
        {
            trace.normal_coeffs[IDX_KING_ENEMY_PAWN + 64 * white_pov(game_state.get_king_square(side), side) + white_pov(idx, side)] += if side == WHITE { 1 } else { -1 };
        }
    }
    res += king_enemy_pawn_sum;
    #[cfg(feature = "display-eval")]
    {
        println!("\t King-Enemy-Pawn-PSQT: {}", king_enemy_pawn_sum);
        println!("Sum: {}", res);
    }
    res
}

#[inline(always)]
pub fn kp_remove_pawn(king_square: usize, square: usize, side: usize, score: &mut EvaluationScore) {
    *score -= KING_ENEMY_PAWN[side][king_square][square];
}
#[inline(always)]
pub fn kp_add_pawn(king_square: usize, square: usize, side: usize, score: &mut EvaluationScore) {
    *score += KING_ENEMY_PAWN[side][king_square][square];
}
#[inline(always)]
pub fn kp_move_king(king_start: usize, king_dest: usize, mut enemy_pawns: u64, side: usize, score: &mut EvaluationScore) {
    while enemy_pawns > 0 {
        let idx = enemy_pawns.trailing_zeros() as usize;
        enemy_pawns ^= square(idx);
        kp_remove_pawn(king_start, idx, side, score);
        kp_add_pawn(king_dest, idx, side, score);
    }
}
#[inline(always)]
pub fn psqt_remove_piece(piece: PieceType, square: usize, side: usize, score: &mut EvaluationScore) {
    *score -= piece.to_psqt(side, square);
}

#[inline(always)]
pub fn psqt_add_piece(piece: PieceType, square: usize, side: usize, score: &mut EvaluationScore) {
    *score += piece.to_psqt(side, square);
}
