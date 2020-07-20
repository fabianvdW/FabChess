use crate::bitboards::bitboards::constants::square;
use crate::board_representation::game_state::{GameState, PIECE_TYPES, WHITE};
use crate::evaluation::nn::NN;
use crate::evaluation::nn_trace::trace_pos;

pub const BLACK_INDEX: [usize; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63, 48, 49, 50, 51, 52, 53, 54, 55, 40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39, 24, 25, 26, 27, 28, 29, 30, 31, 16, 17, 18, 19, 20, 21, 22, 23,
    8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7,
];

pub fn psqt(
    game_state: &GameState,
    side: usize,
    nn: &mut NN,
    #[cfg(feature = "texel-tuning")] nn_trace: &mut NNTrace,
) {
    let side_mult = if side == WHITE { 1f32 } else { -1f32 };
    for pt in PIECE_TYPES.iter() {
        let mut piece = game_state.get_piece(*pt, side);
        while piece > 0 {
            #[allow(unused_mut)]
            let mut idx = piece.trailing_zeros() as usize;
            piece ^= square(idx);
            if side != WHITE {
                idx = BLACK_INDEX[idx];
            }
            let position_in_arr = trace_pos::PSQT * 64 * (*pt as usize) + 8 * (idx / 8) + idx % 8;
            #[cfg(feature = "texel-tuning")]
            {
                nn_trace.trace[position_in_arr] += side_mult;
            }
            nn.internal_state
                .evaluate_feature_1d(position_in_arr, side_mult);
        }
    }
}
