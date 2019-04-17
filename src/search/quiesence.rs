use super::super::board_representation::game_state::{GameState, GameMove, GameMoveType, PieceType};
use super::super::evaluation::{eval_game_state, self};
use super::super::move_generation::movegen;
use super::alphabeta::{GameResult, check_end_condition, leaf_score};
use super::search::Search;
use super::alphabeta::PrincipalVariation;
use super::cache::CacheEntry;
use super::statistics::SearchStatistics;
use crate::bitboards;
use super::GradedMove;

pub const DELTA_PRUNING: f64 = 2.0;
lazy_static! {
pub static ref PIECE_VALUES:[f64;6] = [100.0,300.0,310.0,500.0,900.0,999999999.99];
}

pub fn q_search(mut alpha: f64, mut beta: f64, game_state: &GameState, color: isize, depth_left: isize, stats: &mut SearchStatistics, legal_moves: Vec<GameMove>, in_check: bool, current_depth: usize, search: &mut Search) -> PrincipalVariation {
    stats.add_q_node(current_depth);

    let mut pv: PrincipalVariation = PrincipalVariation::new(1);
    if search.stop {
        return pv;
    }
    let game_status = check_end_condition(&game_state, legal_moves.len() > 0, in_check, &search);
    if game_status != GameResult::Ingame {
        pv.score = leaf_score(game_status, color, depth_left);
        return pv;
    }

    let evaluation = eval_game_state(&game_state);
    let stand_pat = evaluation.final_eval * color as f64;
    if stand_pat >= beta {
        pv.score = stand_pat;
        return pv;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    //Apply Big Delta Pruning
    let diff = alpha - stand_pat - DELTA_PRUNING;
    if diff > 0.0 && best_move_value(game_state) < diff {
        pv.score = stand_pat;
        return pv;
    }

    let mut capture_moves = Vec::with_capacity(20);
    for mv in legal_moves {
        if !is_capture(&mv) {
            continue;
        }
        if let GameMoveType::EnPassant = mv.move_type {
            capture_moves.push(GradedMove::new(mv, 100.0));
        } else {
            if !passes_delta_pruning(&mv, evaluation.phase, stand_pat, alpha) {
                stats.add_q_delta_cutoff();
                continue;
            }
            let score = see(&game_state, &mv, false);
            if score < 0.0 {
                stats.add_q_see_cutoff();
                continue;
            }
            capture_moves.push(GradedMove::new(mv, score));
        }
    }

    //Probe TT
    {
        let ce = &search.cache.cache[game_state.hash as usize & super::cache::CACHE_MASK];
        if let Some(s) = ce {
            let ce: &CacheEntry = s;
            if ce.hash == game_state.hash {
                stats.add_cache_hit_qs();
                if ce.occurences == 0 && ce.depth >= depth_left as i8 {
                    if !ce.alpha && !ce.beta {
                        stats.add_cache_hit_replace_qs();
                        pv.pv.push(CacheEntry::u16_to_mv(ce.mv, &game_state));
                        pv.score = ce.score;
                        return pv;
                    } else {
                        if ce.beta {
                            if ce.score > alpha {
                                alpha = ce.score;
                            }
                        } else {
                            if ce.score < beta {
                                beta = ce.score;
                            }
                        }
                        if alpha >= beta {
                            stats.add_cache_hit_aj_replace_qs();
                            pv.score = ce.score;
                            pv.pv.push(CacheEntry::u16_to_mv(ce.mv, &game_state));
                            return pv;
                        }
                    }
                }
                let mv = CacheEntry::u16_to_mv(ce.mv, &game_state);
                for cmv in capture_moves.iter_mut() {
                    if cmv.mv.from == mv.from && cmv.mv.to == mv.to && cmv.mv.move_type == mv.move_type {
                        cmv.score += 10000.0;
                        break;
                    }
                }
            }
        }
    }

    let mut index = 0;
    pv.score = stand_pat;
    while capture_moves.len() > 0 {
        let gmvindex = super::alphabeta::get_next_gm(&capture_moves);
        let capture_move = capture_moves.remove(gmvindex);
        let mv = capture_move.mv;
        let next_g = movegen::make_move(&game_state, &mv);
        let next_g_movegen = movegen::generate_moves(&next_g);
        let mut following_pv = q_search(-beta, -alpha, &next_g, -color, depth_left - 1, stats, next_g_movegen.0, next_g_movegen.1, current_depth + 1, search);
        let score = -following_pv.score;
        if score > pv.score {
            pv.score = score;
            pv.pv.clear();
            pv.pv.push(mv);
        }
        if score >= beta {
            stats.add_q_beta_cutoff(index);
            break;
        }
        if score > alpha {
            alpha = score;
        }
        index += 1;
    }
    if pv.score < beta {
        if index > 0 {
            stats.add_q_beta_noncutoff();
        }
    }
    if pv.pv.len() > 0 {
        super::alphabeta::make_cache(search, &pv, &game_state, alpha, beta, depth_left);
    }
    pv
}

pub fn is_capture(mv: &GameMove) -> bool {
    match &mv.move_type {
        GameMoveType::Capture(_) => true,
        GameMoveType::Promotion(_, s) => match s {
            Some(s) => true,
            _ => false
        },
        GameMoveType::EnPassant => true,
        _ => false,
    }
}

pub fn best_move_value(state: &GameState) -> f64 {
    let mut res = 1.0;
    let mut i = 4;
    while i > 0 {
        if state.pieces[i][1 - state.color_to_move] != 0u64 {
            res = PIECE_VALUES[i] / 100.0;
            break;
        }
        i -= 1;
    }

    if (state.pieces[0][state.color_to_move] & bitboards::RANKS[if state.color_to_move == 0 { 6 } else { 1 }]) != 0u64 {
        res += (PIECE_VALUES[4] - PIECE_VALUES[0]) / 100.0;
    }
    res
}

pub fn passes_delta_pruning(capture_move: &GameMove, phase: f64, eval: f64, alpha: f64) -> bool {
    if phase == 0.0 {
        return true;
    }
    if let GameMoveType::Promotion(_, _) = capture_move.move_type {
        return true;
    }
    let captured_piece = match &capture_move.move_type {
        GameMoveType::Capture(c) => c,
        GameMoveType::EnPassant => &PieceType::Pawn,
        _ => panic!("No capture!")
    };
    eval + (evaluation::piece_value(&captured_piece, phase)) / 100.0 + DELTA_PRUNING >= alpha
}

pub fn see(game_state: &GameState, mv: &GameMove, exact: bool) -> f64 {
    let mut gain = Vec::with_capacity(32);
    let may_xray = game_state.pieces[0][0] | game_state.pieces[0][1] | game_state.pieces[2][0] | game_state.pieces[2][1] | game_state.pieces[3][0] | game_state.pieces[3][1] |
        game_state.pieces[4][0] | game_state.pieces[4][1];
    let mut from_set = 1u64 << mv.from;
    let mut occ = get_occupied_board(&game_state);
    let mut attadef = attacks_to(&game_state, mv.to, occ);
    gain.push(capture_value(&mv));
    let mut color_to_move = game_state.color_to_move;
    let mut attacked_piece = match mv.piece_type {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => 5
    };
    let mut index = 0;
    let mut deleted_pieces = 0u64;
    while from_set != 0u64 {
        deleted_pieces |= from_set;
        index += 1;
        gain.push(PIECE_VALUES[attacked_piece] - gain[index - 1]);
        if !exact && (-gain[index - 1]).max(gain[index]) < 0.0 {
            break;
        }
        attadef ^= from_set;
        occ ^= from_set;
        if from_set & may_xray != 0u64 {
            //Recalculate rays
            attadef |= recalculate_sliders(&game_state, color_to_move, mv.to, occ) & (!deleted_pieces);
        }
        color_to_move = 1 - color_to_move;
        let res = least_valuable_piece(attadef, color_to_move, &game_state);
        from_set = res.0;
        attacked_piece = res.1;
        if attacked_piece == 5 && least_valuable_piece(attadef, 1 - color_to_move, &game_state).1 != 1000 {
            break;
        }
    }
    while index > 1 {
        index -= 1;
        gain[index - 1] = -((-gain[index - 1]).max(gain[index]));
    }
    gain[0]
}

pub fn recalculate_sliders(game_state: &GameState, color_to_move: usize, square: usize, occ: u64) -> u64 {
    //Bishops
    movegen::bishop_attack(square, occ) & (game_state.pieces[2][color_to_move] | game_state.pieces[4][color_to_move])
        | movegen::rook_attack(square, occ) & (game_state.pieces[3][color_to_move] | game_state.pieces[4][color_to_move])
}

pub fn attacks_to(game_state: &GameState, square: usize, occ: u64) -> u64 {
    let square_board = 1u64 << square;
    movegen::attackers_from_white(square_board, square, game_state.pieces[0][0], game_state.pieces[1][0], game_state.pieces[2][0] | game_state.pieces[4][0], game_state.pieces[3][0] | game_state.pieces[4][0], occ).0
        | movegen::attackers_from_black(square_board, square, game_state.pieces[0][1], game_state.pieces[1][1], game_state.pieces[2][1] | game_state.pieces[4][1], game_state.pieces[3][1] | game_state.pieces[4][1], occ).0
        | bitboards::KING_ATTACKS[square] & (game_state.pieces[5][0] | game_state.pieces[5][1])
}

pub fn capture_value(mv: &GameMove) -> f64 {
    match &mv.move_type {
        GameMoveType::Capture(c) => {
            piece_value(&c)
        }
        GameMoveType::Promotion(_, b) => {
            match b {
                Some(c) => {
                    piece_value(&c)
                }
                _ => panic!("Promotion but no capture")
            }
        }
        _ => panic!("No capture")
    }
}

pub fn piece_value(piece_type: &PieceType) -> f64 {
    match piece_type {
        PieceType::Pawn => 100.0,
        PieceType::Knight => 300.0,
        PieceType::Bishop => 310.0,
        PieceType::Rook => 500.0,
        PieceType::Queen => 900.0,
        PieceType::King => 999999999999.99,
    }
}

pub fn get_occupied_board(game_state: &GameState) -> u64 {
    game_state.pieces[0][0] | game_state.pieces[1][0] | game_state.pieces[2][0] | game_state.pieces[3][0] | game_state.pieces[4][0] | game_state.pieces[5][0]
        | game_state.pieces[0][1] | game_state.pieces[1][1] | game_state.pieces[2][1] | game_state.pieces[3][1] | game_state.pieces[4][1] | game_state.pieces[5][1]
}

pub fn least_valuable_piece(from_board: u64, color_to_move: usize, game_state: &GameState) -> (u64, usize) {
    for i in 0..6 {
        let subset = game_state.pieces[i][color_to_move] & from_board;
        if subset != 0u64 {
            return (1u64 << subset.trailing_zeros(), i);
        }
    }
    (0u64, 1000)
}

#[cfg(test)]
mod tests {
    use super::GameState;
    use super::GameMove;
    use super::GameMoveType;
    use super::PieceType;
    use super::see;
    use super::q_search;
    use super::SearchStatistics;
    use super::movegen;

    #[test]
    fn see_test() {
        assert_eq!(see(&GameState::from_fen("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -"), &GameMove {
            from: 4usize,
            to: 36usize,
            move_type: GameMoveType::Capture(PieceType::Pawn),
            piece_type: PieceType::Rook,
        }, true), 100.0);
        assert_eq!(see(&GameState::from_fen("1k2r3/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -"), &GameMove {
            from: 4usize,
            to: 36usize,
            move_type: GameMoveType::Capture(PieceType::Pawn),
            piece_type: PieceType::Rook,
        }, true), -400.0);
        assert_eq!(see(&GameState::from_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -"), &GameMove {
            from: 19,
            to: 36,
            move_type: GameMoveType::Capture(PieceType::Pawn),
            piece_type: PieceType::Knight,
        }, true), -200.0);
        assert_eq!(see(&GameState::from_fen("1k1r3q/1ppn3p/p4b2/4n3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -"), &GameMove {
            from: 19,
            to: 36,
            move_type: GameMoveType::Capture(PieceType::Knight),
            piece_type: PieceType::Knight,
        }, true), 0.0);
        assert_eq!(see(&GameState::from_fen("1k1r2q1/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -"), &GameMove {
            from: 19,
            to: 36,
            move_type: GameMoveType::Capture(PieceType::Pawn),
            piece_type: PieceType::Knight,
        }, true), -90.0);
        assert_eq!(see(&GameState::from_fen("8/8/3p4/4r3/2RKP3/5k2/8/8 b - -"), &GameMove {
            from: 36,
            to: 28,
            move_type: GameMoveType::Capture(PieceType::Pawn),
            piece_type: PieceType::Rook,
        }, true), 100.0);
        assert_eq!(see(&GameState::from_fen("k7/8/5q2/8/3r4/2KQ4/8/8 w - -"), &GameMove {
            from: 19,
            to: 27,
            move_type: GameMoveType::Capture(PieceType::Rook),
            piece_type: PieceType::Queen,
        }, true), 500.0);
        assert_eq!(see(&GameState::from_fen("8/8/5q2/2k5/3r4/2KQ4/8/8 w - -"), &GameMove {
            from: 19,
            to: 27,
            move_type: GameMoveType::Capture(PieceType::Rook),
            piece_type: PieceType::Queen,
        }, true), -400.0);
        assert_eq!(see(&GameState::from_fen("4pq2/3P4/8/8/8/8/8/k1K5 w - -"), &GameMove {
            from: 51,
            to: 60,
            move_type: GameMoveType::Promotion(PieceType::Queen, Some(PieceType::Pawn)),
            piece_type: PieceType::Pawn,
        }, true), 0.0);
        assert_eq!(see(&GameState::from_fen("4pq2/3P4/2B5/8/8/8/8/k1K5 w - -"), &GameMove {
            from: 51,
            to: 60,
            move_type: GameMoveType::Promotion(PieceType::Queen, Some(PieceType::Pawn)),
            piece_type: PieceType::Pawn,
        }, true), 100.0);
    }

    #[test]
    fn q_test() {
        let mut stats = SearchStatistics::new();
        let state = GameState::standard();
        let movegen = movegen::generate_moves(&state);
        assert_eq!(q_search(-100000.0, 100000.0, &state, 1, 0, &mut stats, movegen.0, movegen.1, 0), 0.0);
        assert_eq!(stats.seldepth, 0);
        assert_eq!(stats.nodes_searched, 1);
    }
}