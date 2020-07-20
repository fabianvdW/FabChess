pub mod nn;
pub mod nn_trace;
pub mod parameters;
pub mod phase;
pub mod psqt_evaluation;
pub mod trace;

use crate::bitboards::bitboards;
use crate::bitboards::bitboards::constants::*;
use crate::board_representation::game_state::{GameState, PieceType, BLACK, WHITE};
use crate::evaluation::nn::NN;
use crate::evaluation::nn_trace::trace_pos;
#[cfg(feature = "texel-tuning")]
use crate::evaluation::nn_trace::NNTrace;
use crate::move_generation::movegen;
use crate::move_generation::movegen::{pawn_east_targets, pawn_targets, pawn_west_targets};
use psqt_evaluation::psqt;
use psqt_evaluation::BLACK_INDEX;

pub const MG: usize = 0;
pub const EG: usize = 1;

pub struct EvaluationResult {
    pub final_eval: i16,
}

pub fn eval_game_state(
    g: &GameState,
    nn: &mut NN,
    #[cfg(feature = "texel-tuning")] nn_trace: &mut NNTrace,
) {
    let phase = g.get_phase().phase;
    #[cfg(feature = "texel-tuning")]
    {
        nn_trace.trace.mapv_inplace(|_| 0.);
        nn_trace.trace[trace_pos::PHASE] = phase as f32;
    }
    nn.internal_state.o_0.mapv_inplace(|_| 0.);
    nn.internal_state
        .evaluate_feature_1d(trace_pos::PHASE, phase as f32);

    let tempo = if g.get_color_to_move() == WHITE {
        1f32
    } else {
        -1f32
    };
    #[cfg(feature = "texel-tuning")]
    {
        nn_trace.trace[trace_pos::TEMPO_BONUS] = tempo;
    }
    nn.internal_state
        .evaluate_feature_1d(trace_pos::TEMPO_BONUS, tempo);
    //Initialize all attacks
    let (white_defended_by_minors, white_defended_by_majors) = (
        g.get_minor_attacks_from_side(WHITE),
        g.get_major_attacks_from_side(WHITE),
    );
    let white_defended = white_defended_by_minors
        | white_defended_by_majors
        | KING_ATTACKS[g.get_king_square(WHITE)];
    let (black_defended_by_minors, black_defended_by_majors) = (
        g.get_minor_attacks_from_side(BLACK),
        g.get_major_attacks_from_side(BLACK),
    );
    let black_defended = black_defended_by_minors
        | black_defended_by_majors
        | KING_ATTACKS[g.get_king_square(BLACK)];

    psqt(
        &g,
        WHITE,
        nn,
        #[cfg(feature = "texel-tuning")]
        nn_trace,
    );
    psqt(
        &g,
        BLACK,
        nn,
        #[cfg(feature = "texel-tuning")]
        nn_trace,
    );
    piece_values(
        true,
        g,
        nn,
        #[cfg(feature = "texel-tuning")]
        nn_trace,
    );
    piece_values(
        false,
        g,
        nn,
        #[cfg(feature = "texel-tuning")]
        nn_trace,
    );
    pawns(
        true,
        g,
        nn,
        white_defended,
        black_defended,
        #[cfg(feature = "texel-tuning")]
        nn_trace,
    );
    pawns(
        false,
        g,
        nn,
        black_defended,
        white_defended,
        #[cfg(feature = "texel-tuning")]
        nn_trace,
    );

    knights(
        true,
        g,
        nn,
        #[cfg(feature = "texel-tuning")]
        nn_trace,
    );
    knights(
        false,
        g,
        nn,
        #[cfg(feature = "texel-tuning")]
        nn_trace,
    );
    piecewise(
        true,
        g,
        nn,
        black_defended_by_minors,
        black_defended,
        #[cfg(feature = "texel-tuning")]
        nn_trace,
    );
    piecewise(
        false,
        g,
        nn,
        white_defended_by_minors,
        white_defended,
        #[cfg(feature = "texel-tuning")]
        nn_trace,
    );

    king(
        true,
        g,
        nn,
        #[cfg(feature = "texel-tuning")]
        nn_trace,
    );
    king(
        false,
        g,
        nn,
        #[cfg(feature = "texel-tuning")]
        nn_trace,
    );
}
pub fn is_guaranteed_draw(g: &GameState) -> bool {
    if g.get_piece_bb(PieceType::Pawn)
        | g.get_piece_bb(PieceType::Rook)
        | g.get_piece_bb(PieceType::Queen)
        > 0
    {
        return false;
    }
    let white_knights = g.get_piece(PieceType::Knight, WHITE).count_ones() as usize;
    let black_knights = g.get_piece(PieceType::Knight, BLACK).count_ones() as usize;
    let white_bishops = g.get_piece(PieceType::Bishop, WHITE).count_ones() as usize;
    let black_bishops = g.get_piece(PieceType::Bishop, BLACK).count_ones() as usize;
    if white_knights + white_bishops <= 2 && black_knights + black_bishops <= 2 {
        if white_knights + white_bishops < 2 || black_knights + black_bishops < 2 {
            if !(white_bishops == 2 && black_bishops == 0)
                && !(black_bishops == 2 && white_bishops == 0)
            {
                return true;
            }
        }
    }
    false
}
pub fn endgame_rescaling(g: &GameState, mut res_score: i16, phase: f64) -> i16 {
    let side_ahead = if res_score >= 0 { WHITE } else { BLACK };
    let side_losing = 1 - side_ahead;
    let winning_pawns = g.get_piece(PieceType::Pawn, side_ahead).count_ones() as usize;
    let mut res_phased = res_score;
    if winning_pawns <= 1 {
        let losing_minors = (g.get_piece(PieceType::Bishop, side_losing)
            | g.get_piece(PieceType::Knight, side_losing))
        .count_ones() as usize;
        let score = res_score.abs();
        let winnable_ahead =
            score.abs() >= PieceType::Pawn.to_piece_score() + PieceType::Knight.to_piece_score();

        if !winnable_ahead && (winning_pawns == 0) {
            let factor = 1. / 16.;
            res_phased = (res_score as f64 * factor) as i16;
        } else if !winnable_ahead
            && losing_minors >= 1
            && score.abs() + PieceType::Knight.to_piece_score() - PieceType::Pawn.to_piece_score()
                <= PieceType::Pawn.to_piece_score() + PieceType::Knight.to_piece_score()
        {
            let factor = 1. / 8.;
            res_phased = (res_score as f64 * factor) as i16;
        }
    }
    ((res_score as f64 * phase + res_phased as f64 * (128.0 - phase)) / 128.0) as i16
}
pub fn knights(
    white: bool,
    g: &GameState,
    nn: &mut NN,
    #[cfg(feature = "texel-tuning")] nn_trace: &mut NNTrace,
) {
    let side = if white { WHITE } else { BLACK };
    let side_mult = if white { 1f32 } else { -1f32 };

    let my_pawn_attacks = pawn_targets(side, g.get_piece(PieceType::Pawn, side));
    let supported_knights = g.get_piece(PieceType::Knight, side) & my_pawn_attacks;
    let supported_knights_amount = supported_knights.count_ones() as i16;

    #[cfg(feature = "texel-tuning")]
    {
        nn_trace.trace[trace_pos::KNIGHT_SUPPORTED] += supported_knights_amount as f32 * side_mult;
    }
    nn.internal_state.evaluate_feature_1d(
        trace_pos::KNIGHT_SUPPORTED,
        supported_knights_amount as f32 * side_mult,
    );

    let mut supp = supported_knights;
    while supp != 0u64 {
        let mut idx = supp.trailing_zeros() as usize;
        supp &= not_file(idx % 8);
        let mut front_span = if white {
            bitboards::w_front_span(square(idx))
        } else {
            bitboards::b_front_span(square(idx))
        };
        front_span = bitboards::west_one(front_span) | bitboards::east_one(front_span);
        if g.get_piece(PieceType::Pawn, 1 - side) & front_span == 0u64 {
            if !white {
                idx = BLACK_INDEX[idx];
            }
            let mut position_in_arr = trace_pos::KNIGHT_OUTPOST_TABLE + 8 * (idx / 8) + idx % 8;
            #[cfg(feature = "nn-eval")]
            {
                nn_trace.trace[position_in_arr] += side_mult;
            }
            nn.internal_state
                .evaluate_feature_1d(position_in_arr, side_mult);
        }
    }
}

