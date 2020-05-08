pub mod parameters;
pub mod params;
pub mod phase;
pub mod psqt_evaluation;
pub mod trace;

use crate::bitboards::bitboards;
use crate::bitboards::bitboards::constants::*;
use crate::board_representation::game_state::{GameState, PieceType, BLACK, WHITE};
use crate::board_representation::game_state_attack_container::{
    GameStateAttackContainer, MGSA_BISHOP, MGSA_KNIGHT, MGSA_QUEEN, MGSA_ROOKS,
};
#[cfg(feature = "texel-tuning")]
use crate::evaluation::trace::Trace;
use crate::move_generation::movegen;
use crate::move_generation::movegen::{bishop_attack, rook_attack};
use params::*;
use psqt_evaluation::psqt;
use psqt_evaluation::BLACK_INDEX;
use std::fmt::{Debug, Display, Formatter, Result};
use std::ops;

pub const MG: usize = 0;
pub const EG: usize = 1;

pub const FIRST_LAZY_MARGIN: i16 = 450;
pub const SECOND_LAZY_MARGIN: i16 = 250;
#[derive(Copy, Clone, PartialEq)]
pub struct EvaluationScore(pub i16, pub i16);
impl EvaluationScore {
    pub fn interpolate(self, phase: f64) -> i16 {
        ((f64::from(self.0) * phase + f64::from(self.1) * (128.0 - phase)) / 128.0) as i16
    }
}
impl Default for EvaluationScore {
    fn default() -> Self {
        EvaluationScore(0, 0)
    }
}
impl Display for EvaluationScore {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({} , {})", self.0, self.1)
    }
}
impl Debug for EvaluationScore {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "EvaluationScore({}, {})", self.0, self.1)
    }
}
impl ops::Add<EvaluationScore> for EvaluationScore {
    type Output = EvaluationScore;

    fn add(self, other: EvaluationScore) -> Self::Output {
        EvaluationScore(self.0 + other.0, self.1 + other.1)
    }
}
impl ops::Add<i16> for EvaluationScore {
    type Output = EvaluationScore;

    fn add(self, other: i16) -> Self::Output {
        EvaluationScore(self.0 + other, self.1 + other)
    }
}
impl ops::AddAssign<EvaluationScore> for EvaluationScore {
    fn add_assign(&mut self, other: EvaluationScore) {
        self.0 += other.0;
        self.1 += other.1;
    }
}
impl ops::Sub<EvaluationScore> for EvaluationScore {
    type Output = EvaluationScore;

    fn sub(self, other: EvaluationScore) -> Self::Output {
        EvaluationScore(self.0 - other.0, self.1 - other.1)
    }
}
impl ops::SubAssign<EvaluationScore> for EvaluationScore {
    fn sub_assign(&mut self, other: EvaluationScore) {
        self.0 -= other.0;
        self.1 -= other.1;
    }
}
impl ops::Mul<i16> for EvaluationScore {
    type Output = EvaluationScore;

    fn mul(self, other: i16) -> Self::Output {
        EvaluationScore(self.0 * other, self.1 * other)
    }
}
impl ops::MulAssign<i16> for EvaluationScore {
    fn mul_assign(&mut self, other: i16) {
        self.0 *= other;
        self.1 *= other;
    }
}

pub struct EvaluationResult {
    pub final_eval: i16,
    #[cfg(feature = "texel-tuning")]
    pub trace: Trace,
}

pub fn eval_game_state_from_null(g: &GameState) -> EvaluationResult {
    let mgsac = GameStateAttackContainer::from_state(g);
    eval_game_state(g, &mgsac, -16000, 16000)
}

