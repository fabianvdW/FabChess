use super::super::board_representation::game_state::{
    GameMove, GameMoveType, GameResult, GameState, PieceType, BISHOP, BLACK, KING, KNIGHT, PAWN,
    QUEEN, ROOK, WHITE,
};
use super::super::evaluation::{self, eval_game_state};
use super::super::move_generation::movegen;
use super::super::move_generation::movegen::{AdditionalGameStateInformation, MoveList};
use super::alphabeta::{
    check_end_condition, check_for_draw, clear_pv, get_next_gm, in_check, leaf_score,
};
use super::cache::CacheEntry;
use super::searcher::{Search, SearchUtils};
use super::GradedMove;
use super::{MAX_SEARCH_DEPTH, STANDARD_SCORE};
use crate::bitboards;
use crate::board_representation::game_state_attack_container::GameStateAttackContainer;
use crate::move_generation::makemove::make_move;

pub const DELTA_PRUNING: i16 = 100;
lazy_static! {
    pub static ref PIECE_VALUES: [i16; 6] = [100, 300, 310, 500, 900, 30000];
}

pub fn q_search(
    mut alpha: i16,
    mut beta: i16,
    game_state: &GameState,
    color: i16,
    depth_left: i16,
    current_depth: usize,
    su: &mut SearchUtils,
) -> i16 {
    su.search.search_statistics.add_q_node(current_depth);
    clear_pv(current_depth, su.search);
    if su.search.stop {
        return STANDARD_SCORE;
    }
    //Initialzie attack container
    //Max search-depth reached
    if current_depth >= (MAX_SEARCH_DEPTH - 1) {
        su.thread_memory.reserved_attack_container.attack_containers[current_depth]
            .write_state(game_state);
        return eval_game_state(
            &game_state,
            &su.thread_memory.reserved_attack_container.attack_containers[current_depth],
        )
        .final_eval
            * color;
    }

    //check for draw
    if check_for_draw(game_state, su.history) {
        return leaf_score(GameResult::Draw, color, current_depth as i16);
    }
    su.thread_memory.reserved_attack_container.attack_containers[current_depth]
        .write_state(game_state);

    let incheck = in_check(
        game_state,
        &su.thread_memory.reserved_attack_container.attack_containers[current_depth],
    );

    let (phase, stand_pat) = if !incheck {
        let static_evaluation = eval_game_state(
            &game_state,
            &su.thread_memory.reserved_attack_container.attack_containers[current_depth],
        );

        (
            Some(static_evaluation.phase),
            Some(static_evaluation.final_eval * color),
        )
    } else {
        (None, None)
    };
    if !incheck {
        //Stand pat
        let stand_pat = *stand_pat.as_ref().unwrap();
        if stand_pat >= beta {
            return stand_pat;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }
        //Delta pruning
        let diff = alpha - stand_pat - DELTA_PRUNING;
        //Missing stats
        if diff > 0 && best_move_value(game_state) < diff {
            return stand_pat;
        }
    }

    let mut tt_move: Option<GameMove> = None;
    let mut has_ttmove = false;
    //Probe TT
    {
        let ce = &su.cache.cache[game_state.hash as usize & super::cache::CACHE_MASK];
        if let Some(s) = ce {
            let ce: &CacheEntry = s;
            if ce.hash == game_state.hash {
                su.search.search_statistics.add_cache_hit_qs();
                if ce.depth >= depth_left as i8 {
                    if !ce.alpha && !ce.beta {
                        su.search.search_statistics.add_cache_hit_replace_qs();
                        su.search.pv_table[current_depth].pv[0] =
                            Some(CacheEntry::u16_to_mv(ce.mv, &game_state));
                        return ce.score;
                    } else {
                        if ce.beta {
                            if ce.score > alpha {
                                alpha = ce.score;
                            }
                        } else if ce.score < beta {
                            beta = ce.score;
                        }
                        if alpha >= beta {
                            su.search.search_statistics.add_cache_hit_aj_replace_qs();
                            su.search.pv_table[current_depth].pv[0] =
                                Some(CacheEntry::u16_to_mv(ce.mv, &game_state));
                            return ce.score;
                        }
                    }
                }
                let mv = CacheEntry::u16_to_mv(ce.mv, &game_state);
                tt_move = Some(mv);
                has_ttmove = true;
            }
        }
    }

    let hash_move_counter = if has_ttmove { 1 } else { 0 };
    let mut has_legal_move = false;

    su.history.push(game_state.hash, game_state.half_moves == 0);
    let mut current_max_score = if incheck {
        STANDARD_SCORE
    } else {
        *stand_pat.as_ref().unwrap()
    };
    let mut has_pv = false;

    let mut index = 0;
    let mut moves_from_movelist_tried: usize = 0;
    let mut has_generated_moves = false;

    let mut available_captures_in_movelist = 0;

    while index < available_captures_in_movelist + hash_move_counter || !has_generated_moves {
        if index == hash_move_counter && !has_generated_moves {
            has_generated_moves = true;
            let (agsi, mvs) = make_and_evaluate_moves_qsearch(
                game_state,
                su.search,
                &mut su.thread_memory.reserved_movelist.move_lists[current_depth],
                &su.thread_memory.reserved_attack_container.attack_containers[current_depth],
                phase,
                stand_pat,
                alpha,
                incheck,
            );
            has_legal_move = agsi.stm_haslegalmove;
            available_captures_in_movelist = mvs;
            continue;
        }
        let capture_move: GameMove = if index < hash_move_counter {
            tt_move.expect("Couldn't unwrap tt move in q search")
        } else {
            let r = get_next_gm(
                &mut su.thread_memory.reserved_movelist.move_lists[current_depth],
                moves_from_movelist_tried,
                available_captures_in_movelist,
            )
            .0;
            su.thread_memory.reserved_movelist.move_lists[current_depth].move_list[r]
                .expect("Could not get next gm")
        };
        //Make sure that our move is not the same as tt move if we have any
        if index >= hash_move_counter {
            moves_from_movelist_tried += 1;
            if hash_move_counter > 0
                && *tt_move
                    .as_ref()
                    .expect("Couldn't unwrap hash move counter in move check")
                    == capture_move
            {
                index += 1;
                continue;
            }
        }
        let next_g = make_move(&game_state, &capture_move);
        let score = -q_search(
            -beta,
            -alpha,
            &next_g,
            -color,
            depth_left - 1,
            current_depth + 1,
            su,
        );
        if score > current_max_score {
            current_max_score = score;
            su.search.pv_table[current_depth].pv[0] = Some(capture_move);
            has_pv = true;
            //Hang on following pv in theory
        }
        if score >= beta {
            su.search.search_statistics.add_q_beta_cutoff(index);
            break;
        }
        if score > alpha {
            alpha = score;
        }
        index += 1;
    }
    su.history.pop();
    if current_max_score < beta && index > 0 {
        su.search.search_statistics.add_q_beta_noncutoff();
    }
    let game_status = check_end_condition(&game_state, has_legal_move, incheck);
    if game_status != GameResult::Ingame {
        clear_pv(current_depth, su.search);
        return leaf_score(game_status, color, current_depth as i16);
    }
    if has_pv {
        super::alphabeta::make_cache(
            su.cache,
            &su.search.pv_table[current_depth],
            current_max_score,
            &game_state,
            alpha,
            beta,
            0,
            su.root_pliesplayed,
            if incheck {
                None
            } else {
                Some(*stand_pat.as_ref().unwrap() * color)
            },
            false,
        );
    }
    current_max_score
}