pub fn piecewise(
    white: bool,
    g: &GameState,
    nn: &mut NN,
    defended_by_minors: u64,
    defended_squares: u64,
    #[cfg(feature = "texel-tuning")] nn_trace: &mut NNTrace,
) {
    let side = if white { WHITE } else { BLACK };
    let side_mult = if white { 1f32 } else { -1f32 };

    let all_pieces = g.get_all_pieces();
    let my_pieces = g.get_pieces_from_side(side);
    let enemy_king_idx = g.get_king_square(1 - side);
    let enemy_king_attackable = if white {
        KING_ZONE_BLACK[enemy_king_idx]
    } else {
        KING_ZONE_WHITE[enemy_king_idx]
    } & !defended_by_minors;

    let knight_checks = KNIGHT_ATTACKS[enemy_king_idx];
    let bishop_checks = PieceType::Bishop.attacks(enemy_king_idx, all_pieces);
    let rook_checks = PieceType::Rook.attacks(enemy_king_idx, all_pieces);

    //Knights
    let mut knight_attackers = 0;
    let mut knight_attacked_sq_sum = 0;
    let mut knight_safe_checkers = 0;
    let mut knights = g.get_piece(PieceType::Knight, side);
    while knights != 0u64 {
        let idx = knights.trailing_zeros() as usize;

        let targets = PieceType::Knight.attacks(idx, all_pieces) & !my_pieces;
        let mobility = targets.count_ones() as usize;

        let has_safe_check = (targets & knight_checks & !defended_squares) != 0u64;
        knight_safe_checkers += has_safe_check as usize;

        let enemy_king_attacks = targets & enemy_king_attackable;
        knight_attacked_sq_sum += enemy_king_attacks.count_ones();

        if has_safe_check || enemy_king_attacks != 0u64 {
            knight_attackers += 1;
        }

        #[cfg(feature = "texel-tuning")]
        {
            nn_trace.trace[trace_pos::KNIGHT_MOBILITY + mobility] += side_mult;
        }
        nn.internal_state
            .evaluate_feature_1d(trace_pos::KNIGHT_MOBILITY + mobility, side_mult);
        knights ^= square(idx);
    }
    #[cfg(feature = "texel-tuning")]
    {
        nn_trace.trace[trace_pos::KNIGHT_ATTACKED_SQ + side] += knight_attacked_sq_sum as f32;
        nn_trace.trace[trace_pos::KNIGHT_SAFE_CHECK + side] += knight_safe_checkers as f32;
    }
    nn.internal_state.evaluate_feature_1d(
        trace_pos::KNIGHT_ATTACKED_SQ + side,
        knight_attacked_sq_sum as f32,
    );

    nn.internal_state.evaluate_feature_1d(
        trace_pos::KNIGHT_SAFE_CHECK + side,
        knight_safe_checkers as f32,
    );

    //Bishops
    let mut bishop_attackers = 0;
    let mut bishop_attacked_sq_sum = 0;
    let mut bishop_safe_checkers = 0;
    let mut bishop_xray_king = 0;

    let mut bishops = g.get_piece(PieceType::Bishop, side);
    while bishops != 0u64 {
        let idx = bishops.trailing_zeros() as usize;
        let bishop_attack = PieceType::Bishop.attacks(idx, all_pieces ^ square(enemy_king_idx));
        if (FREEFIELD_BISHOP_ATTACKS[idx] & g.get_piece(PieceType::King, 1 - side)) != 0u64
            && (movegen::xray_bishop_attacks(bishop_attack, all_pieces, all_pieces, idx)
                & g.get_piece(PieceType::King, 1 - side))
                != 0u64
        {
            bishop_xray_king += 1;
        }
        let diagonally_adjacent_pawns =
            (DIAGONALLY_ADJACENT[idx] & g.get_piece(PieceType::Pawn, side)).count_ones() as usize;

        let targets = bishop_attack & !my_pieces;
        let mobility = targets.count_ones() as usize;

        let has_safe_check = (targets & bishop_checks & !defended_squares) != 0u64;
        bishop_safe_checkers += has_safe_check as usize;

        let enemy_king_attacks = targets & enemy_king_attackable;
        bishop_attacked_sq_sum += enemy_king_attacks.count_ones();
        if has_safe_check || enemy_king_attacks != 0u64 {
            bishop_attackers += 1;
        }

        #[cfg(feature = "texel-tuning")]
        {
            nn_trace.trace
                [trace_pos::DIAGONALLY_ADJACENT_SQUARES_WITHPAWNS + diagonally_adjacent_pawns] +=
                side_mult;
            nn_trace.trace[trace_pos::BISHOP_MOBILITY + mobility] += side_mult;
        }
        nn.internal_state.evaluate_feature_1d(
            trace_pos::DIAGONALLY_ADJACENT_SQUARES_WITHPAWNS + diagonally_adjacent_pawns,
            side_mult,
        );
        nn.internal_state
            .evaluate_feature_1d(trace_pos::BISHOP_MOBILITY + mobility, side_mult);
        bishops ^= square(idx);
    }
    #[cfg(feature = "texel-tuning")]
    {
        nn_trace.trace[trace_pos::BISHOP_XRAY_KING] += bishop_xray_king as f32 * side_mult;
        nn_trace.trace[trace_pos::BISHOP_ATTACKED_SQ + side] += bishop_attacked_sq_sum as f32;
        nn_trace.trace[trace_pos::BISHOP_SAFE_CHECK + side] += bishop_safe_checkers as f32;
    }
    nn.internal_state.evaluate_feature_1d(
        trace_pos::BISHOP_XRAY_KING,
        bishop_xray_king as f32 * side_mult,
    );
    nn.internal_state.evaluate_feature_1d(
        trace_pos::BISHOP_ATTACKED_SQ + side,
        bishop_attacked_sq_sum as f32,
    );
    nn.internal_state.evaluate_feature_1d(
        trace_pos::BISHOP_SAFE_CHECK + side,
        bishop_safe_checkers as f32,
    );

    //Rooks
    let mut rook_attackers = 0;
    let mut rook_attacked_sq_sum = 0;
    let mut rook_safe_checkers = 0;
    let mut rook_xray_king = 0;

    let (mut rooks_onopen, mut rooks_on_semi_open, mut rooks_onseventh) = (0i16, 0i16, 0i16);

    let mut rooks = g.get_piece(PieceType::Rook, side);
    while rooks != 0u64 {
        let idx = rooks.trailing_zeros() as usize;

        let rook_attack = PieceType::Rook.attacks(idx, all_pieces ^ square(enemy_king_idx));
        if (FREEFIELD_ROOK_ATTACKS[idx] & g.get_piece(PieceType::King, 1 - side)) != 0u64
            && (movegen::xray_rook_attacks(rook_attack, all_pieces, all_pieces, idx)
                & g.get_piece(PieceType::King, 1 - side))
                != 0u64
        {
            rook_xray_king += 1;
        }
        if if white { idx / 8 == 6 } else { idx / 8 == 1 } {
            rooks_onseventh += 1;
        }
        if FILES[idx % 8] & g.get_piece_bb(PieceType::Pawn) == 0u64 {
            rooks_onopen += 1;
        } else if (FILES[idx % 8] & g.get_piece(PieceType::Pawn, 1 - side)).count_ones() == 1
            && (FILES[idx % 8] & g.get_piece(PieceType::Pawn, side)) == 0u64
        {
            rooks_on_semi_open += 1;
        }

        let targets = rook_attack & !my_pieces;
        let mobility = targets.count_ones() as usize;

        let has_safe_check = (targets & rook_checks & !defended_squares) != 0u64;
        rook_safe_checkers += has_safe_check as usize;

        let enemy_king_attacks = targets & enemy_king_attackable;
        rook_attacked_sq_sum += enemy_king_attacks.count_ones();
        if has_safe_check || enemy_king_attacks != 0u64 {
            rook_attackers += 1;
        }

        #[cfg(feature = "texel-tuning")]
        {
            nn_trace.trace[trace_pos::ROOK_MOBILITY + mobility] += side_mult;
        }
        nn.internal_state
            .evaluate_feature_1d(trace_pos::ROOK_MOBILITY + mobility, side_mult);
        rooks ^= square(idx);
    }
    #[cfg(feature = "texel-tuning")]
    {
        nn_trace.trace[trace_pos::ROOK_ATTACKED_SQ + side] += rook_attacked_sq_sum as f32;
        nn_trace.trace[trace_pos::ROOK_SAFE_CHECK + side] += rook_safe_checkers as f32;
        nn_trace.trace[trace_pos::ROOK_ON_OPEN] += rooks_onopen as f32 * side_mult;
        nn_trace.trace[trace_pos::ROOK_ON_SEMI_OPEN] += rooks_on_semi_open as f32 * side_mult;
        nn_trace.trace[trace_pos::ROOK_ON_SEVENTH] += rooks_onseventh as f32 * side_mult;
        nn_trace.trace[trace_pos::ROOK_XRAY_KING] += rook_xray_king as f32 * side_mult;
    }
    nn.internal_state.evaluate_feature_1d(
        trace_pos::ROOK_ATTACKED_SQ + side,
        rook_attacked_sq_sum as f32,
    );
    nn.internal_state
        .evaluate_feature_1d(trace_pos::ROOK_SAFE_CHECK + side, rook_safe_checkers as f32);
    nn.internal_state
        .evaluate_feature_1d(trace_pos::ROOK_ON_OPEN, rooks_onopen as f32 * side_mult);
    nn.internal_state.evaluate_feature_1d(
        trace_pos::ROOK_ON_SEMI_OPEN,
        rooks_on_semi_open as f32 * side_mult,
    );
    nn.internal_state.evaluate_feature_1d(
        trace_pos::ROOK_ON_SEVENTH,
        rooks_onseventh as f32 * side_mult,
    );
    nn.internal_state
        .evaluate_feature_1d(trace_pos::ROOK_XRAY_KING, rook_xray_king as f32 * side_mult);

    //Queens
    let mut queen_attackers: i16 = 0;
    let mut queen_attacked_sq_sum = 0;
    let mut queen_safe_checkers = 0;
    let mut queen_xray_king: i16 = 0;
    let (mut queens_onopen, mut queens_on_semi_open) = (0i16, 0i16);

    let mut queens = g.get_piece(PieceType::Queen, side);
    while queens != 0u64 {
        let idx = queens.trailing_zeros() as usize;

        let rooklike_attacks = PieceType::Rook.attacks(idx, all_pieces ^ square(enemy_king_idx));
        let bishoplike_attacks =
            PieceType::Bishop.attacks(idx, all_pieces ^ square(enemy_king_idx));
        let queen_attack = rooklike_attacks | bishoplike_attacks;
        if (FREEFIELD_BISHOP_ATTACKS[idx] & g.get_piece(PieceType::King, 1 - side)) != 0u64
            && (movegen::xray_bishop_attacks(bishoplike_attacks, all_pieces, all_pieces, idx)
                & g.get_piece(PieceType::King, 1 - side))
                != 0u64
            || (FREEFIELD_ROOK_ATTACKS[idx] & g.get_piece(PieceType::King, 1 - side)) != 0u64
                && (movegen::xray_rook_attacks(rooklike_attacks, all_pieces, all_pieces, idx)
                    & g.get_piece(PieceType::King, 1 - side))
                    != 0u64
        {
            queen_xray_king += 1;
        }

        if FILES[idx % 8] & g.get_piece_bb(PieceType::Pawn) == 0u64 {
            queens_onopen += 1;
        } else if (FILES[idx % 8] & g.get_piece(PieceType::Pawn, 1 - side)).count_ones() == 1
            && (FILES[idx % 8] & g.get_piece(PieceType::Pawn, side)) == 0u64
        {
            queens_on_semi_open += 1;
        }

        let targets = queen_attack & !my_pieces;

        let mobility = targets.count_ones() as usize;

        let has_safe_check = (targets & (bishop_checks | rook_checks) & !defended_squares) != 0u64;
        queen_safe_checkers += has_safe_check as usize;
        let enemy_king_attacks = targets & enemy_king_attackable;
        queen_attacked_sq_sum += enemy_king_attacks.count_ones();
        if has_safe_check || enemy_king_attacks != 0u64 {
            queen_attackers += 1;
        }

        #[cfg(feature = "texel-tuning")]
        {
            nn_trace.trace[trace_pos::QUEEN_MOBILITY + mobility] += side_mult;
        }
        nn.internal_state
            .evaluate_feature_1d(trace_pos::QUEEN_MOBILITY + mobility, side_mult);
        queens ^= square(idx);
    }
    let sum_attackers =
        (knight_attackers + bishop_attackers + rook_attackers + queen_attackers).min(7) as f32;
    #[cfg(feature = "texel-tuning")]
    {
        nn_trace.trace[trace_pos::QUEEN_ON_OPEN] += queens_onopen as f32 * side_mult;
        nn_trace.trace[trace_pos::QUEEN_ON_SEMI_OPEN] += queens_on_semi_open as f32 * side_mult;
        nn_trace.trace[trace_pos::QUEEN_XRAY_KING] += queen_xray_king as f32 * side_mult;
        nn_trace.trace[trace_pos::QUEEN_ATTACKED_SQ + side] += queen_attacked_sq_sum as f32;
        nn_trace.trace[trace_pos::QUEEN_SAFE_CHECK + side] += queen_safe_checkers as f32;
        nn_trace.trace[trace_pos::ATTACKERS + side] = sum_attackers
    }
    nn.internal_state
        .evaluate_feature_1d(trace_pos::QUEEN_ON_OPEN, queens_onopen as f32 * side_mult);
    nn.internal_state.evaluate_feature_1d(
        trace_pos::QUEEN_ON_SEMI_OPEN,
        queens_on_semi_open as f32 * side_mult,
    );
    nn.internal_state.evaluate_feature_1d(
        trace_pos::QUEEN_XRAY_KING,
        queen_xray_king as f32 * side_mult,
    );
    nn.internal_state.evaluate_feature_1d(
        trace_pos::QUEEN_ATTACKED_SQ + side,
        queen_attacked_sq_sum as f32,
    );
    nn.internal_state.evaluate_feature_1d(
        trace_pos::QUEEN_SAFE_CHECK + side,
        queen_safe_checkers as f32,
    );
    nn.internal_state
        .evaluate_feature_1d(trace_pos::ATTACKERS + side, sum_attackers);
}

