use super::super::board_representation::game_state::{
    GameMove, GameMoveType, GameState, PieceType, BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK,
    WHITE,
};
use super::super::evaluation::eval_game_state;
use super::super::move_generation::movegen;
use super::alphabeta::*;
use super::*;
use crate::bitboards::bitboards::constants::{KING_ATTACKS, KNIGHT_ATTACKS, RANKS};
use crate::move_generation::makemove::{make_move, unmake_move};
use crate::search::cache::CacheEntry;
use crate::search::moveordering::{MoveOrderer, QUIESCENCE_IN_CHECK_STAGES, QUIESCENCE_STAGES};

pub const DELTA_PRUNING: i16 = 100;
pub const PIECE_VALUES: [i16; 6] = [100, 400, 400, 650, 1100, 30000];

pub fn q_search(mut p: CombinedSearchParameters, thread: &mut Thread) -> i16 {
    //Step 0. Prepare variables
    thread.search_statistics.add_q_node(p.current_depth);
    clear_pv(p.current_depth, thread);
    let color = if p.game_state.color_to_move == WHITE {
        1
    } else {
        -1
    };
    //Step 1. Stop flag set, return immediatly
    if thread.self_stop {
        return STANDARD_SCORE;
    }

    //Step 2. Max search-depth reached
    if let SearchInstruction::StopSearching(res) = max_depth(&p) {
        return res;
    }

    //Step 3. Check for draw
    if let SearchInstruction::StopSearching(res) = check_for_draw(p.game_state, &thread.history) {
        return res;
    }

    //Step 4. Attacks and in check  flag. Attacks are only recalculated when parent is also a qnode
    let incheck = p.game_state.in_check();

    //Step 5. Get standing pat when not in check
    let stand_pat = if !incheck {
        Some(eval_game_state(&p.game_state, p.alpha * color, p.beta * color).final_eval * color)
    } else {
        None
    };

    //Step 6. Preliminary pruning
    if !incheck {
        if let SearchInstruction::StopSearching(res) = adjust_standpat(&mut p, stand_pat.unwrap()) {
            return res;
        } else if let SearchInstruction::StopSearching(res) = delta_pruning(&p, stand_pat.unwrap())
        {
            return res;
        }
    }

    //Step 7. TT Lookup
    let mut tt_entry = None;
    if p.depth_left == 0 {
        if let SearchInstruction::StopSearching(res) =
            thread
                .itcs
                .cache()
                .lookup(&p, &mut tt_entry, thread.root_plies_played)
        {
            #[cfg(feature = "search-statistics")]
            {
                thread.search_statistics.add_cache_hit_aj_replace_ns();
            }
            return res;
        }
    }
    #[cfg(feature = "search-statistics")]
    {
        if tt_move.is_some() {
            thread.search_statistics.add_cache_hit_ns();
        }
    }
    let mut tt_move = if let Some(ce) = tt_entry {
        Some(CacheEntry::u16_to_mv(ce.mv, p.game_state))
    } else {
        None
    };
    //Only captures are valid tt moves (if not in check)
    if tt_move.is_some() && !incheck && !tt_move.as_ref().unwrap().is_capture() {
        tt_move = None;
    }

    thread
        .history
        .push(p.game_state.hash, p.game_state.irreversible.half_moves == 0);

    //Step 8. Iterate through moves

    let mut current_max_score = if incheck {
        STANDARD_SCORE
    } else {
        *stand_pat.as_ref().unwrap()
    };
    let mut has_pv = false;
    let mut move_orderer = MoveOrderer {
        stage: 0,
        stages: if incheck {
            &QUIESCENCE_IN_CHECK_STAGES
        } else {
            &QUIESCENCE_STAGES
        },
    };

    loop {
        let mv = move_orderer.next(thread, &p, None, tt_move);
        if mv.is_none() {
            break;
        }
        let (capture_move, _) = mv.unwrap();
        if !incheck
            && !passes_delta_pruning(
                capture_move,
                p.game_state.phase.phase,
                *stand_pat.as_ref().unwrap(),
                p.alpha,
            )
        {
            continue;
        }
        debug_assert!(incheck || capture_move.is_capture());
        let irreversible = make_move(p.game_state, capture_move);
        //Step 8.4. Search move
        let score = -q_search(
            CombinedSearchParameters::from(
                -p.beta,
                -p.alpha,
                p.depth_left - 1,
                p.game_state,
                p.current_depth + 1,
            ),
            thread,
        );
        unmake_move(p.game_state, capture_move, irreversible);
        //Step 8.5 Move raises best moves score, so update pv and score
        if score > current_max_score {
            current_max_score = score;
            thread.pv_table[p.current_depth].pv[0] = Some(capture_move);
            has_pv = true;
            //Hang on following pv in theory
        }
        //Step 8.6 Beta cutoff, break
        if score >= p.beta {
            #[cfg(feature = "search-statistics")]
            {
                thread.search_statistics.add_q_beta_cutoff(index);
            }
            break;
        }

        //Step 8.7 Raise alpha if score > alpha
        if score > p.alpha {
            p.alpha = score;
        }
    }

    thread.history.pop();
    #[cfg(feature = "search-statistics")]
    {
        if current_max_score < p.beta {
            thread.search_statistics.add_q_beta_noncutoff();
        }
    }
    //Step 9. Evaluate leafs correctly
    if incheck {
        let game_status =
            check_end_condition(p.game_state, current_max_score > STANDARD_SCORE, incheck);
        if game_status != GameResult::Ingame {
            assert!(thread.pv_table[p.current_depth].pv[0].is_none() || thread.self_stop);
            clear_pv(p.current_depth, thread);
            return leaf_score(game_status, color, p.current_depth as i16);
        }
    }

    //This is not done anymore. Let's see if it still works.

    //Step 10. Make TT entry
    if has_pv && p.depth_left == 0 && !thread.self_stop {
        thread.itcs.cache().insert(
            &p,
            thread.pv_table[p.current_depth].pv[0].expect("Can't unwrap move for TT in qsearch!"),
            current_max_score,
            p.alpha,
            thread.root_plies_played,
            if incheck {
                None
            } else {
                Some(*stand_pat.as_ref().unwrap() * color)
            },
        );
    }

    //Step 11. Return
    current_max_score
}