pub fn eval_game_state(
    g: &GameState,
    attacks: &GameStateAttackContainer,
    _alpha: i16, //Lazy Eval components, unneeded currently
    _beta: i16,
) -> EvaluationResult {
    #[cfg(feature = "display-eval")]
    {
        println!("Evaluating GameState fen: {}", g.to_fen());
    }
    let mut result = EvaluationResult {
        final_eval: 0,
        #[cfg(feature = "texel-tuning")]
        trace: Trace::default(),
    };
    let phase = g.phase.phase;
    #[cfg(feature = "texel-tuning")]
    {
        result.trace.phase = phase;
    }
    let mut res = EvaluationScore::default();

    if g.color_to_move == WHITE {
        res += TEMPO_BONUS;
    } else {
        res -= TEMPO_BONUS;
    }
    #[cfg(feature = "display-eval")]
    {
        let tempo = if g.color_to_move == WHITE {
            TEMPO_BONUS
        } else {
            TEMPO_BONUS * -1
        };
        println!("\nTempo:{}", tempo);
    }
    #[cfg(feature = "texel-tuning")]
    {
        result.trace.tempo_bonus = if g.color_to_move == WHITE { 1 } else { -1 };
    }

    let psqt_score: EvaluationScore =
        if cfg!(feature = "display-eval") || cfg!(feature = "texel-tuning") {
            let (psqt_w, psqt_b) = (
                psqt(true, &g.pieces, &mut result),
                psqt(false, &g.pieces, &mut result),
            );
            psqt_w - psqt_b
        } else {
            g.psqt
        };
    #[cfg(feature = "display-eval")]
    {
        println!("\nPSQT Sum: {}", psqt_score);
    }
    res += psqt_score;

    let (pieces_w, pieces_b) = (
        piece_values(true, g, &mut result),
        piece_values(false, g, &mut result),
    );
    #[cfg(feature = "display-eval")]
    {
        println!(
            "\nPiece value Sum: {} - {} -> {}",
            pieces_w,
            pieces_b,
            pieces_w - pieces_b
        );
    }
    res += pieces_w - pieces_b;

    /*let lazy_eval = EvaluationScore(res.0, (f64::from(res.1) / 1.5) as i16);
    let lazy_eval = lazy_eval.interpolate(phase);

    if lazy_eval + FIRST_LAZY_MARGIN < alpha {
        result.final_eval = lazy_eval + FIRST_LAZY_MARGIN;
        return result;
    } else if lazy_eval - FIRST_LAZY_MARGIN > beta {
        result.final_eval = lazy_eval - FIRST_LAZY_MARGIN;
        return result;
    }*/
    let (pawns_w, pawns_b) = (
        pawns(true, g, &mut result, attacks),
        pawns(false, g, &mut result, attacks),
    );
    #[cfg(feature = "display-eval")]
    {
        println!(
            "\nPawn Sum: {} - {} -> {}",
            pawns_w,
            pawns_b,
            pawns_w - pawns_b
        );
    }
    res += pawns_w - pawns_b;

    /*let lazy_eval = EvaluationScore(res.0, (f64::from(res.1) / 1.5) as i16);
    let lazy_eval = lazy_eval.interpolate(phase);

    if lazy_eval + SECOND_LAZY_MARGIN < alpha {
        result.final_eval = lazy_eval + SECOND_LAZY_MARGIN;
        return result;
    } else if lazy_eval - SECOND_LAZY_MARGIN > beta {
        result.final_eval = lazy_eval - SECOND_LAZY_MARGIN;
        return result;
    }*/

    let (knights_w, knights_b) = (
        knights(true, g, &mut result, attacks),
        knights(false, g, &mut result, attacks),
    );
    #[cfg(feature = "display-eval")]
    {
        println!(
            "\nKnights Sum: {} - {} -> {}",
            knights_w,
            knights_b,
            knights_w - knights_b
        );
    }
    res += knights_w - knights_b;

    let (piecewise_w, piecewise_b) = (
        piecewise(true, g, &mut result, attacks),
        piecewise(false, g, &mut result, attacks),
    );
    #[cfg(feature = "display-eval")]
    {
        println!(
            "\nPiecewise Sum: {} - {} -> {}\n",
            piecewise_w,
            piecewise_b,
            piecewise_w - piecewise_b
        );
    }
    res += piecewise_w - piecewise_b;

    let (king_w, king_b) = (king(true, g, &mut result), king(false, g, &mut result));
    #[cfg(feature = "display-eval")]
    {
        println!("\nKing Sum: {} - {} -> {}", king_w, king_b, king_w - king_b);
    }
    res += king_w - king_b;

    res.1 = (f64::from(res.1) / 1.5) as i16;
    //Phasing is done the same way stockfish does it
    let final_res = res.interpolate(phase);
    #[cfg(feature = "display-eval")]
    {
        println!(
            "\nSum: {} + {} + {} + {} + {} + {} + {} -> {} (EG/=1.5)",
            psqt_score,
            knights_w - knights_b,
            piecewise_w - piecewise_b,
            king_w - king_b,
            pawns_w - pawns_b,
            pieces_w - pieces_b,
            if g.color_to_move == 0 {
                TEMPO_BONUS
            } else {
                TEMPO_BONUS * -1
            },
            res
        );
        println!("Phase: {}", phase);
        println!(
            "\nFinal Result: ({} * {} + {} * (128.0 - {}))/128.0 -> {}",
            res.0, phase, res.1, phase, final_res,
        );
    }
    result.final_eval = final_res;
    result
}

pub fn knights(
    white: bool,
    g: &GameState,
    _eval: &mut EvaluationResult,
    attack_container: &GameStateAttackContainer,
) -> EvaluationScore {
    let mut res = EvaluationScore::default();
    let side = if white { WHITE } else { BLACK };

    let my_pawn_attacks = attack_container.pawn_attacks[side];

    let supported_knights = g.pieces[PieceType::Knight as usize][side] & my_pawn_attacks;
    let supported_knights_amount = supported_knights.count_ones() as i16;
    res += KNIGHT_SUPPORTED_BY_PAWN * supported_knights_amount;
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.knight_supported +=
            supported_knights_amount as i8 * if side == WHITE { 1 } else { -1 };
    }
    let mut outpost = EvaluationScore::default();
    let mut _outposts = 0;
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
        if g.pieces[PieceType::Pawn as usize][1 - side] & front_span == 0u64 {
            if !white {
                idx = BLACK_INDEX[idx];
            }
            _outposts += 1;
            outpost += KNIGHT_OUTPOST_TABLE[idx / 8][idx % 8];
            #[cfg(feature = "texel-tuning")]
            {
                _eval.trace.knight_outpost_table[idx / 8][idx % 8] +=
                    if side == WHITE { 1 } else { -1 };
            }
        }
    }
    res += outpost;
    #[cfg(feature = "display-eval")]
    {
        println!("\nKnights for {}:", if white { "White" } else { "Black" });
        println!(
            "\tSupported by pawns: {} -> {}",
            supported_knights_amount,
            KNIGHT_SUPPORTED_BY_PAWN * supported_knights_amount,
        );
        println!("\tOutposts: {} -> {}", _outposts, outpost);
        println!("Sum: {}", res);
    }

    res
}