#[inline(always)]
pub fn make_and_evaluate_moves_qsearch(
    game_state: &GameState,
    search: &mut Search,
    move_list: &mut MoveList,
    attack_container: &GameStateAttackContainer,
    phase: Option<f64>,
    stand_pat: Option<i16>,
    alpha: i16,
    incheck: bool,
) -> (AdditionalGameStateInformation, usize) {
    let agsi = movegen::generate_moves(&game_state, !incheck, move_list, attack_container);
    let (mut mv_index, mut capture_index) = (0, 0);
    while mv_index < move_list.counter {
        let mv: &GameMove = move_list.move_list[mv_index].as_ref().unwrap();
        if let GameMoveType::EnPassant = mv.move_type {
            move_list.graded_moves[capture_index] = Some(GradedMove::new(mv_index, 99.0));
        } else {
            if !incheck
                && !passes_delta_pruning(
                    mv,
                    *phase.as_ref().unwrap(),
                    *stand_pat.as_ref().unwrap(),
                    alpha,
                )
            {
                search.search_statistics.add_q_delta_cutoff();
                mv_index += 1;
                continue;
            }
            if capture_index > 0 && is_capture(mv) || !incheck {
                let score = see(&game_state, mv, true, &mut search.see_buffer);
                if score < 0 {
                    search.search_statistics.add_q_see_cutoff();
                    mv_index += 1;
                    continue;
                }
                move_list.graded_moves[capture_index] =
                    Some(GradedMove::new(mv_index, f64::from(score)));
            } else {
                if !incheck {
                    panic!("Not in check but also not capture");
                }
                let score = search.hh_score[game_state.color_to_move][mv.from][mv.to] as f64
                    / search.bf_score[game_state.color_to_move][mv.from][mv.to] as f64
                    / 1000.0;
                move_list.graded_moves[capture_index] = Some(GradedMove::new(mv_index, score));
            }
        }
        mv_index += 1;
        capture_index += 1;
    }
    (agsi, capture_index)
}
#[inline(always)]
pub fn is_capture(mv: &GameMove) -> bool {
    match &mv.move_type {
        GameMoveType::Capture(_) => true,
        GameMoveType::Promotion(_, s) => match s {
            Some(_) => true,
            _ => false,
        },
        GameMoveType::EnPassant => true,
        _ => false,
    }
}

