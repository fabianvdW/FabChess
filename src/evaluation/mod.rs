pub mod params;
pub mod psqt_evaluation;
use super::bitboards;
use super::board_representation::game_state::{
    GameState, PieceType, BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE,
};
#[cfg(feature = "display-eval")]
use super::logging::log;
use super::move_generation::movegen;
use crate::board_representation::game_state_attack_container::{
    GameStateAttackContainer, MGSA_BISHOP, MGSA_KNIGHT, MGSA_QUEEN, MGSA_ROOKS,
};
use crate::move_generation::movegen::{bishop_attack, knight_attack, rook_attack};
#[cfg(feature = "texel-tuning")]
use crate::tuning::trace::Trace;
use params::*;
use psqt_evaluation::psqt;
use psqt_evaluation::BLACK_INDEX;
pub const MG: usize = 0;
pub const EG: usize = 1;

const MG_LIMIT: i16 = 9100;
const EG_LIMIT: i16 = 2350;

pub struct EvaluationResult {
    pub phase: f64,
    pub final_eval: i16,
    #[cfg(feature = "texel-tuning")]
    pub trace: Trace,
}

pub fn eval_game_state_from_null(g: &GameState) -> EvaluationResult {
    let mgsac = GameStateAttackContainer::from_state(g);
    eval_game_state(g, &mgsac)
}
pub fn eval_game_state(g: &GameState, attacks: &GameStateAttackContainer) -> EvaluationResult {
    #[cfg(feature = "display-eval")]
    {
        log(&format!("Evaluating GameState fen: {}\n", g.to_fen()));
    }
    let mut result = EvaluationResult {
        phase: 0.,
        final_eval: 0,
        #[cfg(feature = "texel-tuning")]
        trace: Trace::default(),
    };
    let phase = calculate_phase(g);
    #[cfg(feature = "texel-tuning")]
    {
        result.trace.phase = phase;
    }
    let (psqt_mg, psqt_eg) = if cfg!(feature = "display-eval") || cfg!(feature = "texel-tuning") {
        let (psqt_w, psqt_b) = (
            psqt(true, &g.pieces, &mut result),
            psqt(false, &g.pieces, &mut result),
        );
        (psqt_w.0 - psqt_b.0, psqt_w.1 - psqt_b.1)
    } else {
        (g.psqt_mg, g.psqt_eg)
    };

    #[cfg(feature = "display-eval")]
    {
        log(&format!("\nMG PSQT Sum: {}\n", psqt_mg));
        log(&format!("EG PSQT Sum: {}\n", psqt_eg));
    }
    let (knights_w, knights_b) = (
        knights(true, g, &mut result, attacks),
        knights(false, g, &mut result, attacks),
    );
    #[cfg(feature = "display-eval")]
    {
        log(&format!(
            "\nMG Knight Sum: {} - {} -> {}\n",
            knights_w.0,
            knights_b.0,
            knights_w.0 - knights_b.0
        ));
        log(&format!(
            "EG Knight Sum: {} - {} -> {}\n",
            knights_w.1,
            knights_b.1,
            knights_w.1 - knights_b.1
        ));
    }
    let (piecewise_w, piecewise_b) = (
        piecewise(true, g, &mut result, attacks),
        piecewise(false, g, &mut result, attacks),
    );
    #[cfg(feature = "display-eval")]
    {
        log(&format!(
            "\nMG Piecewise Sum: {} - {} -> {}\n",
            piecewise_w.0,
            piecewise_b.0,
            piecewise_w.0 - piecewise_b.0
        ));
        log(&format!(
            "EG Piecewise Sum: {} - {} -> {}\n",
            piecewise_w.1,
            piecewise_b.1,
            piecewise_w.1 - piecewise_b.1
        ));
    }
    let (king_w, king_b) = (king(true, g, &mut result), king(false, g, &mut result));
    #[cfg(feature = "display-eval")]
    {
        log(&format!(
            "\nMG King Sum: {} - {} -> {}\n",
            king_w.0,
            king_b.0,
            king_w.0 - king_b.0
        ));
        log(&format!(
            "EG King Sum: {} - {} -> {}\n",
            king_w.1,
            king_b.1,
            king_w.1 - king_b.1
        ));
    }
    let (pawns_w, pawns_b) = (
        pawns(true, g, &mut result, attacks),
        pawns(false, g, &mut result, attacks),
    );
    #[cfg(feature = "display-eval")]
    {
        log(&format!(
            "\nMG Pawn Sum: {} - {} -> {}\n",
            pawns_w.0,
            pawns_b.0,
            pawns_w.0 - pawns_b.0
        ));
        log(&format!(
            "EG Pawn Sum: {} - {} -> {}\n",
            pawns_w.1,
            pawns_b.1,
            pawns_w.1 - pawns_b.1
        ));
    }
    let (pieces_w, pieces_b) = (
        piece_values(true, g, &mut result),
        piece_values(false, g, &mut result),
    );
    #[cfg(feature = "display-eval")]
    {
        log(&format!(
            "\nMG Piece value Sum: {} - {} -> {}\n",
            pieces_w.0,
            pieces_b.0,
            pieces_w.0 - pieces_b.0
        ));
        log(&format!(
            "EG Piece value Sum: {} - {} -> {}\n",
            pieces_w.1,
            pieces_b.1,
            pieces_w.1 - pieces_b.1
        ));
    }
    #[cfg(feature = "display-eval")]
    {
        let tempo_mg;
        let tempo_eg;
        if g.color_to_move == WHITE {
            tempo_mg = TEMPO_BONUS_MG;
            tempo_eg = TEMPO_BONUS_EG;
        } else {
            tempo_mg = -TEMPO_BONUS_MG;
            tempo_eg = -TEMPO_BONUS_EG;
        }
        log(&format!("\nTempo:({} , {})\n", tempo_mg, tempo_eg,));
    }
    let mut mg_eval = (knights_w.0 + piecewise_w.0 + king_w.0 + pawns_w.0 + pieces_w.0)
        - (knights_b.0 + piecewise_b.0 + king_b.0 + pawns_b.0 + pieces_b.0)
        + psqt_mg;
    let mut eg_eval = (knights_w.1 + piecewise_w.1 + king_w.1 + pawns_w.1 + pieces_w.1)
        - (knights_b.1 + piecewise_b.1 + king_b.1 + pawns_b.1 + pieces_b.1)
        + psqt_eg;
    if g.color_to_move == WHITE {
        mg_eval += TEMPO_BONUS_MG;
        eg_eval += TEMPO_BONUS_EG;
    } else {
        mg_eval += -TEMPO_BONUS_MG;
        eg_eval += -TEMPO_BONUS_EG;
    }
    #[cfg(feature = "texel-tuning")]
    {
        result.trace.tempo_bonus[g.color_to_move] = 1;
    }
    eg_eval = (f64::from(eg_eval) / 1.5) as i16;
    //Phasing is done the same way stockfish does it
    let res = ((f64::from(mg_eval) * phase + f64::from(eg_eval) * (128.0 - phase)) / 128.0) as i16;
    #[cfg(feature = "display-eval")]
    {
        log(&format!(
            "\nMG Sum: {} + {} + {} + {} + {} + {} + {} -> {}\n",
            psqt_mg,
            knights_w.0 - knights_b.0,
            piecewise_w.0 - piecewise_b.0,
            king_w.0 - king_b.0,
            pawns_w.0 - pawns_b.0,
            pieces_w.0 - pieces_b.0,
            if g.color_to_move == 0 {
                TEMPO_BONUS_MG
            } else {
                -TEMPO_BONUS_MG
            },
            mg_eval
        ));
        log(&format!(
            "\nEG Sum: ({} + {} + {} + {} + {} + {} + {}) /1.5 -> {}\n",
            psqt_eg,
            knights_w.1 - knights_b.1,
            piecewise_w.1 - piecewise_b.1,
            king_w.1 - king_b.1,
            pawns_w.1 - pawns_b.1,
            pieces_w.1 - pieces_b.1,
            if g.color_to_move == 0 {
                TEMPO_BONUS_EG
            } else {
                -TEMPO_BONUS_EG
            },
            eg_eval
        ));
        log(&format!("Phase: {}\n", phase));
        log(&format!(
            "\nFinal Result: ({} * {} + {} * (128.0 - {}))/128.0 -> {}",
            mg_eval, phase, eg_eval, phase, res,
        ));
    }
    result.phase = phase;
    result.final_eval = res;
    result
}