pub fn piecewise(
    white: bool,
    g: &GameState,
    _eval: &mut EvaluationResult,
    attack_container: &GameStateAttackContainer,
) -> EvaluationScore {
    let side = if white { WHITE } else { BLACK };

    let defended_by_minors = attack_container.attacks_minor_sum[1 - side];
    let defended_squares = attack_container.attacks_sum[1 - side];
    let my_pieces = g.get_pieces_from_side(side);

    let enemy_king_idx = g.king_square(1 - side);
    let enemy_king_attackable = if white {
        KING_ZONE_BLACK[enemy_king_idx]
    } else {
        KING_ZONE_WHITE[enemy_king_idx]
    } & !defended_by_minors;

    let knight_checks = KNIGHT_ATTACKS[enemy_king_idx];
    let all_pieces = g.get_all_pieces();
    let bishop_checks = bishop_attack(enemy_king_idx, all_pieces);
    let rook_checks = rook_attack(enemy_king_idx, all_pieces);
    //Knights
    let mut knight_attackers: i16 = 0;
    let mut knight_attacker_values = EvaluationScore::default();
    let mut mk = EvaluationScore::default();
    let mut knights = g.pieces[PieceType::Knight as usize][side];
    let mut index = 0;
    while knights != 0u64 {
        let idx = knights.trailing_zeros() as usize;
        let targets = attack_container.attack[MGSA_KNIGHT][side][index] & !my_pieces;

        let mobility = targets.count_ones() as usize;
        mk += KNIGHT_MOBILITY_BONUS[mobility];

        let has_safe_check = (targets & knight_checks & !defended_squares) != 0u64;
        let enemy_king_attacks = targets & enemy_king_attackable;
        if has_safe_check || enemy_king_attacks != 0u64 {
            knight_attackers += 1;
        }
        knight_attacker_values += KNIGHT_ATTACK_WORTH * enemy_king_attacks.count_ones() as i16;
        if has_safe_check {
            knight_attacker_values += KNIGHT_SAFE_CHECK;
        }
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.knight_mobility[mobility] += if side == WHITE { 1 } else { -1 };
            _eval.trace.knight_attacked_sq[side] += enemy_king_attacks.count_ones() as u8;
            if has_safe_check {
                _eval.trace.knight_safe_check[side] += 1;
            }
        }
        knights ^= square(idx);
        index += 1;
    }
    //Bishops
    let mut bishop_attackers: i16 = 0;
    let mut bishop_attacker_values = EvaluationScore::default();
    let mut bishop_xray_king: i16 = 0;
    let (mut mb, mut mb_diag) = (EvaluationScore::default(), EvaluationScore::default());
    let mut bishops = g.pieces[PieceType::Bishop as usize][side];
    let mut index = 0;
    while bishops != 0u64 {
        let idx = bishops.trailing_zeros() as usize;
        if (FREEFIELD_BISHOP_ATTACKS[idx] & g.pieces[PieceType::King as usize][1 - side]) != 0u64
            && (movegen::xray_bishop_attacks(
                attack_container.attack[MGSA_BISHOP][side][index],
                all_pieces,
                all_pieces,
                idx,
            ) & g.pieces[PieceType::King as usize][1 - side])
                != 0u64
        {
            bishop_xray_king += 1;
        }
        let diagonally_adjacent_pawns = (DIAGONALLY_ADJACENT[idx]
            & g.pieces[PieceType::Pawn as usize][side])
            .count_ones() as usize;
        mb_diag += DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS[diagonally_adjacent_pawns];

        let targets = attack_container.attack[MGSA_BISHOP][side][index] & !my_pieces;
        let mobility = targets.count_ones() as usize;
        mb += BISHOP_MOBILITY_BONUS[mobility];

        let has_safe_check = (targets & bishop_checks & !defended_squares) != 0u64;
        let enemy_king_attacks = targets & enemy_king_attackable;
        if has_safe_check || enemy_king_attacks != 0u64 {
            bishop_attackers += 1;
        }
        bishop_attacker_values += BISHOP_ATTACK_WORTH * enemy_king_attacks.count_ones() as i16;
        if has_safe_check {
            bishop_attacker_values += BISHOP_SAFE_CHECK;
        }
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.diagonally_adjacent_squares_withpawns[diagonally_adjacent_pawns] +=
                if side == WHITE { 1 } else { -1 };
            _eval.trace.bishop_mobility[mobility] += if side == WHITE { 1 } else { -1 };
            _eval.trace.bishop_attacked_sq[side] += enemy_king_attacks.count_ones() as u8;
            if has_safe_check {
                _eval.trace.bishop_safe_check[side] += 1;
            }
        }
        bishops ^= square(idx);
        index += 1;
    }
    //Rooks
    let mut rook_attackers: i16 = 0;
    let mut rook_attacker_values = EvaluationScore::default();
    let mut rook_xray_king: i16 = 0;
    let (mut mr, mut rooks_onopen, mut rooks_on_semi_open, mut rooks_onseventh) =
        (EvaluationScore::default(), 0i16, 0i16, 0i16);
    let mut rooks = g.pieces[PieceType::Rook as usize][side];
    let mut index = 0;
    while rooks != 0u64 {
        let idx = rooks.trailing_zeros() as usize;
        if (FREEFIELD_ROOK_ATTACKS[idx] & g.pieces[PieceType::King as usize][1 - side]) != 0u64
            && (movegen::xray_rook_attacks(
                attack_container.attack[MGSA_ROOKS][side][index],
                all_pieces,
                all_pieces,
                idx,
            ) & g.pieces[PieceType::King as usize][1 - side])
                != 0u64
        {
            rook_xray_king += 1;
        }
        if if white { idx / 8 == 6 } else { idx / 8 == 1 } {
            rooks_onseventh += 1;
        }
        if FILES[idx % 8]
            & (g.pieces[PieceType::Pawn as usize][side]
                | g.pieces[PieceType::Pawn as usize][1 - side])
            == 0u64
        {
            rooks_onopen += 1;
        } else if (FILES[idx % 8] & g.pieces[PieceType::Pawn as usize][1 - side]).count_ones() == 1
            && (FILES[idx % 8] & g.pieces[PieceType::Pawn as usize][side]) == 0u64
        {
            rooks_on_semi_open += 1;
        }

        let targets = attack_container.attack[MGSA_ROOKS][side][index] & !my_pieces;

        let mobility = targets.count_ones() as usize;
        mr += ROOK_MOBILITY_BONUS[mobility];

        let has_safe_check = (targets & rook_checks & !defended_squares) != 0u64;
        let enemy_king_attacks = targets & enemy_king_attackable;
        if has_safe_check || enemy_king_attacks != 0u64 {
            rook_attackers += 1;
        }
        rook_attacker_values += ROOK_ATTACK_WORTH * enemy_king_attacks.count_ones() as i16;
        if has_safe_check {
            rook_attacker_values += ROOK_SAFE_CHECK;
        }
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.rook_mobility[mobility] += if side == WHITE { 1 } else { -1 };
            _eval.trace.rook_attacked_sq[side] += enemy_king_attacks.count_ones() as u8;
            if has_safe_check {
                _eval.trace.rook_safe_check[side] += 1;
            }
        }
        rooks ^= square(idx);
        index += 1;
    }

    //Queens
    let mut queen_attackers: i16 = 0;
    let mut queen_attacker_values = EvaluationScore::default();
    let mut queen_xray_king: i16 = 0;
    let (mut queens_onopen, mut queens_on_semi_open) = (0i16, 0i16);
    let mut mq = EvaluationScore::default();
    let mut queens = g.pieces[PieceType::Queen as usize][side];
    let mut index = 0;
    while queens != 0u64 {
        let idx = queens.trailing_zeros() as usize;
        if (FREEFIELD_BISHOP_ATTACKS[idx] & g.pieces[PieceType::King as usize][1 - side]) != 0u64
            && (movegen::xray_bishop_attacks(
                attack_container.attack[MGSA_QUEEN][side][index] & FREEFIELD_BISHOP_ATTACKS[idx],
                all_pieces,
                all_pieces,
                idx,
            ) & g.pieces[PieceType::King as usize][1 - side])
                != 0u64
            || (FREEFIELD_ROOK_ATTACKS[idx] & g.pieces[PieceType::King as usize][1 - side]) != 0u64
                && (movegen::xray_rook_attacks(
                    attack_container.attack[MGSA_QUEEN][side][index] & FREEFIELD_ROOK_ATTACKS[idx],
                    all_pieces,
                    all_pieces,
                    idx,
                ) & g.pieces[PieceType::King as usize][1 - side])
                    != 0u64
        {
            queen_xray_king += 1;
        }

        if FILES[idx % 8]
            & (g.pieces[PieceType::Pawn as usize][side]
                | g.pieces[PieceType::Pawn as usize][1 - side])
            == 0u64
        {
            queens_onopen += 1;
        } else if (FILES[idx % 8] & g.pieces[PieceType::Pawn as usize][1 - side]).count_ones() == 1
            && (FILES[idx % 8] & g.pieces[PieceType::Pawn as usize][side]) == 0u64
        {
            queens_on_semi_open += 1;
        }

        let targets = attack_container.attack[MGSA_QUEEN][side][index] & !my_pieces;

        let mobility = targets.count_ones() as usize;
        mq += QUEEN_MOBILITY_BONUS[mobility];

        let has_safe_check = (targets & (bishop_checks | rook_checks) & !defended_squares) != 0u64;
        let enemy_king_attacks = targets & enemy_king_attackable;
        if has_safe_check || enemy_king_attacks != 0u64 {
            queen_attackers += 1;
        }
        queen_attacker_values += QUEEN_ATTACK_WORTH * enemy_king_attacks.count_ones() as i16;
        if has_safe_check {
            queen_attacker_values += QUEEN_SAFE_CHECK;
        }

        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.queen_mobility[mobility] += if side == WHITE { 1 } else { -1 };
            _eval.trace.queen_attacked_sq[side] += enemy_king_attacks.count_ones() as u8;
            if has_safe_check {
                _eval.trace.queen_safe_check[side] += 1;
            }
        }
        queens ^= square(idx);
        index += 1;
    }
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.rook_on_open += rooks_onopen as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.rook_on_semi_open +=
            rooks_on_semi_open as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.rook_on_seventh += rooks_onseventh as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.queen_on_open += queens_onopen as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.queen_on_semi_open +=
            queens_on_semi_open as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.bishop_xray_king += bishop_xray_king as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.rook_xray_king += rook_xray_king as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.queen_xray_king += queen_xray_king as i8 * if side == WHITE { 1 } else { -1 };
    }

    let attack_mg = ((SAFETY_TABLE[(knight_attacker_values.0
        + bishop_attacker_values.0
        + rook_attacker_values.0
        + queen_attacker_values.0)
        .min(99) as usize]
        .0 as isize
        * ATTACK_WEIGHT[(knight_attackers + bishop_attackers + rook_attackers + queen_attackers)
            .min(7) as usize]
            .0 as isize) as f64
        / 100.0) as i16;
    let attack_eg = ((SAFETY_TABLE[(knight_attacker_values.1
        + bishop_attacker_values.1
        + rook_attacker_values.1
        + queen_attacker_values.1)
        .min(99) as usize]
        .1 as isize
        * ATTACK_WEIGHT[(knight_attackers + bishop_attackers + rook_attackers + queen_attackers)
            .min(7) as usize]
            .1 as isize) as f64
        / 100.0) as i16;
    let attack = EvaluationScore(attack_mg, attack_eg);
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.attackers[side] =
            (knight_attackers + bishop_attackers + rook_attackers + queen_attackers).min(7) as u8;
    }
    #[allow(clippy::let_and_return)]
    let res = mk
        + mb
        + mr
        + mq
        + mb_diag
        + ROOK_ON_OPEN_FILE_BONUS * rooks_onopen
        + ROOK_ON_SEMI_OPEN_FILE_BONUS * rooks_on_semi_open
        + ROOK_ON_SEVENTH * rooks_onseventh
        + QUEEN_ON_OPEN_FILE_BONUS * queens_onopen
        + QUEEN_ON_SEMI_OPEN_FILE_BONUS * queens_on_semi_open
        + BISHOP_XRAY_KING * bishop_xray_king
        + ROOK_XRAY_KING * rook_xray_king
        + QUEEN_XRAY_KING * queen_xray_king
        + attack;

    #[cfg(feature = "display-eval")]
    {
        println!("\nPiecewise for {}:", if white { "White" } else { "Black" });
        println!("\tMobility Knight: {}", mk);
        println!("\tMobility Bishop: {}", mb);
        println!("\tBishop Diagonally Adj: {}", mb_diag);
        println!("\tMobility Rook  : {}", mr);
        println!("\tMobility Queen : {}", mq);
        println!(
            "\tBishopXrayKing : {} -> {}",
            bishop_xray_king,
            BISHOP_XRAY_KING * bishop_xray_king,
        );
        println!(
            "\tRookXrayKing : {} -> {}",
            rook_xray_king,
            ROOK_XRAY_KING * rook_xray_king,
        );
        println!(
            "\tQueenXrayKing : {} -> {}",
            queen_xray_king,
            QUEEN_XRAY_KING * queen_xray_king,
        );
        println!(
            "\tRooks on open  : {} -> {}",
            rooks_onopen,
            ROOK_ON_OPEN_FILE_BONUS * rooks_onopen,
        );
        println!(
            "\tRooks on semi-open  : {} -> {}",
            rooks_on_semi_open,
            ROOK_ON_SEMI_OPEN_FILE_BONUS * rooks_on_semi_open,
        );
        println!(
            "\tQueens on open  : {} -> {}",
            queens_onopen,
            QUEEN_ON_OPEN_FILE_BONUS * queens_onopen,
        );
        println!(
            "\tQueens on semi-open  : {} -> {}",
            queens_on_semi_open,
            QUEEN_ON_SEMI_OPEN_FILE_BONUS * queens_on_semi_open,
        );
        println!(
            "\tRooks on seventh: {} -> {}",
            rooks_onseventh,
            ROOK_ON_SEVENTH * rooks_onseventh
        );
        println!(
            "\tKnight Attackers: Num: {} , Val: {}",
            knight_attackers, knight_attacker_values
        );
        println!(
            "\tBishop Attackers: Num: {} , Val: {}",
            bishop_attackers, bishop_attacker_values
        );
        println!(
            "\tRook Attackers: Num: {} , Val: {}",
            rook_attackers, rook_attacker_values
        );
        println!(
            "\tQueen Attackers: Num: {} , Val: {}",
            queen_attackers, queen_attacker_values
        );
        println!(
            "\tSum Attackers: (Num: {} , Val: {}",
            knight_attackers + bishop_attackers + rook_attackers + queen_attackers,
            knight_attacker_values
                + bishop_attacker_values
                + rook_attacker_values
                + queen_attacker_values
        );
        println!(
            "\tAttack MG value: {} * {} / 100.0 -> {}",
            SAFETY_TABLE[(knight_attacker_values.0
                + bishop_attacker_values.0
                + rook_attacker_values.0
                + queen_attacker_values.0)
                .min(99) as usize]
                .0,
            ATTACK_WEIGHT[(knight_attackers + bishop_attackers + rook_attackers + queen_attackers)
                .min(7) as usize]
                .0,
            attack_mg
        );
        println!(
            "\tAttack EG value: {} * {} / 100.0 -> {}",
            SAFETY_TABLE[(knight_attacker_values.1
                + bishop_attacker_values.1
                + rook_attacker_values.1
                + queen_attacker_values.1)
                .min(99) as usize]
                .1,
            ATTACK_WEIGHT[(knight_attackers + bishop_attackers + rook_attackers + queen_attackers)
                .min(7) as usize]
                .1,
            attack_eg
        );
        println!("Sum: {}", res);
    }
    res
}