#[inline(always)]
pub fn best_move_value(state: &GameState) -> i16 {
    let mut res = 0;
    let mut i = 4;
    while i > 0 {
        if state.pieces[i][1 - state.color_to_move] != 0u64 {
            res = PIECE_VALUES[i];
            break;
        }
        i -= 1;
    }

    if (state.pieces[PAWN][state.color_to_move]
        & bitboards::RANKS[if state.color_to_move == WHITE { 6 } else { 1 }])
        != 0u64
    {
        res += PIECE_VALUES[QUEEN] - PIECE_VALUES[PAWN];
    }
    res
}

#[inline(always)]
pub fn passes_delta_pruning(capture_move: &GameMove, phase: f64, eval: i16, alpha: i16) -> bool {
    if phase == 0.0 || eval >= alpha {
        return true;
    }
    if let GameMoveType::Promotion(_, _) = capture_move.move_type {
        return true;
    }
    let captured_piece = match &capture_move.move_type {
        GameMoveType::Capture(c) => c,
        GameMoveType::EnPassant => &PieceType::Pawn,
        _ => panic!("No capture!"),
    };
    eval + evaluation::piece_value(*captured_piece, phase) + DELTA_PRUNING >= alpha
}

#[inline(always)]
pub fn see(game_state: &GameState, mv: &GameMove, exact: bool, gain: &mut Vec<i16>) -> i16 {
    let may_xray = game_state.pieces[PAWN][WHITE]
        | game_state.pieces[PAWN][BLACK]
        | game_state.pieces[BISHOP][WHITE]
        | game_state.pieces[BISHOP][BLACK]
        | game_state.pieces[ROOK][WHITE]
        | game_state.pieces[ROOK][BLACK]
        | game_state.pieces[QUEEN][WHITE]
        | game_state.pieces[QUEEN][BLACK];
    let mut from_set = 1u64 << mv.from;
    let mut occ = game_state.get_all_pieces();
    let mut attadef = attacks_to(&game_state, mv.to, occ);
    gain[0] = capture_value(&mv);
    let mut color_to_move = game_state.color_to_move;
    let mut attacked_piece = match mv.piece_type {
        PieceType::Pawn => PAWN,
        PieceType::Knight => KNIGHT,
        PieceType::Bishop => BISHOP,
        PieceType::Rook => ROOK,
        PieceType::Queen => QUEEN,
        PieceType::King => KING,
    };
    let mut index = 0;
    let mut deleted_pieces = 0u64;
    while from_set != 0u64 {
        deleted_pieces |= from_set;
        index += 1;
        gain[index] = PIECE_VALUES[attacked_piece] - gain[index - 1];
        if !exact && (-gain[index - 1]).max(gain[index]) < 0 {
            break;
        }
        attadef ^= from_set;
        occ ^= from_set;
        if from_set & may_xray != 0u64 {
            //Recalculate rays
            attadef |=
                recalculate_sliders(&game_state, color_to_move, mv.to, occ) & (!deleted_pieces);
        }
        color_to_move = 1 - color_to_move;
        let res = least_valuable_piece(attadef, color_to_move, &game_state);
        from_set = res.0;
        attacked_piece = res.1;
        if attacked_piece == 5
            && least_valuable_piece(attadef, 1 - color_to_move, &game_state).1 != 1000
        {
            break;
        }
    }
    while index > 1 {
        index -= 1;
        gain[index - 1] = -((-gain[index - 1]).max(gain[index]));
    }
    gain[0]
}