pub fn knights(
    white: bool,
    g: &GameState,
    _eval: &mut EvaluationResult,
    attack_container: &GameStateAttackContainer,
) -> (i16, i16) {
    let (mut mg_res, mut eg_res) = (0i16, 0i16);
    let side = if white { WHITE } else { BLACK };

    let my_pawn_attacks = attack_container.pawn_attacks[side];

    let supported_knights = g.pieces[KNIGHT][side] & my_pawn_attacks;
    let supported_knights_amount = supported_knights.count_ones() as i16;
    mg_res += KNIGHT_SUPPORTED_BY_PAWN_MG * supported_knights_amount;
    eg_res += KNIGHT_SUPPORTED_BY_PAWN_EG * supported_knights_amount;
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.knight_supported[side] = supported_knights_amount as i8;
    }

    let (mut outpost_mg, mut outpost_eg, mut _outposts) = (0i16, 0i16, 0);
    let mut supp = supported_knights;
    while supp != 0u64 {
        let mut idx = supp.trailing_zeros() as usize;
        supp &= !bitboards::FILES[idx % 8];
        let mut front_span = if white {
            bitboards::w_front_span(1u64 << idx)
        } else {
            bitboards::b_front_span(1u64 << idx)
        };
        front_span = bitboards::west_one(front_span) | bitboards::east_one(front_span);
        if g.pieces[PAWN][1 - side] & front_span == 0u64 {
            if !white {
                idx = BLACK_INDEX[idx];
            }
            _outposts += 1;
            outpost_mg += KNIGHT_OUTPOST_MG_TABLE[idx / 8][idx % 8];
            outpost_eg += KNIGHT_OUTPOST_EG_TABLE[idx / 8][idx % 8];
            #[cfg(feature = "texel-tuning")]
            {
                _eval.trace.knight_outpost_table[side][idx / 8][idx % 8] += 1;
            }
        }
    }
    mg_res += outpost_mg;
    eg_res += outpost_eg;
    #[cfg(feature = "display-eval")]
    {
        log(&format!(
            "\nKnights for {}:\n",
            if white { "White" } else { "Black" }
        ));
        log(&format!(
            "\tSupported by pawns: {} -> ({} , {})\n",
            supported_knights_amount,
            KNIGHT_SUPPORTED_BY_PAWN_MG * supported_knights_amount,
            KNIGHT_SUPPORTED_BY_PAWN_EG * supported_knights_amount
        ));
        log(&format!(
            "\tOutposts: {} -> ({} , {})\n",
            _outposts, outpost_mg, outpost_eg
        ));
        log(&format!("Sum: ({} , {})\n", mg_res, eg_res));
    }

    (mg_res, eg_res)
}