pub fn king(white: bool, g: &GameState, _eval: &mut EvaluationResult) -> EvaluationScore {
    let side = if white { WHITE } else { BLACK };
    let mut pawn_shield = if white {
        SHIELDING_PAWNS_WHITE[g.pieces[PieceType::King as usize][side].trailing_zeros() as usize]
    } else {
        SHIELDING_PAWNS_BLACK[g.pieces[PieceType::King as usize][side].trailing_zeros() as usize]
    };
    let mut king_front_span = if white {
        bitboards::w_front_span(g.pieces[PieceType::King as usize][side])
    } else {
        bitboards::b_front_span(g.pieces[PieceType::King as usize][side])
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
            if g.pieces[PieceType::Pawn as usize][side] & pawn_shield & FILES[idx % 8] == 0u64 {
                shields_missing += 1;
                if g.pieces[PieceType::Pawn as usize][1 - side] & FILES[idx % 8] & king_front_span
                    == 0u64
                {
                    shields_on_open_missing += 1;
                }
            }
            pawn_shield &= !FILES[idx % 8];
        }
    }
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.shielding_pawn_missing[shields_missing] += if side == WHITE { 1 } else { -1 };
        _eval.trace.shielding_pawn_onopen_missing[shields_on_open_missing] +=
            if side == WHITE { 1 } else { -1 };
    }
    #[allow(clippy::let_and_return)]
    let res = SHIELDING_PAWN_MISSING[shields_missing]
        + SHIELDING_PAWN_MISSING_ON_OPEN_FILE[shields_on_open_missing];

    #[cfg(feature = "display-eval")]
    {
        println!("\nKing for {}:", if white { "White" } else { "Black" });
        println!(
            "\tShield pawn missing: {} -> {}",
            shields_missing, SHIELDING_PAWN_MISSING[shields_missing],
        );
        println!(
            "\tShield pawn on open file missing: {} -> {}",
            shields_on_open_missing, SHIELDING_PAWN_MISSING_ON_OPEN_FILE[shields_on_open_missing],
        );
        println!("Sum: {}", res);
    }
    res
}

