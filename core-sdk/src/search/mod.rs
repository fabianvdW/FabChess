pub mod alphabeta;
pub mod cache;
pub mod history;
pub mod moveordering;
pub mod quiescence;
pub mod reserved_memory;
pub mod searcher;
pub mod statistics;
pub mod timecontrol;

use crate::board_representation::game_state::*;
use crate::search::searcher::Thread;
use crate::search::timecontrol::TimeControlInformation;
use history::History;
use std::fmt::{Display, Formatter, Result};

pub const MAX_SEARCH_DEPTH: usize = 100;
pub const MATE_SCORE: i16 = 15000;
pub const MATED_IN_MAX: i16 = -14000;
pub const STANDARD_SCORE: i16 = -32767;

pub struct CombinedSearchParameters<'a> {
    pub alpha: i16,
    pub beta: i16,
    pub depth_left: i16,
    pub game_state: &'a GameState,
    pub color: i16,
    pub current_depth: usize,
}
impl<'a> CombinedSearchParameters<'a> {
    pub fn from(
        alpha: i16,
        beta: i16,
        depth_left: i16,
        game_state: &'a GameState,
        color: i16,
        current_depth: usize,
    ) -> Self {
        CombinedSearchParameters {
            alpha,
            beta,
            depth_left,
            game_state,
            color,
            current_depth,
        }
    }
}
pub enum SearchInstruction {
    SkipMove,
    ContinueSearching,
    StopSearching(i16),
}

#[derive(Clone)]
pub struct ScoredPrincipalVariation {
    pub score: i16,
    pub pv: PrincipalVariation,
    pub depth: usize,
}
impl Default for ScoredPrincipalVariation {
    fn default() -> Self {
        ScoredPrincipalVariation {
            score: 0,
            pv: PrincipalVariation::new(0),
            depth: 0,
        }
    }
}
#[derive(Clone)]
pub struct PrincipalVariation {
    pub pv: Vec<Option<GameMove>>,
}

impl PrincipalVariation {
    pub fn new(depth_left: usize) -> PrincipalVariation {
        PrincipalVariation {
            pv: vec![None; depth_left + 1],
        }
    }
}

impl Display for PrincipalVariation {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        let mut index = 0;
        while let Some(mv) = self.pv[index].as_ref() {
            res_str.push_str(&format!("{:?} ", mv));
            index += 1;
        }
        write!(formatter, "{}", res_str)
    }
}

#[derive(Copy, Clone)]
pub struct GradedMove(pub GameMove, pub Option<f64>);

#[inline(always)]
pub fn leaf_score(game_status: GameResult, color: i16, current_depth: i16) -> i16 {
    if game_status == GameResult::Draw {
        return 0;
    } else if game_status == GameResult::WhiteWin {
        return (MATE_SCORE - current_depth) * color;
    } else if game_status == GameResult::BlackWin {
        return (MATE_SCORE - current_depth) * -color;
    }
    panic!("Invalid Leaf");
}

//Doesn't actually check for stalemate
#[inline(always)]
pub fn check_for_draw(game_state: &GameState, history: &History) -> SearchInstruction {
    if game_state.get_piece_bb(PieceType::Pawn)
        | game_state.get_piece_bb(PieceType::Rook)
        | game_state.get_piece_bb(PieceType::Queen)
        == 0u64
        && (game_state.get_piece(PieceType::Knight, WHITE)
            | game_state.get_piece(PieceType::Bishop, WHITE))
        .count_ones()
            <= 1
        && (game_state.get_piece(PieceType::Knight, BLACK)
            | game_state.get_piece(PieceType::Bishop, BLACK))
        .count_ones()
            <= 1
    {
        return SearchInstruction::StopSearching(0);
    }

    if game_state.get_half_moves() >= 100 {
        return SearchInstruction::StopSearching(0);
    }

    if history.get_occurences(game_state) >= 1 {
        return SearchInstruction::StopSearching(0);
    }
    SearchInstruction::ContinueSearching
}

#[inline(always)]
pub fn check_end_condition(
    game_state: &GameState,
    has_legal_moves: bool,
    in_check: bool,
) -> GameResult {
    if in_check && !has_legal_moves {
        if game_state.get_color_to_move() == WHITE {
            return GameResult::BlackWin;
        } else {
            return GameResult::WhiteWin;
        }
    }
    if !in_check && !has_legal_moves {
        return GameResult::Draw;
    }
    GameResult::Ingame
}

#[inline(always)]
pub fn clear_pv(at_depth: usize, thread: &mut Thread) {
    let mut index = 0;
    while let Some(_) = thread.pv_table[at_depth].pv[index].as_ref() {
        thread.pv_table[at_depth].pv[index] = None;
        index += 1;
    }
}

#[inline(always)]
pub fn concatenate_pv(at_depth: usize, thread: &mut Thread) {
    let mut index = 0;
    while let Some(mv) = thread.pv_table[at_depth + 1].pv[index].as_ref() {
        thread.pv_table[at_depth].pv[index + 1] = Some(*mv);
        index += 1;
    }
    while let Some(_) = thread.pv_table[at_depth].pv[index + 1].as_ref() {
        thread.pv_table[at_depth].pv[index + 1] = None;
        index += 1;
    }
}

#[inline(always)]
pub fn checkup(thread: &mut Thread) {
    if (thread.id == 0
        && thread.tc.time_over(
            thread.itcs.get_time_elapsed(),
            &TimeControlInformation {
                high_score_diff: false,
                time_saved: thread.time_saved,
                stable_pv: thread
                    .itcs
                    .stable_pv
                    .load(std::sync::atomic::Ordering::Relaxed),
            },
            thread.itcs.uci_options().move_overhead,
        ))
        || *thread
            .itcs
            .timeout_flag
            .read()
            .expect("Reading posioned timeoutflag")
    {
        if thread.id == 0 {
            *thread
                .itcs
                .timeout_flag
                .write()
                .expect("Writing poisoned timeoutflag") = true;
        }
        thread.self_stop = true;
    }
}