pub fn piecewise(
    white: bool,
    g: &GameState,
    _eval: &mut EvaluationResult,
    attack_container: &GameStateAttackContainer,
) -> (i16, i16) {
    let side = if white { WHITE } else { BLACK };

    let defended_by_minors = attack_container.attacks_minor_sum[1 - side];
    let defended_squares = attack_container.attacks_sum[1 - side];
    let my_pieces = g.get_pieces_from_side(side);

    let enemy_king_idx = g.king_square(1 - side);
    let enemy_king_attackable = if white {
        bitboards::KING_ZONE_BLACK[enemy_king_idx]
    } else {
        bitboards::KING_ZONE_WHITE[enemy_king_idx]
    } & !defended_by_minors;

    let knight_checks = knight_attack(enemy_king_idx);
    let all_pieces = g.get_all_pieces();
    let bishop_checks = bishop_attack(enemy_king_idx, all_pieces);
    let rook_checks = rook_attack(enemy_king_idx, all_pieces);
    //Knights
    let mut knight_attackers: i16 = 0;
    let (mut knight_attacker_values_mg, mut knight_attacker_values_eg): (i16, i16) = (0, 0);
    let mut knights = g.pieces[KNIGHT][side];
    let (mut mk_mg, mut mk_eg) = (0i16, 0i16);
    let mut index = 0;
    while knights != 0u64 {
        let idx = knights.trailing_zeros() as usize;
        let targets = attack_container.attack[MGSA_KNIGHT][side][index] & !my_pieces;
        let mobility = targets.count_ones() as usize;
        mk_mg += KNIGHT_MOBILITY_BONUS_MG[mobility];
        mk_eg += KNIGHT_MOBILITY_BONUS_EG[mobility];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.knight_mobility[side][mobility] += 1;
        }

        let has_safe_check = (targets & knight_checks & !defended_squares) != 0u64;
        let enemy_king_attacks = targets & enemy_king_attackable;
        if has_safe_check || enemy_king_attacks != 0u64 {
            knight_attackers += 1;
        }
        knight_attacker_values_mg +=
            KNIGHT_ATTACK_WORTH_MG * enemy_king_attacks.count_ones() as i16;
        knight_attacker_values_eg +=
            KNIGHT_ATTACK_WORTH_EG * enemy_king_attacks.count_ones() as i16;
        knight_attacker_values_mg += if has_safe_check {
            KNIGHT_SAFE_CHECK_MG
        } else {
            0
        };
        knight_attacker_values_eg += if has_safe_check {
            KNIGHT_SAFE_CHECK_EG
        } else {
            0
        };
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.knight_attacked_sq[side] += enemy_king_attacks.count_ones() as u8;
            if has_safe_check {
                _eval.trace.knight_safe_check[side] += 1;
            }
        }
        knights ^= 1u64 << idx;
        index += 1;
    }
    //Bishops
    let mut bishop_attackers: i16 = 0;
    let (mut bishop_attacker_values_mg, mut bishop_attacker_values_eg): (i16, i16) = (0, 0);
    let mut bishops = g.pieces[BISHOP][side];
    let (mut mb_mg, mut mb_eg, mut mb_diag_mg, mut mb_diag_eg) = (0i16, 0i16, 0i16, 0i16);
    let mut index = 0;
    while bishops != 0u64 {
        let idx = bishops.trailing_zeros() as usize;
        let diagonally_adjacent_pawns =
            (bitboards::DIAGONALLY_ADJACENT[idx] & g.pieces[PAWN][side]).count_ones() as usize;
        mb_diag_mg += DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_MG[diagonally_adjacent_pawns];
        mb_diag_eg += DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS_EG[diagonally_adjacent_pawns];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.diagonally_adjacent_squares_withpawns[side][diagonally_adjacent_pawns] += 1;
        }
        let targets = attack_container.attack[MGSA_BISHOP][side][index] & !my_pieces;
        let mobility = targets.count_ones() as usize;
        mb_mg += BISHOP_MOBILITY_BONUS_MG[mobility];
        mb_eg += BISHOP_MOBILITY_BONUS_EG[mobility];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.bishop_mobility[side][mobility] += 1;
        }
        let has_safe_check = (targets & bishop_checks & !defended_squares) != 0u64;
        let enemy_king_attacks = targets & enemy_king_attackable;
        if has_safe_check || enemy_king_attacks != 0u64 {
            bishop_attackers += 1;
        }
        bishop_attacker_values_mg +=
            BISHOP_ATTACK_WORTH_MG * enemy_king_attacks.count_ones() as i16;
        bishop_attacker_values_eg +=
            BISHOP_ATTACK_WORTH_EG * enemy_king_attacks.count_ones() as i16;
        bishop_attacker_values_mg += if has_safe_check {
            BISHOP_SAFE_CHECK_MG
        } else {
            0
        };
        bishop_attacker_values_eg += if has_safe_check {
            BISHOP_SAFE_CHECK_EG
        } else {
            0
        };
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.bishop_attacked_sq[side] += enemy_king_attacks.count_ones() as u8;
            if has_safe_check {
                _eval.trace.bishop_safe_check[side] += 1;
            }
        }
        bishops ^= 1u64 << idx;
        index += 1;
    }

    //Rooks
    let mut rook_attackers: i16 = 0;
    let (mut rook_attacker_values_mg, mut rook_attacker_values_eg): (i16, i16) = (0, 0);
    let mut rooks = g.pieces[ROOK][side];
    let (mut mr_mg, mut mr_eg, mut rooks_onopen, mut rooks_onseventh) = (0i16, 0i16, 0i16, 0i16);
    let mut index = 0;
    while rooks != 0u64 {
        let idx = rooks.trailing_zeros() as usize;
        if if white { idx / 8 == 6 } else { idx / 8 == 1 } {
            rooks_onseventh += 1;
        }
        if bitboards::FILES[idx % 8] & (g.pieces[PAWN][side] | g.pieces[PAWN][1 - side]) == 0u64 {
            rooks_onopen += 1;
        }
        let targets = attack_container.attack[MGSA_ROOKS][side][index] & !my_pieces;
        let mobility = targets.count_ones() as usize;
        mr_mg += ROOK_MOBILITY_BONUS_MG[mobility];
        mr_eg += ROOK_MOBILITY_BONUS_EG[mobility];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.rook_mobility[side][mobility] += 1;
        }
        let has_safe_check = (targets & rook_checks & !defended_squares) != 0u64;
        let enemy_king_attacks = targets & enemy_king_attackable;
        if has_safe_check || enemy_king_attacks != 0u64 {
            rook_attackers += 1;
        }
        rook_attacker_values_mg += ROOK_ATTACK_WORTH_MG * enemy_king_attacks.count_ones() as i16;
        rook_attacker_values_eg += ROOK_ATTACK_WORTH_EG * enemy_king_attacks.count_ones() as i16;
        rook_attacker_values_mg += if has_safe_check {
            ROOK_SAFE_CHECK_MG
        } else {
            0
        };
        rook_attacker_values_eg += if has_safe_check {
            ROOK_SAFE_CHECK_EG
        } else {
            0
        };
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.rook_attacked_sq[side] += enemy_king_attacks.count_ones() as u8;
            if has_safe_check {
                _eval.trace.rook_safe_check[side] += 1;
            }
        }
        rooks ^= 1u64 << idx;
        index += 1;
    }
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.rook_on_open[side] = rooks_onopen as i8;
        _eval.trace.rook_on_seventh[side] = rooks_onseventh as i8;
    }
    //Queens
    let mut queen_attackers: i16 = 0;
    let (mut queen_attacker_values_mg, mut queen_attacker_values_eg): (i16, i16) = (0, 0);
    let mut queens = g.pieces[QUEEN][side];
    let (mut mq_mg, mut mq_eg) = (0i16, 0i16);
    let mut index = 0;
    while queens != 0u64 {
        let idx = queens.trailing_zeros() as usize;
        let targets = attack_container.attack[MGSA_QUEEN][side][index] & !my_pieces;
        let mobility = targets.count_ones() as usize;
        mq_mg += QUEEN_MOBILITY_BONUS_MG[mobility];
        mq_eg += QUEEN_MOBILITY_BONUS_EG[mobility];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.queen_mobility[side][mobility] += 1;
        }
        let has_safe_check = (targets & (bishop_checks | rook_checks) & !defended_squares) != 0u64;
        let enemy_king_attacks = targets & enemy_king_attackable;
        if has_safe_check || enemy_king_attacks != 0u64 {
            queen_attackers += 1;
        }
        queen_attacker_values_mg += QUEEN_ATTACK_WORTH_MG * enemy_king_attacks.count_ones() as i16;
        queen_attacker_values_eg += QUEEN_ATTACK_WORTH_EG * enemy_king_attacks.count_ones() as i16;
        queen_attacker_values_mg += if has_safe_check {
            QUEEN_SAFE_CHECK_MG
        } else {
            0
        };
        queen_attacker_values_eg += if has_safe_check {
            QUEEN_SAFE_CHECK_EG
        } else {
            0
        };
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.queen_attacked_sq[side] += enemy_king_attacks.count_ones() as u8;
            if has_safe_check {
                _eval.trace.queen_safe_check[side] += 1;
            }
        }
        queens ^= 1u64 << idx;
        index += 1;
    }
    let attack_mg = ((SAFETY_TABLE_MG[(knight_attacker_values_mg
        + bishop_attacker_values_mg
        + rook_attacker_values_mg
        + queen_attacker_values_mg)
        .min(99) as usize] as isize
        * ATTACK_WEIGHT_MG[(knight_attackers + bishop_attackers + rook_attackers + queen_attackers)
            .min(7) as usize] as isize) as f64
        / 100.0) as i16;
    let attack_eg = ((SAFETY_TABLE_EG[(knight_attacker_values_eg
        + bishop_attacker_values_eg
        + rook_attacker_values_eg
        + queen_attacker_values_eg)
        .min(99) as usize] as isize
        * ATTACK_WEIGHT_EG[(knight_attackers + bishop_attackers + rook_attackers + queen_attackers)
            .min(7) as usize] as isize) as f64
        / 100.0) as i16;
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.attackers[side] =
            (knight_attackers + bishop_attackers + rook_attackers + queen_attackers).min(7) as u8;
    }
    let mg_res = mk_mg
        + mb_mg
        + mr_mg
        + mq_mg
        + mb_diag_mg
        + rooks_onopen * ROOK_ON_OPEN_FILE_BONUS_MG
        + rooks_onseventh * ROOK_ON_SEVENTH_MG
        + attack_mg;
    let eg_res = mk_eg
        + mb_eg
        + mr_eg
        + mq_eg
        + mb_diag_eg
        + rooks_onopen * ROOK_ON_OPEN_FILE_BONUS_EG
        + rooks_onseventh * ROOK_ON_SEVENTH_EG
        + attack_eg;
    #[cfg(feature = "display-eval")]
    {
        log(&format!(
            "\nPiecewise for {}:\n",
            if white { "White" } else { "Black" }
        ));
        log(&format!("\tMobility Knight: ({} , {})\n", mk_mg, mk_eg));
        log(&format!("\tMobility Bishop: ({} , {})\n", mb_mg, mb_eg));
        log(&format!(
            "\tBishop Diagonally Adj: ({} , {})\n",
            mb_diag_mg, mb_diag_eg
        ));
        log(&format!("\tMobility Rook  : ({} , {})\n", mr_mg, mr_eg));
        log(&format!("\tMobility Queen : ({} , {})\n", mq_mg, mq_eg));
        log(&format!(
            "\tRooks on open  : {} -> ({} , {})\n",
            rooks_onopen,
            rooks_onopen * ROOK_ON_OPEN_FILE_BONUS_MG,
            rooks_onopen * ROOK_ON_OPEN_FILE_BONUS_EG
        ));
        log(&format!(
            "\tRooks on seventh: {} -> ({} , {})\n",
            rooks_onseventh,
            rooks_onseventh * ROOK_ON_SEVENTH_MG,
            rooks_onseventh * ROOK_ON_SEVENTH_EG
        ));
        log(&format!(
            "\tKnight Attackers/Value: ({} , MG: {}, EG: {})\n",
            knight_attackers, knight_attacker_values_mg, knight_attacker_values_eg
        ));
        log(&format!(
            "\tBishop Attackers/Value: ({} , MG: {}, EG: {})\n",
            bishop_attackers, bishop_attacker_values_mg, bishop_attacker_values_eg
        ));
        log(&format!(
            "\tRook Attackers/Value: ({} , MG: {}, EG: {})\n",
            rook_attackers, rook_attacker_values_mg, rook_attacker_values_eg
        ));
        log(&format!(
            "\tQueen Attackers/Value: ({} , MG: {}, EG: {})\n",
            queen_attackers, queen_attacker_values_mg, queen_attacker_values_eg
        ));
        log(&format!(
            "\tSum Attackers/Value: ({} , MG: {}, EG: {})\n",
            knight_attackers + bishop_attackers + rook_attackers + queen_attackers,
            knight_attacker_values_mg
                + bishop_attacker_values_mg
                + rook_attacker_values_mg
                + queen_attacker_values_mg,
            knight_attacker_values_eg
                + bishop_attacker_values_eg
                + rook_attacker_values_eg
                + queen_attacker_values_eg
        ));
        log(&format!(
            "\tAttack MG value: {} * {} / 100.0 -> {}\n",
            SAFETY_TABLE_MG[(knight_attacker_values_mg
                + bishop_attacker_values_mg
                + rook_attacker_values_mg
                + queen_attacker_values_mg)
                .min(99) as usize],
            ATTACK_WEIGHT_MG[(knight_attackers
                + bishop_attackers
                + rook_attackers
                + queen_attackers)
                .min(7) as usize],
            attack_mg
        ));
        log(&format!(
            "\tAttack EG value: {} * {} / 100.0 -> {}\n",
            SAFETY_TABLE_EG[(knight_attacker_values_eg
                + bishop_attacker_values_eg
                + rook_attacker_values_eg
                + queen_attacker_values_eg)
                .min(99) as usize],
            ATTACK_WEIGHT_EG[(knight_attackers
                + bishop_attackers
                + rook_attackers
                + queen_attackers)
                .min(7) as usize],
            attack_eg
        ));
        log(&format!("Sum: ({} , {})\n", mg_res, eg_res));
    }
    (mg_res, eg_res)
}