#[inline(always)]
pub fn recalculate_sliders(
    game_state: &GameState,
    color_to_move: usize,
    square: usize,
    occ: u64,
) -> u64 {
    //Bishops
    movegen::bishop_attack(square, occ)
        & (game_state.pieces[BISHOP][color_to_move] | game_state.pieces[QUEEN][color_to_move])
        | movegen::rook_attack(square, occ)
            & (game_state.pieces[ROOK][color_to_move] | game_state.pieces[QUEEN][color_to_move])
}

#[inline(always)]
pub fn attacks_to(game_state: &GameState, square: usize, occ: u64) -> u64 {
    let square_board = 1u64 << square;
    let mut attacks = 0u64;
    let knights = game_state.pieces[KNIGHT][WHITE] | game_state.pieces[KNIGHT][BLACK];
    let bishops = game_state.pieces[BISHOP][WHITE]
        | game_state.pieces[QUEEN][WHITE]
        | game_state.pieces[BISHOP][BLACK]
        | game_state.pieces[QUEEN][BLACK];
    let rooks = game_state.pieces[ROOK][WHITE]
        | game_state.pieces[QUEEN][WHITE]
        | game_state.pieces[ROOK][BLACK]
        | game_state.pieces[QUEEN][BLACK];
    attacks |= movegen::knight_attack(square) & knights
        | movegen::bishop_attack(square, occ) & bishops
        | movegen::rook_attack(square, occ) & rooks;
    attacks |= (movegen::w_pawn_west_targets(square_board)
        | movegen::w_pawn_east_targets(square_board))
        & game_state.pieces[PAWN][BLACK];
    attacks |= (movegen::b_pawn_west_targets(square_board)
        | movegen::b_pawn_east_targets(square_board))
        & game_state.pieces[PAWN][WHITE];
    attacks |= bitboards::KING_ATTACKS[square]
        & (game_state.pieces[KING][WHITE] | game_state.pieces[KING][BLACK]);
    attacks
}

#[inline(always)]
pub fn capture_value(mv: &GameMove) -> i16 {
    match &mv.move_type {
        GameMoveType::Capture(c) => piece_value(*c),
        GameMoveType::Promotion(_, b) => match b {
            Some(c) => piece_value(*c),
            _ => panic!("Promotion but no capture"),
        },
        _ => panic!("No capture"),
    }
}

#[inline(always)]
pub fn piece_value(piece_type: PieceType) -> i16 {
    match piece_type {
        PieceType::Pawn => PIECE_VALUES[PAWN],
        PieceType::Knight => PIECE_VALUES[KNIGHT],
        PieceType::Bishop => PIECE_VALUES[BISHOP],
        PieceType::Rook => PIECE_VALUES[ROOK],
        PieceType::Queen => PIECE_VALUES[QUEEN],
        PieceType::King => PIECE_VALUES[KING],
    }
}