pub fn get_distance(sq: isize, sq2: isize) -> usize {
    (sq / 8 - sq2 / 8).abs().max((sq % 8 - sq2 % 8).abs()) as usize
}

pub fn pawns(
    white: bool,
    g: &GameState,
    _eval: &mut EvaluationResult,
    attack_container: &GameStateAttackContainer,
) -> EvaluationScore {
    let mut res = EvaluationScore::default();
    let side = if white { WHITE } else { BLACK };
    let empty = !g.get_all_pieces();
    //Bitboards
    let pawn_file_fill = bitboards::file_fill(g.pieces[PieceType::Pawn as usize][side]);
    let front_span = if white {
        bitboards::w_front_span(g.pieces[PieceType::Pawn as usize][side])
    } else {
        bitboards::b_front_span(g.pieces[PieceType::Pawn as usize][side])
    };
    let mut enemy_front_spans = if white {
        bitboards::b_front_span(g.pieces[PieceType::Pawn as usize][1 - side])
    } else {
        bitboards::w_front_span(g.pieces[PieceType::Pawn as usize][1 - side])
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
            movegen::w_single_push_pawn_targets(g.pieces[PieceType::Pawn as usize][side], empty),
            movegen::w_double_push_pawn_targets(g.pieces[PieceType::Pawn as usize][side], empty),
        )
    } else {
        (
            movegen::b_single_push_pawn_targets(g.pieces[PieceType::Pawn as usize][side], empty),
            movegen::b_double_push_pawn_targets(g.pieces[PieceType::Pawn as usize][side], empty),
        )
    };
    let is_attackable = bitboards::west_one(front_span) | bitboards::east_one(front_span);
    let enemy_pieces = g.get_pieces_from_side(1 - side);

    let doubled_pawns = (g.pieces[PieceType::Pawn as usize][side] & front_span).count_ones() as i16;
    let isolated_pawns = (g.pieces[PieceType::Pawn as usize][side]
        & !bitboards::west_one(pawn_file_fill)
        & !bitboards::east_one(pawn_file_fill))
    .count_ones() as i16;
    let backward_pawns = (if white {
        g.pieces[PieceType::Pawn as usize][side] << 8
    } else {
        g.pieces[PieceType::Pawn as usize][side] >> 8
    } & enemy_pawn_attacks
        & !is_attackable)
        .count_ones() as i16;
    let mut supported_pawns = g.pieces[PieceType::Pawn as usize][side] & my_pawn_attacks;
    let _supported_amt = supported_pawns.count_ones() as usize;
    let mut supp = EvaluationScore::default();
    while supported_pawns != 0u64 {
        let mut index = supported_pawns.trailing_zeros() as usize;
        supported_pawns ^= square(index);
        if !white {
            index = BLACK_INDEX[index];
        }
        supp += PAWN_SUPPORTED_VALUE[index / 8][index % 8];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.pawn_supported[index / 8][index % 8] += if side == WHITE { 1 } else { -1 };
        }
    }
    res += supp;
    let center_attack_pawns = (g.pieces[PieceType::Pawn as usize][side]
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
    res += PAWN_DOUBLED_VALUE * doubled_pawns
        + PAWN_ISOLATED_VALUE * isolated_pawns
        + PAWN_BACKWARD_VALUE * backward_pawns
        + PAWN_ATTACK_CENTER * center_attack_pawns
        + PAWN_MOBILITY * pawn_mobility;

    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.pawn_doubled += doubled_pawns as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.pawn_isolated += isolated_pawns as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.pawn_backward += backward_pawns as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.pawn_attack_center +=
            center_attack_pawns as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.pawn_mobility += pawn_mobility as i8 * if side == WHITE { 1 } else { -1 };
    }
    //Passers
    let mut passed_pawns: u64 = g.pieces[PieceType::Pawn as usize][side]

        /*& !if white {
            bitboards::w_rear_span(g.pieces[PieceType::Pawn as usize][side])
        } else {
            bitboards::b_rear_span(g.pieces[PieceType::Pawn as usize][side])
        }*/
        & !enemy_front_spans;
    let (mut passer_score, mut _passer_normal, mut _passer_notblocked) =
        (EvaluationScore::default(), 0, 0);
    let mut passer_dist = EvaluationScore::default();
    let mut weak_passers = 0;
    let behind_passers = if white {
        bitboards::b_front_span(passed_pawns)
    } else {
        bitboards::w_front_span(passed_pawns)
    };
    let rooks_support_passer = (behind_passers
        & (g.pieces[PieceType::Rook as usize][side] | g.pieces[PieceType::Queen as usize][side]))
        .count_ones() as i16;
    let enemy_rooks_attack_passer = (behind_passers
        & (g.pieces[PieceType::Rook as usize][1 - side]
            | g.pieces[PieceType::Queen as usize][1 - side]))
        .count_ones() as i16;
    res += ROOK_BEHIND_SUPPORT_PASSER * rooks_support_passer
        + ROOK_BEHIND_ENEMY_PASSER * enemy_rooks_attack_passer;
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.rook_behind_support_passer +=
            rooks_support_passer as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.rook_behind_enemy_passer +=
            enemy_rooks_attack_passer as i8 * if side == WHITE { 1 } else { -1 };
    }
    while passed_pawns != 0u64 {
        let idx = passed_pawns.trailing_zeros() as usize;
        //Passed and blocked
        _passer_normal += 1;
        passer_score += PAWN_PASSED_VALUES[if white { idx / 8 } else { 7 - idx / 8 }];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.pawn_passed[if white { idx / 8 } else { 7 - idx / 8 }] +=
                if side == WHITE { 1 } else { -1 };
        }
        //A weak passer is an attacked and not defended passer
        let weak_passer = square(idx) & attack_container.attacks_sum[1 - side] != 0u64
            && square(idx) & attack_container.attacks_sum[side] == 0u64;
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
            } & (attack_container.attacks_sum[1 - side] | enemy_pieces)
                & !attack_container.attacks_sum[side]
                == 0u64
        {
            //Passed and not blocked
            _passer_notblocked += 1;
            passer_score +=
                PAWN_PASSED_NOT_BLOCKED_VALUES[if white { idx / 8 } else { 7 - idx / 8 }];
            #[cfg(feature = "texel-tuning")]
            {
                _eval.trace.pawn_passed_notblocked[if white { idx / 8 } else { 7 - idx / 8 }] +=
                    if side == WHITE { 1 } else { -1 };
            }
        }

        //Distance to kings
        let d_myking = get_distance(idx as isize, g.king_square(side) as isize);
        let d_enemyking = get_distance(idx as isize, g.king_square(1 - side) as isize);
        let sub_dist = ((d_myking as isize - d_enemyking as isize) + 6) as usize;
        passer_dist += PASSED_KING_DISTANCE[d_myking - 1]
            + PASSED_ENEMY_KING_DISTANCE[d_enemyking - 1]
            + PASSED_SUBTRACT_DISTANCE[sub_dist];
        #[cfg(feature = "texel-tuning")]
        {
            _eval.trace.pawn_passed_kingdistance[d_myking - 1] +=
                if side == WHITE { 1 } else { -1 };
            _eval.trace.pawn_passed_enemykingdistance[d_enemyking - 1] +=
                if side == WHITE { 1 } else { -1 };
            _eval.trace.pawn_passed_subdistance[sub_dist] += if side == WHITE { 1 } else { -1 };
        }
        passed_pawns ^= square(idx);
    }
    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.pawn_passed_weak += weak_passers as i8 * if side == WHITE { 1 } else { -1 };
    }
    res += passer_score + PAWN_PASSED_WEAK * weak_passers + passer_dist;
    #[cfg(feature = "display-eval")]
    {
        println!("\nPawns for {}:", if white { "White" } else { "Black" });
        println!(
            "\tDoubled: {} -> {}",
            doubled_pawns,
            PAWN_DOUBLED_VALUE * doubled_pawns
        );
        println!(
            "\tIsolated: {} -> {}",
            isolated_pawns,
            PAWN_ISOLATED_VALUE * isolated_pawns,
        );
        println!(
            "\tBackward: {} -> {}",
            backward_pawns,
            PAWN_BACKWARD_VALUE * backward_pawns,
        );
        println!("\tSupported: {} -> {}", _supported_amt, supp);
        println!(
            "\tAttack Center: {} -> {}",
            center_attack_pawns,
            PAWN_ATTACK_CENTER * center_attack_pawns,
        );
        println!(
            "\tMobility: {} -> {}",
            pawn_mobility,
            PAWN_MOBILITY * pawn_mobility,
        );
        println!(
            "\tPasser Blocked/Not Blocked: {} , {} -> {}",
            _passer_normal, _passer_notblocked, passer_score
        );
        println!(
            "\tRook behind passer: {} -> {}",
            rooks_support_passer,
            ROOK_BEHIND_SUPPORT_PASSER * rooks_support_passer,
        );
        println!(
            "\tEnemy Rook behind passer: {} -> {}",
            enemy_rooks_attack_passer,
            ROOK_BEHIND_ENEMY_PASSER * enemy_rooks_attack_passer,
        );
        println!(
            "\tWeak passer: {} -> {}",
            weak_passers,
            PAWN_PASSED_WEAK * weak_passers,
        );
        println!("\tPassers distance to kings -> {}", passer_dist);
        println!("Sum: {}", res);
    }
    res
}