pub fn king(white: bool, g: &GameState, _eval: &mut EvaluationResult) -> (i16, i16) {
    let side = if white { WHITE } else { BLACK };
    let mut pawn_shield = if white {
        bitboards::SHIELDING_PAWNS_WHITE[g.pieces[KING][side].trailing_zeros() as usize]
    } else {
        bitboards::SHIELDING_PAWNS_BLACK[g.pieces[KING][side].trailing_zeros() as usize]
    };
    let mut king_front_span = if white {
        bitboards::w_front_span(g.pieces[KING][side])
    } else {
        bitboards::b_front_span(g.pieces[KING][side])
    };
    king_front_span |= bitboards::west_one(king_front_span) | bitboards::east_one(king_front_span);
    let file = g.king_square(side) % 8;
    if file == 7 {
        king_front_span |= bitboards::west_one(king_front_span);
    } else if file == 0 {
        king_front_span |= bitboards::east_one(king_front_span);
    }
    let mut shields_missing = 0;
    let mut shields_on_open_missing = 0;
    if g.full_moves >= 1 {
        while pawn_shield != 0u64 {
            let idx = pawn_shield.trailing_zeros() as usize;
            if g.pieces[PAWN][side] & pawn_shield & bitboards::FILES[idx % 8] == 0u64 {
                shields_missing += 1;
                if g.pieces[PAWN][1 - side] & bitboards::FILES[idx % 8] & king_front_span == 0u64 {
                    shields_on_open_missing += 1;
                }
            }
            pawn_shield &= !bitboards::FILES[idx % 8];
        }
    }
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.shielding_pawn_missing[side][shields_missing] += 1;
        _eval.trace.shielding_pawn_onopen_missing[side][shields_on_open_missing] += 1;
    }
    let (mg_res, eg_res) = (
        SHIELDING_PAWN_MISSING_MG[shields_missing]
            + SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG[shields_on_open_missing],
        SHIELDING_PAWN_MISSING_EG[shields_missing]
            + SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG[shields_on_open_missing],
    );
    #[cfg(feature = "display-eval")]
    {
        log(&format!(
            "\nKing for {}:\n",
            if white { "White" } else { "Black" }
        ));
        log(&format!(
            "\tShield pawn missing: {} -> ({} , {})\n",
            shields_missing,
            SHIELDING_PAWN_MISSING_MG[shields_missing],
            SHIELDING_PAWN_MISSING_EG[shields_missing]
        ));
        log(&format!(
            "\tShield pawn on open file missing: {} -> ({} , {})\n",
            shields_on_open_missing,
            SHIELDING_PAWN_MISSING_ON_OPEN_FILE_MG[shields_on_open_missing],
            SHIELDING_PAWN_MISSING_ON_OPEN_FILE_EG[shields_on_open_missing]
        ));
        log(&format!("Sum: ({} , {})\n", mg_res, eg_res));
    }
    (mg_res, eg_res)
}