pub fn king(
    white: bool,
    g: &GameState,
    nn: &mut NN,
    #[cfg(feature = "texel-tuning")] nn_trace: &mut NNTrace,
) {
    let side = if white { WHITE } else { BLACK };
    let mut side_mult = if white { 1f32 } else { -1f32 };
    let mut pawn_shield = if white {
        SHIELDING_PAWNS_WHITE[g.get_king_square(side)]
    } else {
        SHIELDING_PAWNS_BLACK[g.get_king_square(side)]
    };
    let mut king_front_span = if white {
        bitboards::w_front_span(g.get_piece(PieceType::King, side))
    } else {
        bitboards::b_front_span(g.get_piece(PieceType::King, side))
    };
    king_front_span |= bitboards::west_one(king_front_span) | bitboards::east_one(king_front_span);
    let file = g.get_king_square(side) % 8;
    if file == 7 {
        king_front_span |= bitboards::west_one(king_front_span);
    } else if file == 0 {
        king_front_span |= bitboards::east_one(king_front_span);
    }
    let mut shields_missing = 0;
    let mut shields_on_open_missing = 0;
    if g.get_full_moves() >= 1 {
        while pawn_shield != 0u64 {
            let idx = pawn_shield.trailing_zeros() as usize;
            if g.get_piece(PieceType::Pawn, side) & pawn_shield & FILES[idx % 8] == 0u64 {
                shields_missing += 1;
                if g.get_piece(PieceType::Pawn, 1 - side) & FILES[idx % 8] & king_front_span == 0u64
                {
                    shields_on_open_missing += 1;
                }
            }
            pawn_shield &= !FILES[idx % 8];
        }
    }
    #[cfg(feature = "texel-tuning")]
    {
        nn_trace.trace[trace_pos::SHIELDING_PAWN_MISSING + shields_missing] += side_mult;
        nn_trace.trace[trace_pos::SHIELDING_PAWN_ONOPEN_MISSING + shields_on_open_missing] +=
            side_mult;
    }
    nn.internal_state.evaluate_feature_1d(
        trace_pos::SHIELDING_PAWN_MISSING + shields_missing,
        side_mult,
    );
    nn.internal_state.evaluate_feature_1d(
        trace_pos::SHIELDING_PAWN_ONOPEN_MISSING + shields_on_open_missing,
        side_mult,
    );
}

