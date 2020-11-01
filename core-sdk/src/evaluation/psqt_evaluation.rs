use super::EvaluationScore;
use crate::bitboards::bitboards::constants::square;
use crate::board_representation::game_state::{GameState, PieceType, PIECE_TYPES, WHITE};
use crate::evaluation::params::{KING_PIECE_TABLE, PSQT};

#[cfg(feature = "tuning")]
use crate::board_representation::game_state::white_pov;
#[cfg(feature = "tuning")]
use crate::evaluation::parameters::normal_parameters::IDX_KING_PIECE_TABLE;
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
                trace.add(IDX_PSQT + 64 * pt as usize + white_pov(idx, side), if side == WHITE { 1 } else { -1 });
            }
        }
        res += piece_sum;

        #[cfg(feature = "display-eval")]
        {
            println!("\t{:?}  : {}", pt, piece_sum);
        }
    }

    //KP table
    for piece_side in 0..2 {
        for &piece_type in [PieceType::Pawn, PieceType::Rook].iter() {
            if piece_type == PieceType::Rook && piece_side != 1 {
                continue;
            }
            let mut king_piece_sum = EvaluationScore::default();
            let mut piece = game_state.get_piece(PieceType::Pawn, side ^ piece_side);
            while piece > 0 {
                let idx = piece.trailing_zeros() as usize;
                piece ^= square(idx);
                king_piece_sum += KING_PIECE_TABLE[side][game_state.get_king_square(side)][piece_side][piece_type as usize][idx] * if side == WHITE { 1 } else { -1 };
                #[cfg(feature = "tuning")]
                {
                    trace.add(
                        IDX_KING_PIECE_TABLE + 64 * 64 * (5 * piece_side + piece_type as usize) + 64 * white_pov(game_state.get_king_square(side), side) + white_pov(idx, side),
                        if side == WHITE { 1 } else { -1 },
                    );
                }
            }
            res += king_piece_sum;
            #[cfg(feature = "display-eval")]
            {
                println!("\t King-Piece{:?}-PSQT: {}", piece_type, king_enemy_pawn_sum);
                println!("Sum: {}", res);
            }
        }
    }

    res
}

#[inline(always)]
pub fn kp_remove_piece(king_side: usize, king_square: usize, friendly_piece: bool, piece_type: PieceType, square: usize, score: &mut EvaluationScore) {
    *score -= KING_PIECE_TABLE[king_side][king_square][!friendly_piece as usize][piece_type as usize][square];
}
#[inline(always)]
pub fn kp_add_piece(king_side: usize, king_square: usize, friendly_piece: bool, piece_type: PieceType, square: usize, score: &mut EvaluationScore) {
    *score += KING_PIECE_TABLE[king_side][king_square][!friendly_piece as usize][piece_type as usize][square];
}
#[inline(always)]
pub fn kp_move_king(king_start: usize, king_dest: usize, mut friendly_pawns: u64, mut enemy_pawns: u64, mut enemy_rooks: u64, side: usize, score: &mut EvaluationScore) {
    while enemy_pawns > 0 {
        let idx = enemy_pawns.trailing_zeros() as usize;
        enemy_pawns ^= square(idx);
        kp_remove_piece(side, king_start, false, PieceType::Pawn, idx, score);
        kp_add_piece(side, king_dest, false, PieceType::Pawn, idx, score);
    }
    while friendly_pawns > 0 {
        let idx = friendly_pawns.trailing_zeros() as usize;
        friendly_pawns ^= square(idx);
        kp_remove_piece(side, king_start, true, PieceType::Pawn, idx, score);
        kp_add_piece(side, king_dest, true, PieceType::Pawn, idx, score);
    }
    while enemy_rooks > 0 {
        let idx = enemy_rooks.trailing_zeros() as usize;
        enemy_rooks ^= square(idx);
        kp_remove_piece(side, king_start, false, PieceType::Rook, idx, score);
        kp_add_piece(side, king_dest, false, PieceType::Rook, idx, score);
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