pub fn pawns(
    white: bool,
    g: &GameState,
    _eval: &mut EvaluationResult,
    attack_container: &GameStateAttackContainer,
) -> (i16, i16) {
    let (mut mg_res, mut eg_res) = (0i16, 0i16);
    let side = if white { WHITE } else { BLACK };
    let empty = !g.get_all_pieces();
    //Bitboards
    let pawn_file_fill = bitboards::file_fill(g.pieces[PAWN][side]);
    let front_span = if white {
        bitboards::w_front_span(g.pieces[PAWN][side])
    } else {
        bitboards::b_front_span(g.pieces[PAWN][side])
    };
    let mut enemy_front_spans = if white {
        bitboards::b_front_span(g.pieces[PAWN][1 - side])
    } else {
        bitboards::w_front_span(g.pieces[PAWN][1 - side])
    };
    enemy_front_spans |=
        bitboards::west_one(enemy_front_spans) | bitboards::east_one(enemy_front_spans);
    let (my_west_attacks, my_east_attacks, enemy_pawn_attacks) = (
        attack_container.pawn_west_attacks[side],
        attack_container.pawn_east_attacks[side],
        attack_container.pawn_attacks[1 - side],
    );
    let my_pawn_attacks = my_west_attacks | my_east_attacks;
    let (my_pawn_pushes, my_pawn_double_pushes) = if white {
        (
            movegen::w_single_push_pawn_targets(g.pieces[PAWN][side], empty),
            movegen::w_double_push_pawn_targets(g.pieces[PAWN][side], empty),
        )
    } else {
        (
            movegen::b_single_push_pawn_targets(g.pieces[PAWN][side], empty),
            movegen::b_double_push_pawn_targets(g.pieces[PAWN][side], empty),
        )
    };
    let is_attackable = bitboards::west_one(front_span) | bitboards::east_one(front_span);
    let enemy_pieces = g.get_pieces_from_side(1 - side);

    let doubled_pawns = (g.pieces[PAWN][side] & front_span).count_ones() as i16;
    let isolated_pawns = (g.pieces[PAWN][side]
        & !bitboards::west_one(pawn_file_fill)
        & !bitboards::east_one(pawn_file_fill))
    .count_ones() as i16;
    let backward_pawns = (if white {
        g.pieces[PAWN][side] << 8
    } else {
        g.pieces[PAWN][side] >> 8
    } & enemy_pawn_attacks
        & !is_attackable)
        .count_ones() as i16;
    let mut supported_pawns = g.pieces[PAWN][side] & my_pawn_attacks;
    let _supported_amt = supported_pawns.count_ones() as usize;
    let mut mg_supp = 0;
    let mut eg_supp = 0;
    while supported_pawns != 0u64 {
        let mut index = supported_pawns.trailing_zeros() as usize;
        supported_pawns ^= 1u64 << index;
        if !white {
            index = BLACK_INDEX[index];
        }
        mg_supp += PAWN_SUPPORTED_VALUE_MG[index / 8][index % 8];
        eg_supp += PAWN_SUPPORTED_VALUE_EG[index / 8][index % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.pawn_supported[side][index / 8][index % 8] += 1;
        }
    }
    mg_res += mg_supp;
    eg_res += eg_supp;
    let center_attack_pawns = (g.pieces[PAWN][side]
        & if white {
            bitboards::south_east_one(*bitboards::INNER_CENTER)
                | bitboards::south_west_one(*bitboards::INNER_CENTER)
        } else {
            bitboards::north_east_one(*bitboards::INNER_CENTER)
                | bitboards::north_west_one(*bitboards::INNER_CENTER)
        })
    .count_ones() as i16;
    let pawn_mobility = (my_west_attacks.count_ones()
        + my_east_attacks.count_ones()
        + my_pawn_pushes.count_ones()
        + my_pawn_double_pushes.count_ones()) as i16;
    mg_res += doubled_pawns * PAWN_DOUBLED_VALUE_MG
        + isolated_pawns * PAWN_ISOLATED_VALUE_MG
        + backward_pawns * PAWN_BACKWARD_VALUE_MG
        + center_attack_pawns * PAWN_ATTACK_CENTER_MG
        + pawn_mobility * PAWN_MOBILITY_MG;
    eg_res += doubled_pawns * PAWN_DOUBLED_VALUE_EG
        + isolated_pawns * PAWN_ISOLATED_VALUE_EG
        + backward_pawns * PAWN_BACKWARD_VALUE_EG
        + center_attack_pawns * PAWN_ATTACK_CENTER_EG
        + pawn_mobility * PAWN_MOBILITY_EG;
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.pawn_doubled[side] = doubled_pawns as i8;
        _eval.trace.pawn_isolated[side] = isolated_pawns as i8;
        _eval.trace.pawn_backward[side] = backward_pawns as i8;
        _eval.trace.pawn_attack_center[side] = center_attack_pawns as i8;
        _eval.trace.pawn_mobility[side] = pawn_mobility as i8;
    }
    //Passers
    let mut passed_pawns: u64 = g.pieces[PAWN][side]

        /*& !if white {
            bitboards::w_rear_span(g.pieces[PAWN][side])
        } else {
            bitboards::b_rear_span(g.pieces[PAWN][side])
        }*/
        & !enemy_front_spans;
    let (mut passer_mg, mut passer_eg, mut _passer_normal, mut _passer_notblocked) =
        (0i16, 0i16, 0, 0);
    let mut weak_passers = 0;
    let behind_passers = if white {
        bitboards::b_front_span(passed_pawns)
    } else {
        bitboards::w_front_span(passed_pawns)
    };
    let rooks_support_passer =
        (behind_passers & (g.pieces[ROOK][side] | g.pieces[QUEEN][side])).count_ones() as i16;
    let enemy_rooks_attack_passer = (behind_passers
        & (g.pieces[ROOK][1 - side] | g.pieces[QUEEN][1 - side]))
        .count_ones() as i16;
    mg_res += rooks_support_passer * ROOK_BEHIND_SUPPORT_PASSER_MG
        + enemy_rooks_attack_passer * ROOK_BEHIND_ENEMY_PASSER_MG;
    eg_res += rooks_support_passer * ROOK_BEHIND_SUPPORT_PASSER_EG
        + enemy_rooks_attack_passer * ROOK_BEHIND_ENEMY_PASSER_EG;
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.rook_behind_support_passer[side] = rooks_support_passer as i8;
        _eval.trace.rook_behind_enemy_passer[side] = enemy_rooks_attack_passer as i8;
    }
    while passed_pawns != 0u64 {
        let idx = passed_pawns.trailing_zeros() as usize;
        //Passed and blocked
        _passer_normal += 1;
        passer_mg += PAWN_PASSED_VALUES_MG[if white { idx / 8 } else { 7 - idx / 8 }];
        passer_eg += PAWN_PASSED_VALUES_EG[if white { idx / 8 } else { 7 - idx / 8 }];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.pawn_passed[side][if white { idx / 8 } else { 7 - idx / 8 }] += 1;
        }
        //A weak passer is an attacked and not defended passer
        let weak_passer = (1u64 << idx) & attack_container.attacks_sum[1 - side] != 0u64
            && (1u64 << idx) & attack_container.attacks_sum[side] == 0u64;
        if weak_passer {
            //Weak passer
            weak_passers += 1;
        }
        //An unblocked passer is a) not weak b) all the squares until conversions are either not attacked or defended and unoccupied or attacked
        if !weak_passer
            && if white {
                bitboards::w_front_span(1u64 << idx)
            } else {
                bitboards::b_front_span(1u64 << idx)
            } & (attack_container.attacks_sum[1 - side] | enemy_pieces)
                & !attack_container.attacks_sum[side]
                == 0u64
        {
            //Passed and not blocked
            _passer_notblocked += 1;
            passer_mg +=
                PAWN_PASSED_NOT_BLOCKED_VALUES_MG[if white { idx / 8 } else { 7 - idx / 8 }];
            passer_eg +=
                PAWN_PASSED_NOT_BLOCKED_VALUES_EG[if white { idx / 8 } else { 7 - idx / 8 }];
            #[cfg(feature = "texel-tuning")]
            {
                _eval.trace.pawn_passed_notblocked[side]
                    [if white { idx / 8 } else { 7 - idx / 8 }] += 1;
            }
        }

        passed_pawns ^= 1u64 << idx;
    }
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.pawn_passed_weak[side] = weak_passers as i8;
    }
    mg_res += passer_mg + weak_passers * PAWN_PASSED_WEAK_MG;
    eg_res += passer_eg + weak_passers * PAWN_PASSED_WEAK_EG;
    #[cfg(feature = "display-eval")]
    {
        log(&format!(
            "\nPawns for {}:\n",
            if white { "White" } else { "Black" }
        ));
        log(&format!(
            "\tDoubled: {} -> ({} , {})\n",
            doubled_pawns,
            PAWN_DOUBLED_VALUE_MG * doubled_pawns,
            PAWN_DOUBLED_VALUE_EG * doubled_pawns
        ));
        log(&format!(
            "\tIsolated: {} -> ({} , {})\n",
            isolated_pawns,
            PAWN_ISOLATED_VALUE_MG * isolated_pawns,
            PAWN_ISOLATED_VALUE_EG * isolated_pawns
        ));
        log(&format!(
            "\tBackward: {} -> ({} , {})\n",
            backward_pawns,
            PAWN_BACKWARD_VALUE_MG * backward_pawns,
            PAWN_BACKWARD_VALUE_EG * backward_pawns
        ));
        log(&format!(
            "\tSupported: {} -> ({} , {})\n",
            _supported_amt, mg_supp, eg_supp
        ));
        log(&format!(
            "\tAttack Center: {} -> ({} , {})\n",
            center_attack_pawns,
            PAWN_ATTACK_CENTER_MG * center_attack_pawns,
            PAWN_ATTACK_CENTER_EG * center_attack_pawns
        ));
        log(&format!(
            "\tMobility: {} -> ({} , {})\n",
            pawn_mobility,
            PAWN_MOBILITY_MG * pawn_mobility,
            PAWN_MOBILITY_EG * pawn_mobility
        ));
        log(&format!(
            "\tPasser Blocked/Not Blocked: {} , {} -> MG/EG({} , {})\n",
            _passer_normal, _passer_notblocked, passer_mg, passer_eg
        ));
        log(&format!(
            "\tRook behind passer: {} -> ({} , {})\n",
            rooks_support_passer,
            ROOK_BEHIND_SUPPORT_PASSER_MG * rooks_support_passer,
            ROOK_BEHIND_SUPPORT_PASSER_EG * rooks_support_passer
        ));
        log(&format!(
            "\tEnemy Rook behind passer: {} -> ({} , {})\n",
            enemy_rooks_attack_passer,
            ROOK_BEHIND_ENEMY_PASSER_MG * enemy_rooks_attack_passer,
            ROOK_BEHIND_ENEMY_PASSER_EG * enemy_rooks_attack_passer
        ));
        log(&format!(
            "\tWeak passer: {} -> ({} , {})\n",
            weak_passers,
            PAWN_PASSED_WEAK_MG * weak_passers,
            PAWN_PASSED_WEAK_EG * weak_passers
        ));
        log(&format!("Sum: ({} , {})\n", mg_res, eg_res));
    }
    (mg_res, eg_res)
}

