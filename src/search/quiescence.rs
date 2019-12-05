use super::super::board_representation::game_state::{
    GameMove, GameMoveType, GameResult, GameState, PieceType, BISHOP, BLACK, KING, KNIGHT, PAWN,
    QUEEN, ROOK, WHITE,
};
use super::super::evaluation::eval_game_state;
use super::super::move_generation::movegen;
use super::super::move_generation::movegen::AdditionalGameStateInformation;
use super::alphabeta::*;
use super::*;
use crate::bitboards;
use crate::move_generation::makemove::make_move;

pub const DELTA_PRUNING: i16 = 100;
lazy_static! {
    pub static ref PIECE_VALUES: [i16; 6] = [100, 400, 400, 650, 1100, 30000];
}

pub fn q_search(mut p: CombinedSearchParameters, thread: &mut Thread) -> i16 {
    //Step 0. Prepare variables
    thread.search_statistics.add_q_node(p.current_depth);
    clear_pv(p.current_depth, thread);

    //Step 1. Stop flag set, return immediatly
    if thread.self_stop {
        return STANDARD_SCORE;
    }

    //Step 2. Max search-depth reached
    if let SearchInstruction::StopSearching(res) = max_depth(&p, thread) {
        return res;
    }

    //Step 3. Check for draw
    if let SearchInstruction::StopSearching(res) = check_for_draw(p.game_state, &thread.history) {
        return res;
    }

    //Step 4. Attacks and in check  flag. Attacks are only recalculated when parent is also a qnode
    if p.depth_left < 0 {
        // Before dropping into qsearch we make sure we're not in check in pvs
        thread.attack_container.attack_containers[p.current_depth].write_state(p.game_state);
    }
    let incheck = in_check(
        p.game_state,
        &thread.attack_container.attack_containers[p.current_depth],
    );

    //Step 5. Get standing pat when not in check
    let stand_pat = if !incheck {
        Some(
            eval_game_state(
                &p.game_state,
                &thread.attack_container.attack_containers[p.current_depth],
                p.alpha * p.color,
                p.beta * p.color,
            )
            .final_eval
                * p.color,
        )
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
    let mut tt_move: Option<GameMove> = None;
    if let SearchInstruction::StopSearching(res) =
        thread
            .itcs
            .cache
            .lookup(&p, &mut None, &mut tt_move, thread.root_plies_played)
    {
        thread.search_statistics.add_cache_hit_aj_replace_ns();
        thread.pv_table[p.current_depth].pv[0] = tt_move;
        return res;
    }
    if tt_move.is_some() {
        thread.search_statistics.add_cache_hit_ns();
    }
    //Only captures are valid tt moves (if not in check)
    if tt_move.is_some() && !incheck && !tt_move.as_ref().unwrap().is_capture() {
        tt_move = None;
    }

    thread
        .history
        .push(p.game_state.hash, p.game_state.half_moves == 0);

    //Step 8. Iterate through moves
    let hash_move_counter = if tt_move.is_some() { 1 } else { 0 };
    let mut has_legal_move = false;
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
        //Step 8.1. Staged movegen. Generate all moves after trying tt move
        if index == hash_move_counter && !has_generated_moves {
            has_generated_moves = true;
            let (agsi, mvs) = make_and_evaluate_moves_qsearch(thread, &p, stand_pat, incheck);
            has_legal_move = agsi.stm_haslegalmove;
            available_captures_in_movelist = mvs;
            continue;
        }
        //Step 8.2. Select the next move
        let capture_move: GameMove = select_next_move_qsearch(
            &p,
            thread,
            index,
            &tt_move,
            moves_from_movelist_tried,
            available_captures_in_movelist,
        );
        debug_assert!(incheck || capture_move.is_capture());
        //Step 8.3. If the move is from the movelist, make sure we haven't searched it already as tt move
        if index >= hash_move_counter {
            moves_from_movelist_tried += 1;
            if let SearchInstruction::SkipMove = is_duplicate(&capture_move, &None, &tt_move) {
                index += 1;
                continue;
            }
        }

        let next_g = make_move(p.game_state, &capture_move);
        //Step 8.4. Search move
        let score = -q_search(
            CombinedSearchParameters::from(
                -p.beta,
                -p.alpha,
                p.depth_left - 1,
                &next_g,
                -p.color,
                p.current_depth + 1,
            ),
            thread,
        );

        //Step 8.5 Move raises best moves score, so update pv and score
        if score > current_max_score {
            current_max_score = score;
            thread.pv_table[p.current_depth].pv[0] = Some(capture_move);
            has_pv = true;
            //Hang on following pv in theory
        }
        //Step 8.6 Beta cutoff, break
        if score >= p.beta {
            thread.search_statistics.add_q_beta_cutoff(index);
            break;
        }

        //Step 8.7 Raise alpha if score > alpha
        if score > p.alpha {
            p.alpha = score;
        }
        index += 1;
    }

    thread.history.pop();
    if current_max_score < p.beta && index > 0 {
        thread.search_statistics.add_q_beta_noncutoff();
    }
    //Step 9. Evaluate leafs correctly
    let game_status = check_end_condition(p.game_state, has_legal_move, incheck);
    if game_status != GameResult::Ingame {
        clear_pv(p.current_depth, thread);
        return leaf_score(game_status, p.color, p.current_depth as i16);
    }

    //Step 10. Make TT entry
    if has_pv && p.depth_left == 0 && !thread.self_stop {
        thread.itcs.cache.insert(
            &p,
            &thread.pv_table[p.current_depth].pv[0].expect("Can't unwrap move for TT in qsearch!"),
            current_max_score,
            p.alpha,
            thread.root_plies_played,
            if incheck {
                None
            } else {
                Some(*stand_pat.as_ref().unwrap() * p.color)
            },
        );
    }

    //Step 11. Return
    current_max_score
}