#[inline(always)]
pub fn least_valuable_piece(
    from_board: u64,
    color_to_move: usize,
    game_state: &GameState,
) -> (u64, usize) {
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
    use super::see;
    use super::GameMove;
    use super::GameMoveType;
    use super::GameState;
    use super::PieceType;

    #[test]
    fn see_test() {
        let mut see_buffer = vec![0i16; 128];
        assert_eq!(
            see(
                &GameState::from_fen("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -"),
                &GameMove {
                    from: 4usize,
                    to: 36usize,
                    move_type: GameMoveType::Capture(PieceType::Pawn),
                    piece_type: PieceType::Rook,
                },
                true,
                &mut see_buffer
            ),
            100
        );
        assert_eq!(
            see(
                &GameState::from_fen("1k2r3/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -"),
                &GameMove {
                    from: 4usize,
                    to: 36usize,
                    move_type: GameMoveType::Capture(PieceType::Pawn),
                    piece_type: PieceType::Rook,
                },
                true,
                &mut see_buffer
            ),
            -400
        );
        assert_eq!(
            see(
                &GameState::from_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -"),
                &GameMove {
                    from: 19,
                    to: 36,
                    move_type: GameMoveType::Capture(PieceType::Pawn),
                    piece_type: PieceType::Knight,
                },
                true,
                &mut see_buffer
            ),
            -200
        );
        assert_eq!(
            see(
                &GameState::from_fen("1k1r3q/1ppn3p/p4b2/4n3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -"),
                &GameMove {
                    from: 19,
                    to: 36,
                    move_type: GameMoveType::Capture(PieceType::Knight),
                    piece_type: PieceType::Knight,
                },
                true,
                &mut see_buffer
            ),
            0
        );
        assert_eq!(
            see(
                &GameState::from_fen("1k1r2q1/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -"),
                &GameMove {
                    from: 19,
                    to: 36,
                    move_type: GameMoveType::Capture(PieceType::Pawn),
                    piece_type: PieceType::Knight,
                },
                true,
                &mut see_buffer
            ),
            -90
        );
        assert_eq!(
            see(
                &GameState::from_fen("8/8/3p4/4r3/2RKP3/5k2/8/8 b - -"),
                &GameMove {
                    from: 36,
                    to: 28,
                    move_type: GameMoveType::Capture(PieceType::Pawn),
                    piece_type: PieceType::Rook,
                },
                true,
                &mut see_buffer
            ),
            100
        );
        assert_eq!(
            see(
                &GameState::from_fen("k7/8/5q2/8/3r4/2KQ4/8/8 w - -"),
                &GameMove {
                    from: 19,
                    to: 27,
                    move_type: GameMoveType::Capture(PieceType::Rook),
                    piece_type: PieceType::Queen,
                },
                true,
                &mut see_buffer
            ),
            500
        );
        assert_eq!(
            see(
                &GameState::from_fen("8/8/5q2/2k5/3r4/2KQ4/8/8 w - -"),
                &GameMove {
                    from: 19,
                    to: 27,
                    move_type: GameMoveType::Capture(PieceType::Rook),
                    piece_type: PieceType::Queen,
                },
                true,
                &mut see_buffer
            ),
            -400
        );
        assert_eq!(
            see(
                &GameState::from_fen("4pq2/3P4/8/8/8/8/8/k1K5 w - -"),
                &GameMove {
                    from: 51,
                    to: 60,
                    move_type: GameMoveType::Promotion(PieceType::Queen, Some(PieceType::Pawn)),
                    piece_type: PieceType::Pawn,
                },
                true,
                &mut see_buffer
            ),
            0
        );
        assert_eq!(
            see(
                &GameState::from_fen("4pq2/3P4/2B5/8/8/8/8/k1K5 w - -"),
                &GameMove {
                    from: 51,
                    to: 60,
                    move_type: GameMoveType::Promotion(PieceType::Queen, Some(PieceType::Pawn)),
                    piece_type: PieceType::Pawn,
                },
                true,
                &mut see_buffer
            ),
            100
        );
    }
}