pub fn piece_values(white: bool, g: &GameState, _eval: &mut EvaluationResult) -> (i16, i16) {
    let (mut mg_res, mut eg_res) = (0i16, 0i16);
    let side = if white { WHITE } else { BLACK };

    let my_pawns = g.pieces[PAWN][side].count_ones() as i16;
    let mut my_knights = g.pieces[KNIGHT][side].count_ones() as i16;
    let mut my_bishops = g.pieces[BISHOP][side].count_ones() as i16;
    let my_rooks = g.pieces[ROOK][side].count_ones() as i16;
    let my_queens = g.pieces[QUEEN][side].count_ones() as i16;
    if my_pawns + my_knights + my_bishops + my_rooks + my_queens == 1 {
        my_knights = 0;
        my_bishops = 0;
    }
    mg_res += PAWN_PIECE_VALUE_MG * my_pawns;
    eg_res += PAWN_PIECE_VALUE_EG * my_pawns;

    let pawns_on_board = (g.pieces[PAWN][WHITE] | g.pieces[PAWN][BLACK]).count_ones() as usize;

    mg_res += (KNIGHT_PIECE_VALUE_MG + KNIGHT_VALUE_WITH_PAWNS[pawns_on_board]) * my_knights;
    eg_res += (KNIGHT_PIECE_VALUE_EG + KNIGHT_VALUE_WITH_PAWNS[pawns_on_board]) * my_knights;

    mg_res += BISHOP_PIECE_VALUE_MG * my_bishops;
    eg_res += BISHOP_PIECE_VALUE_EG * my_bishops;
    if my_bishops > 1 {
        mg_res += BISHOP_PAIR_BONUS_MG;
        eg_res += BISHOP_PAIR_BONUS_EG;
    }

    mg_res += ROOK_PIECE_VALUE_MG * my_rooks;
    eg_res += ROOK_PIECE_VALUE_EG * my_rooks;

    mg_res += QUEEN_PIECE_VALUE_MG * my_queens;
    eg_res += QUEEN_PIECE_VALUE_EG * my_queens;

    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.pawns[side] = my_pawns as i8;
        _eval.trace.knight_value_with_pawns = pawns_on_board as u8;
        _eval.trace.knights[side] = my_knights as i8;
        _eval.trace.bishops[side] = my_bishops as i8;
        if my_bishops > 1 {
            _eval.trace.bishop_bonus[side] = 1;
        }
        _eval.trace.rooks[side] = my_rooks as i8;
        _eval.trace.queens[side] = my_queens as i8;
    }
    #[cfg(feature = "display-eval")]
    {
        log(&format!(
            "\nPiece values for {}\n",
            if white { "White" } else { "Black" }
        ));
        log(&format!(
            "\tPawns: {} -> ({} , {})\n",
            my_pawns,
            PAWN_PIECE_VALUE_MG * my_pawns,
            PAWN_PIECE_VALUE_EG * my_pawns
        ));
        log(&format!(
            "\tKnights: {} -> ({} , {})\n",
            my_knights,
            (KNIGHT_PIECE_VALUE_MG + KNIGHT_VALUE_WITH_PAWNS[pawns_on_board]) * my_knights,
            (KNIGHT_PIECE_VALUE_EG + KNIGHT_VALUE_WITH_PAWNS[pawns_on_board]) * my_knights
        ));
        log(&format!(
            "\tBishops: {} -> ({} , {})\n",
            my_bishops,
            BISHOP_PIECE_VALUE_MG * my_bishops,
            BISHOP_PIECE_VALUE_EG * my_bishops
        ));
        if my_bishops > 1 {
            log(&format!(
                "\tBishop-Pair: {} -> ({} , {})\n",
                1, BISHOP_PAIR_BONUS_MG, BISHOP_PAIR_BONUS_EG
            ));
        }
        log(&format!(
            "\tRooks: {} -> ({} , {})\n",
            my_rooks,
            ROOK_PIECE_VALUE_MG * my_rooks,
            ROOK_PIECE_VALUE_EG * my_rooks
        ));
        log(&format!(
            "\tQueens: {} -> ({} , {})\n",
            my_queens,
            QUEEN_PIECE_VALUE_MG * my_queens,
            QUEEN_PIECE_VALUE_EG * my_queens
        ));
        log(&format!("Sum: ({} , {})\n", mg_res, eg_res));
    }
    (mg_res, eg_res)
}