pub fn get_distance(sq: isize, sq2: isize) -> usize {
    (sq / 8 - sq2 / 8).abs().max((sq % 8 - sq2 % 8).abs()) as usize
}

pub fn pawns(
    white: bool,
    g: &GameState,
    nn: &mut NN,
    defended: u64,
    enemy_defended: u64,
    #[cfg(feature = "texel-tuning")] nn_trace: &mut NNTrace,
) {
    let side = if white { WHITE } else { BLACK };
    let side_mult = if white { 1f32 } else { -1f32 };

    let empty = !g.get_all_pieces();
    let pawns = g.get_piece(PieceType::Pawn, side);
    let enemy_pawns = g.get_piece(PieceType::Pawn, 1 - side);
    //Bitboards
    let pawn_file_fill = bitboards::file_fill(pawns);
    let front_span = if white {
        bitboards::w_front_span(pawns)
    } else {
        bitboards::b_front_span(pawns)
    };
    let mut enemy_front_spans = if white {
        bitboards::b_front_span(enemy_pawns)
    } else {
        bitboards::w_front_span(enemy_pawns)
    };
    enemy_front_spans |=
        bitboards::west_one(enemy_front_spans) | bitboards::east_one(enemy_front_spans);
    let (my_west_attacks, my_east_attacks, enemy_pawn_attacks) = (
        pawn_west_targets(side, pawns),
        pawn_east_targets(side, pawns),
        pawn_targets(1 - side, enemy_pawns),
    );
    let my_pawn_attacks = my_west_attacks | my_east_attacks;
    let (my_pawn_pushes, my_pawn_double_pushes) = (
        movegen::single_push_pawn_targets(side, pawns, empty),
        movegen::double_push_pawn_targets(side, pawns, empty),
    );

    let is_attackable = bitboards::west_one(front_span) | bitboards::east_one(front_span);
    let enemy_pieces = g.get_pieces_from_side(1 - side);

    let doubled_pawns = (pawns & front_span).count_ones() as i16;
    let isolated_pawns =
        (pawns & !bitboards::west_one(pawn_file_fill) & !bitboards::east_one(pawn_file_fill))
            .count_ones() as i16;
    let backward_pawns = (if white { pawns << 8 } else { pawns >> 8 }
        & enemy_pawn_attacks
        & !is_attackable
        & !enemy_pawns)
        .count_ones() as i16;

    let mut supported_pawns = pawns & my_pawn_attacks;
    while supported_pawns != 0u64 {
        let mut index = supported_pawns.trailing_zeros() as usize;
        supported_pawns ^= square(index);
        if !white {
            index = BLACK_INDEX[index];
        }
        let position_in_array = trace_pos::PAWN_SUPPORTED + 8 * (index / 8) + index % 8;
        #[cfg(feature = "texel-tuning")]
        {
            nn_trace.trace[position_in_array] += side_mult;
        }
        nn.internal_state
            .evaluate_feature_1d(position_in_array, side_mult);
    }
    let center_attack_pawns = (pawns
        & if white {
            bitboards::south_east_one(INNER_CENTER) | bitboards::south_west_one(INNER_CENTER)
        } else {
            bitboards::north_east_one(INNER_CENTER) | bitboards::north_west_one(INNER_CENTER)
        })
    .count_ones() as i16;
    let pawn_mobility = (my_west_attacks.count_ones()
        + my_east_attacks.count_ones()
        + my_pawn_pushes.count_ones()
        + my_pawn_double_pushes.count_ones()) as i16;

    #[cfg(feature = "texel-tuning")]
    {
        nn_trace.trace[trace_pos::PAWN_DOUBLED] += doubled_pawns as f32 * side_mult;
        nn_trace.trace[trace_pos::PAWN_ISOLATED] += isolated_pawns as f32 * side_mult;
        nn_trace.trace[trace_pos::PAWN_BACKWARD] += backward_pawns as f32 * side_mult;
        nn_trace.trace[trace_pos::PAWN_ATTACK_CENTER] += center_attack_pawns as f32 * side_mult;
        nn_trace.trace[trace_pos::PAWN_MOBILITY] += pawn_mobility as f32 * side_mult;
    }
    nn.internal_state
        .evaluate_feature_1d(trace_pos::PAWN_DOUBLED, doubled_pawns as f32 * side_mult);
    nn.internal_state
        .evaluate_feature_1d(trace_pos::PAWN_ISOLATED, isolated_pawns as f32 * side_mult);
    nn.internal_state
        .evaluate_feature_1d(trace_pos::PAWN_BACKWARD, backward_pawns as f32 * side_mult);
    nn.internal_state.evaluate_feature_1d(
        trace_pos::PAWN_ATTACK_CENTER,
        center_attack_pawns as f32 * side_mult,
    );
    nn.internal_state
        .evaluate_feature_1d(trace_pos::PAWN_MOBILITY, pawn_mobility as f32 * side_mult);

    //Passers
    let mut passed_pawns: u64 = pawns & !enemy_front_spans;
    let mut weak_passers = 0;
    let behind_passers = if white {
        bitboards::b_front_span(passed_pawns)
    } else {
        bitboards::w_front_span(passed_pawns)
    };
    let rooks_support_passer = (behind_passers & g.get_rook_like_bb(side)).count_ones() as i16;
    let enemy_rooks_attack_passer =
        (behind_passers & g.get_rook_like_bb(1 - side)).count_ones() as i16;

    #[cfg(feature = "texel-tuning")]
    {
        nn_trace.trace[trace_pos::ROOK_BEHIND_SUPPORT_PASSER] +=
            rooks_support_passer as f32 * side_mult;
        nn_trace.trace[trace_pos::ROOK_BEHIND_ENEMY_PASSER] +=
            enemy_rooks_attack_passer as f32 * side_mult;
    }
    nn.internal_state.evaluate_feature_1d(
        trace_pos::ROOK_BEHIND_SUPPORT_PASSER,
        rooks_support_passer as f32 * side_mult,
    );
    nn.internal_state.evaluate_feature_1d(
        trace_pos::ROOK_BEHIND_ENEMY_PASSER,
        enemy_rooks_attack_passer as f32 * side_mult,
    );

    while passed_pawns != 0u64 {
        let idx = passed_pawns.trailing_zeros() as usize;
        //Passed and blocked
        #[cfg(feature = "texel-tuning")]
        {
            nn_trace.trace[trace_pos::PAWN_PASSED + GameState::relative_rank(side, idx)] +=
                side_mult
        }
        nn.internal_state.evaluate_feature_1d(
            trace_pos::PAWN_PASSED + GameState::relative_rank(side, idx),
            side_mult,
        );
        //A weak passer is an attacked and not defended passer
        let weak_passer = square(idx) & enemy_defended != 0u64 && square(idx) & defended == 0u64;
        if weak_passer {
            //Weak passer
            weak_passers += 1;
        }
        //An unblocked passer is a) not weak b) all the squares until conversions are either not attacked or defended and unoccupied or attacked
        if !weak_passer
            && if white {
                bitboards::w_front_span(square(idx))
            } else {
                bitboards::b_front_span(square(idx))
            } & (enemy_defended | enemy_pieces)
                & !defended
                == 0u64
        {
            //Passed and not blocked
            #[cfg(feature = "texel-tuning")]
            {
                nn_trace.trace
                    [trace_pos::PAWN_PASSED_NOTBLOCKED + GameState::relative_rank(side, idx)] +=
                    side_mult;
            }
            nn.internal_state.evaluate_feature_1d(
                trace_pos::PAWN_PASSED_NOTBLOCKED + GameState::relative_rank(side, idx),
                side_mult,
            );
        }

        //Distance to kings
        let d_myking = get_distance(idx as isize, g.get_king_square(side) as isize);
        let d_enemyking = get_distance(idx as isize, g.get_king_square(1 - side) as isize);
        let sub_dist = ((d_myking as isize - d_enemyking as isize) + 6) as usize;
        #[cfg(feature = "texel-tuning")]
        {
            nn_trace.trace[trace_pos::PAWN_PASSED_KINGDISTANCE + d_myking - 1] += side_mult;
            nn_trace.trace[trace_pos::PAWN_PASSED_ENEMYKINGDISTANCE + d_enemyking - 1] += side_mult;
            nn_trace.trace[trace_pos::PAWN_PASSED_SUBKINGDISTANCE + sub_dist] += side_mult;
        }
        nn.internal_state.evaluate_feature_1d(
            trace_pos::PAWN_PASSED_KINGDISTANCE + d_myking - 1,
            side_mult,
        );
        nn.internal_state.evaluate_feature_1d(
            trace_pos::PAWN_PASSED_ENEMYKINGDISTANCE + d_enemyking - 1,
            side_mult,
        );
        nn.internal_state
            .evaluate_feature_1d(trace_pos::PAWN_PASSED_SUBKINGDISTANCE + sub_dist, side_mult);
        passed_pawns ^= square(idx);
    }
    #[cfg(feature = "texel-tuning")]
    {
        nn_trace.trace[trace_pos::PAWN_PASSED_WEAK] += weak_passers as f32 * side_mult;
    }
    nn.internal_state
        .evaluate_feature_1d(trace_pos::PAWN_PASSED_WEAK, weak_passers as f32 * side_mult);
}