#[inline(always)]
pub fn adjust_standpat(p: &mut CombinedSearchParameters, stand_pat: i16) -> SearchInstruction {
    if stand_pat >= p.beta {
        return SearchInstruction::StopSearching(stand_pat);
    }
    if stand_pat > p.alpha {
        p.alpha = stand_pat;
    }
    SearchInstruction::ContinueSearching
}

#[inline(always)]
pub fn delta_pruning(p: &CombinedSearchParameters, stand_pat: i16) -> SearchInstruction {
    let diff = p.alpha - stand_pat - DELTA_PRUNING;
    if diff > 0 && best_move_value(p.game_state) < diff {
        SearchInstruction::StopSearching(stand_pat)
    } else {
        SearchInstruction::ContinueSearching
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
        & RANKS[if state.color_to_move == WHITE { 6 } else { 1 }])
        != 0u64
    {
        res += PIECE_VALUES[QUEEN] - PIECE_VALUES[PAWN];
    }
    res
}

#[inline(always)]
pub fn passes_delta_pruning(capture_move: GameMove, phase: f64, eval: i16, alpha: i16) -> bool {
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
    eval + captured_piece.to_piece_score().interpolate(phase) + DELTA_PRUNING >= alpha
}

#[inline(always)]
pub fn see(game_state: &GameState, mv: GameMove, exact: bool, gain: &mut Vec<i16>) -> i16 {
    let may_xray = game_state.pieces[PAWN][WHITE]
        | game_state.pieces[PAWN][BLACK]
        | game_state.pieces[BISHOP][WHITE]
        | game_state.pieces[BISHOP][BLACK]
        | game_state.pieces[ROOK][WHITE]
        | game_state.pieces[ROOK][BLACK]
        | game_state.pieces[QUEEN][WHITE]
        | game_state.pieces[QUEEN][BLACK];
    let mut from_set = 1u64 << mv.from;
    let mut occ = game_state.all_pieces();
    let mut attadef = attacks_to(&game_state, mv.to as usize, occ);
    gain[0] = capture_value(mv);
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
            attadef |= recalculate_sliders(&game_state, color_to_move, mv.to as usize, occ)
                & (!deleted_pieces);
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
    movegen::bishop_attacks(square, occ)
        & (game_state.pieces[BISHOP][color_to_move] | game_state.pieces[QUEEN][color_to_move])
        | movegen::rook_attacks(square, occ)
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
    attacks |= KNIGHT_ATTACKS[square] & knights
        | movegen::bishop_attacks(square, occ) & bishops
        | movegen::rook_attacks(square, occ) & rooks;
    attacks |= (movegen::w_pawn_west_targets(square_board)
        | movegen::w_pawn_east_targets(square_board))
        & game_state.pieces[PAWN][BLACK];
    attacks |= (movegen::b_pawn_west_targets(square_board)
        | movegen::b_pawn_east_targets(square_board))
        & game_state.pieces[PAWN][WHITE];
    attacks |=
        KING_ATTACKS[square] & (game_state.pieces[KING][WHITE] | game_state.pieces[KING][BLACK]);
    attacks
}