pub fn calculate_phase(g: &GameState) -> f64 {
    let (w_queens, b_queens, w_knights, b_knights, w_bishops, b_bishops, w_rooks, b_rooks) = (
        g.pieces[QUEEN][WHITE],
        g.pieces[QUEEN][BLACK],
        g.pieces[KNIGHT][WHITE],
        g.pieces[KNIGHT][BLACK],
        g.pieces[BISHOP][WHITE],
        g.pieces[BISHOP][BLACK],
        g.pieces[ROOK][WHITE],
        g.pieces[ROOK][BLACK],
    );
    let mut npm = (w_queens | b_queens).count_ones() as i16 * 1500
        + (w_bishops | b_bishops).count_ones() as i16 * 510
        + (w_rooks | b_rooks).count_ones() as i16 * 650
        + (w_knights | b_knights).count_ones() as i16 * 500;
    if npm < EG_LIMIT {
        npm = EG_LIMIT;
    }
    if npm > MG_LIMIT {
        npm = MG_LIMIT;
    }
    f64::from(npm - EG_LIMIT) * 128.0 / f64::from(MG_LIMIT - EG_LIMIT)
}

pub fn piece_value(piece_type: PieceType, phase: f64) -> i16 {
    if let PieceType::Pawn = piece_type {
        ((f64::from(PAWN_PIECE_VALUE_MG) * phase
            + f64::from(PAWN_PIECE_VALUE_EG) * (128.0 - phase))
            / 128.0) as i16
    } else if let PieceType::Knight = piece_type {
        ((f64::from(KNIGHT_PIECE_VALUE_MG) * phase
            + f64::from(KNIGHT_PIECE_VALUE_EG) * (128.0 - phase))
            / 128.0) as i16
    } else if let PieceType::Bishop = piece_type {
        ((f64::from(BISHOP_PIECE_VALUE_MG) * phase
            + f64::from(BISHOP_PIECE_VALUE_EG) * (128.0 - phase))
            / 128.0) as i16
    } else if let PieceType::Rook = piece_type {
        ((f64::from(ROOK_PIECE_VALUE_MG) * phase
            + f64::from(ROOK_PIECE_VALUE_EG) * (128.0 - phase))
            / 128.0) as i16
    } else if let PieceType::Queen = piece_type {
        ((f64::from(QUEEN_PIECE_VALUE_MG) * phase
            + f64::from(QUEEN_PIECE_VALUE_EG) * (128.0 - phase))
            / 128.0) as i16
    } else {
        panic!("Invalid piece type!");
    }
}