pub fn piece_values(
    white: bool,
    g: &GameState,
    nn: &mut NN,
    #[cfg(feature = "texel-tuning")] nn_trace: &mut NNTrace,
) {
    let side = if white { WHITE } else { BLACK };
    let side_mult = if white { 1f32 } else { -1f32 };

    let my_pawns = g.get_piece(PieceType::Pawn, side).count_ones() as i16;
    let my_knights = g.get_piece(PieceType::Knight, side).count_ones() as i16;
    let my_bishops = g.get_piece(PieceType::Bishop, side).count_ones() as i16;
    let my_rooks = g.get_piece(PieceType::Rook, side).count_ones() as i16;
    let my_queens = g.get_piece(PieceType::Queen, side).count_ones() as i16;
    let pawns_on_board = g.get_piece_bb(PieceType::Pawn).count_ones() as usize;

    #[cfg(feature = "texel-tuning")]
    {
        nn_trace.trace[trace_pos::PAWNS] += my_pawns as f32 * side_mult;
        nn_trace.trace[trace_pos::KNIGHT_VALUE_WITH_PAWNS] = pawns_on_board as f32;
        nn_trace.trace[trace_pos::KNIGHTS] += my_knights as f32 * side_mult;
        nn_trace.trace[trace_pos::BISHOPS] += my_bishops as f32 * side_mult;
        if my_bishops > 1 {
            nn_trace.trace[trace_pos::BISHOP_BONUS] += side_mult;
        }
        nn_trace.trace[trace_pos::ROOKS] += my_rooks as f32 * side_mult;
        nn_trace.trace[trace_pos::QUEENS] += my_queens as f32 * side_mult;
    }
    nn.internal_state
        .evaluate_feature_1d(trace_pos::PAWNS, my_pawns as f32 * side_mult);
    nn.internal_state
        .evaluate_feature_1d(trace_pos::KNIGHT_VALUE_WITH_PAWNS, pawns_on_board as f32);
    nn.internal_state
        .evaluate_feature_1d(trace_pos::KNIGHTS, my_knights as f32 * side_mult);
    nn.internal_state
        .evaluate_feature_1d(trace_pos::BISHOPS, my_bishops as f32 * side_mult);
    if my_bishops > 1 {
        nn.internal_state
            .evaluate_feature_1d(trace_pos::BISHOP_BONUS, side_mult);
    }
    nn.internal_state
        .evaluate_feature_1d(trace_pos::ROOKS, my_rooks as f32 * side_mult);
    nn.internal_state
        .evaluate_feature_1d(trace_pos::QUEENS, my_queens as f32 * side_mult);
}