#[inline(always)]
pub fn capture_value(mv: GameMove) -> i16 {
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
    PIECE_VALUES[piece_type.to_index()]
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
                GameMove {
                    from: 4,
                    to: 36,
                    move_type: GameMoveType::Capture(PieceType::Pawn),
                    piece_type: PieceType::Rook,
                },
                true,
                &mut see_buffer,
            ),
            100
        );
        assert_eq!(
            see(
                &GameState::from_fen("1k2r3/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -"),
                GameMove {
                    from: 4,
                    to: 36,
                    move_type: GameMoveType::Capture(PieceType::Pawn),
                    piece_type: PieceType::Rook,
                },
                true,
                &mut see_buffer,
            ),
            -550
        );
        assert_eq!(
            see(
                &GameState::from_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -"),
                GameMove {
                    from: 19,
                    to: 36,
                    move_type: GameMoveType::Capture(PieceType::Pawn),
                    piece_type: PieceType::Knight,
                },
                true,
                &mut see_buffer,
            ),
            -300
        );
        assert_eq!(
            see(
                &GameState::from_fen("1k1r3q/1ppn3p/p4b2/4n3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -"),
                GameMove {
                    from: 19,
                    to: 36,
                    move_type: GameMoveType::Capture(PieceType::Knight),
                    piece_type: PieceType::Knight,
                },
                true,
                &mut see_buffer,
            ),
            0
        );
        assert_eq!(
            see(
                &GameState::from_fen("1k1r2q1/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -"),
                GameMove {
                    from: 19,
                    to: 36,
                    move_type: GameMoveType::Capture(PieceType::Pawn),
                    piece_type: PieceType::Knight,
                },
                true,
                &mut see_buffer,
            ),
            -150
        );
        assert_eq!(
            see(
                &GameState::from_fen("8/8/3p4/4r3/2RKP3/5k2/8/8 b - -"),
                GameMove {
                    from: 36,
                    to: 28,
                    move_type: GameMoveType::Capture(PieceType::Pawn),
                    piece_type: PieceType::Rook,
                },
                true,
                &mut see_buffer,
            ),
            100
        );
        assert_eq!(
            see(
                &GameState::from_fen("k7/8/5q2/8/3r4/2KQ4/8/8 w - -"),
                GameMove {
                    from: 19,
                    to: 27,
                    move_type: GameMoveType::Capture(PieceType::Rook),
                    piece_type: PieceType::Queen,
                },
                true,
                &mut see_buffer,
            ),
            650
        );
        assert_eq!(
            see(
                &GameState::from_fen("8/8/5q2/2k5/3r4/2KQ4/8/8 w - -"),
                GameMove {
                    from: 19,
                    to: 27,
                    move_type: GameMoveType::Capture(PieceType::Rook),
                    piece_type: PieceType::Queen,
                },
                true,
                &mut see_buffer,
            ),
            -450
        );
        assert_eq!(
            see(
                &GameState::from_fen("4pq2/3P4/8/8/8/8/8/k1K5 w - -"),
                GameMove {
                    from: 51,
                    to: 60,
                    move_type: GameMoveType::Promotion(PieceType::Queen, Some(PieceType::Pawn)),
                    piece_type: PieceType::Pawn,
                },
                true,
                &mut see_buffer,
            ),
            0
        );
        assert_eq!(
            see(
                &GameState::from_fen("4pq2/3P4/2B5/8/8/8/8/k1K5 w - -"),
                GameMove {
                    from: 51,
                    to: 60,
                    move_type: GameMoveType::Promotion(PieceType::Queen, Some(PieceType::Pawn)),
                    piece_type: PieceType::Pawn,
                },
                true,
                &mut see_buffer,
            ),
            100
        );
    }
}