#[inline(always)]
pub fn select_next_move_qsearch(
    p: &CombinedSearchParameters,
    thread: &mut Thread,
    index: usize,
    tt_move: &Option<GameMove>,
    moves_from_movelist_tried: usize,
    available_captures_in_movelist: usize,
) -> GameMove {
    if index == 0 && tt_move.is_some() {
        tt_move.expect("Couldn't unwrap tt move in qsearch")
    } else {
        let r = get_next_gm(
            &mut thread.movelist.move_lists[p.current_depth],
            moves_from_movelist_tried,
            available_captures_in_movelist,
        )
        .0;
        thread.movelist.move_lists[p.current_depth].move_list[r].expect("Could not get next gm")
    }
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
pub fn make_and_evaluate_moves_qsearch(
    thread: &mut Thread,
    p: &CombinedSearchParameters,
    stand_pat: Option<i16>,
    incheck: bool,
) -> (AdditionalGameStateInformation, usize) {
    let agsi = movegen::generate_moves(
        p.game_state,
        !incheck,
        &mut thread.movelist.move_lists[p.current_depth],
        &thread.attack_container.attack_containers[p.current_depth],
    );
    let move_list = &mut thread.movelist.move_lists[p.current_depth];
    let (mut mv_index, mut capture_index) = (0, 0);
    while mv_index < move_list.counter {
        let mv: &GameMove = move_list.move_list[mv_index].as_ref().unwrap();
        if let GameMoveType::EnPassant = mv.move_type {
            move_list.graded_moves[capture_index] = Some(GradedMove::new(mv_index, 99.0));
        } else {
            if !incheck
                && !passes_delta_pruning(
                    mv,
                    p.game_state.phase.phase,
                    *stand_pat.as_ref().unwrap(),
                    p.alpha,
                )
            {
                thread.search_statistics.add_q_delta_cutoff();
                mv_index += 1;
                continue;
            }
            if !incheck || mv.is_capture() {
                let score = see(p.game_state, mv, true, &mut thread.see_buffer);
                if score < 0 && !incheck {
                    thread.search_statistics.add_q_see_cutoff();
                    mv_index += 1;
                    continue;
                }
                move_list.graded_moves[capture_index] =
                    Some(GradedMove::new(mv_index, f64::from(score)));
            } else {
                if !incheck {
                    panic!("Not in check but also not capture");
                }
                let score = thread.hh_score[p.game_state.color_to_move][mv.from as usize]
                    [mv.to as usize] as f64
                    / thread.bf_score[p.game_state.color_to_move][mv.from as usize][mv.to as usize]
                        as f64
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
    eval + captured_piece.to_piece_score().interpolate(phase) + DELTA_PRUNING >= alpha
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
    let mut attadef = attacks_to(&game_state, mv.to as usize, occ);
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
                &GameMove {
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
                &GameMove {
                    from: 4,
                    to: 36,
                    move_type: GameMoveType::Capture(PieceType::Pawn),
                    piece_type: PieceType::Rook,
                },
                true,
                &mut see_buffer,
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
                &mut see_buffer,
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
                &mut see_buffer,
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
                &mut see_buffer,
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
                &mut see_buffer,
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
                &mut see_buffer,
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
                &mut see_buffer,
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
                &mut see_buffer,
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
                &mut see_buffer,
            ),
            100
        );
    }
}