pub fn piece_values(white: bool, g: &GameState, _eval: &mut EvaluationResult) -> EvaluationScore {
    let mut res = EvaluationScore::default();
    let side = if white { WHITE } else { BLACK };

    let my_pawns = g.pieces[PieceType::Pawn as usize][side].count_ones() as i16;
    let mut my_knights = g.pieces[PieceType::Knight as usize][side].count_ones() as i16;
    let mut my_bishops = g.pieces[PieceType::Bishop as usize][side].count_ones() as i16;
    let my_rooks = g.pieces[PieceType::Rook as usize][side].count_ones() as i16;
    let my_queens = g.pieces[PieceType::Queen as usize][side].count_ones() as i16;
    if my_pawns + my_knights + my_bishops + my_rooks + my_queens == 1 {
        my_knights = 0;
        my_bishops = 0;
    }
    res += PAWN_PIECE_VALUE * my_pawns;

    let pawns_on_board = (g.pieces[PieceType::Pawn as usize][WHITE]
        | g.pieces[PieceType::Pawn as usize][BLACK])
        .count_ones() as usize;

    res += (KNIGHT_PIECE_VALUE + KNIGHT_VALUE_WITH_PAWNS[pawns_on_board]) * my_knights;

    res += BISHOP_PIECE_VALUE * my_bishops;
    if my_bishops > 1 {
        res += BISHOP_PAIR_BONUS;
    }

    res += ROOK_PIECE_VALUE * my_rooks;

    res += QUEEN_PIECE_VALUE * my_queens;

    #[cfg(feature = "texel-tuning")]
    {
        _eval.trace.pawns += my_pawns as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.knight_value_with_pawns = pawns_on_board as u8;
        _eval.trace.knights += my_knights as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.bishops += my_bishops as i8 * if side == WHITE { 1 } else { -1 };
        if my_bishops > 1 {
            _eval.trace.bishop_bonus += if side == WHITE { 1 } else { -1 };
        }
        _eval.trace.rooks += my_rooks as i8 * if side == WHITE { 1 } else { -1 };
        _eval.trace.queens += my_queens as i8 * if side == WHITE { 1 } else { -1 };
    }
    #[cfg(feature = "display-eval")]
    {
        println!(
            "\nPiece values for {}",
            if white { "White" } else { "Black" }
        );
        println!("\tPawns: {} -> {}", my_pawns, PAWN_PIECE_VALUE * my_pawns,);
        println!(
            "\tKnights: {} -> {}",
            my_knights,
            (KNIGHT_PIECE_VALUE + KNIGHT_VALUE_WITH_PAWNS[pawns_on_board]) * my_knights,
        );
        println!(
            "\tBishops: {} -> {}",
            my_bishops,
            BISHOP_PIECE_VALUE * my_bishops,
        );
        if my_bishops > 1 {
            println!("\tBishop-Pair: {} -> {}", 1, BISHOP_PAIR_BONUS);
        }
        println!("\tRooks: {} -> {}", my_rooks, ROOK_PIECE_VALUE * my_rooks,);
        println!(
            "\tQueens: {} -> {}",
            my_queens,
            QUEEN_PIECE_VALUE * my_queens,
        );
        println!("Sum: {}", res);
    }
    res
}
