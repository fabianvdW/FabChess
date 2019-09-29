pub mod alphabeta;
pub mod cache;
pub mod history;
pub mod quiescence;
pub mod reserved_memory;
pub mod searcher;
pub mod statistics;
pub mod timecontrol;

use crate::board_representation::game_state::*;
use crate::board_representation::game_state_attack_container::GameStateAttackContainer;
use crate::move_generation::movegen;
use crate::move_generation::movegen::MoveList;
use history::History;
use searcher::Search;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub const MAX_SEARCH_DEPTH: usize = 100;
pub const MATE_SCORE: i16 = 15000;
pub const MATED_IN_MAX: i16 = -14000;
pub const STANDARD_SCORE: i16 = -32767;

pub fn init_constants() {
    quiescence::PIECE_VALUES.len();
}

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

#[derive(Clone)]
pub struct GradedMove {
    pub mv_index: usize,
    pub score: f64,
}

impl GradedMove {
    pub fn new(mv_index: usize, score: f64) -> GradedMove {
        GradedMove { mv_index, score }
    }
}

impl Eq for GradedMove {}

impl PartialEq for GradedMove {
    fn eq(&self, other: &GradedMove) -> bool {
        self.score == other.score
    }
}

impl Ord for GradedMove {
    fn cmp(&self, other: &GradedMove) -> Ordering {
        if self.score > other.score {
            return Ordering::Less;
        } else if self.score < other.score {
            return Ordering::Greater;
        }
        Ordering::Equal
    }
}

impl PartialOrd for GradedMove {
    fn partial_cmp(&self, other: &GradedMove) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

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
    if game_state.pieces[PAWN][WHITE]
        | game_state.pieces[ROOK][WHITE]
        | game_state.pieces[QUEEN][WHITE]
        | game_state.pieces[PAWN][BLACK]
        | game_state.pieces[ROOK][BLACK]
        | game_state.pieces[QUEEN][BLACK]
        == 0u64
        && (game_state.pieces[KNIGHT][WHITE] | game_state.pieces[BISHOP][WHITE]).count_ones() <= 1
        && (game_state.pieces[KNIGHT][BLACK] | game_state.pieces[BISHOP][BLACK]).count_ones() <= 1
    {
        return SearchInstruction::StopSearching(0);
    }

    if game_state.half_moves >= 100 {
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
        if game_state.color_to_move == WHITE {
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
pub fn clear_pv(at_depth: usize, search: &mut Search) {
    let mut index = 0;
    while let Some(_) = search.pv_table[at_depth].pv[index].as_ref() {
        search.pv_table[at_depth].pv[index] = None;
        index += 1;
    }
}

#[inline(always)]
pub fn concatenate_pv(at_depth: usize, search: &mut Search) {
    let mut index = 0;
    while let Some(mv) = search.pv_table[at_depth + 1].pv[index].as_ref() {
        search.pv_table[at_depth].pv[index + 1] = Some(*mv);
        index += 1;
    }
    while let Some(_) = search.pv_table[at_depth].pv[index + 1].as_ref() {
        search.pv_table[at_depth].pv[index + 1] = None;
        index += 1;
    }
}

#[inline(always)]
pub fn in_check(game_state: &GameState, attack_container: &GameStateAttackContainer) -> bool {
    (game_state.pieces[KING][game_state.color_to_move]
        & attack_container.attacks_sum[1 - game_state.color_to_move])
        != 0u64
}

#[inline(always)]
pub fn in_check_slow(game_state: &GameState) -> bool {
    movegen::get_checkers(game_state, true).count_ones() > 0
}

#[inline(always)]
pub fn checkup(search: &mut Search, stop: &Arc<AtomicBool>) {
    search.search_statistics.refresh_time_elapsed();
    if search.tc.time_over(
        search.search_statistics.time_elapsed,
        &search.tc_information,
    ) || stop.load(std::sync::atomic::Ordering::Relaxed)
    {
        search.stop = true;
        //println!("{}", search.search_statistics);
    }
}

#[inline(always)]
pub fn get_next_gm(mv_list: &mut MoveList, mv_index: usize, max_moves: usize) -> (usize, f64) {
    if mv_list.counter == 0 {
        panic!("List has to be longer than 1")
    } else {
        let mut index = mv_index;
        for i in (mv_index + 1)..max_moves {
            if mv_list.graded_moves[i].as_ref().unwrap().score
                > mv_list.graded_moves[index].as_ref().unwrap().score
            {
                index = i;
            }
        }
        let result = mv_list.graded_moves[index].as_ref().unwrap().mv_index;
        let score = mv_list.graded_moves[index].as_ref().unwrap().score;
        mv_list.graded_moves[index] = mv_list.graded_moves[mv_index].clone();
        (result, score)
    }
}

#[inline(always)]
pub fn find_move(mv: &GameMove, mv_list: &MoveList, contains: bool) -> usize {
    let mut mv_index = 0;
    while mv_index < mv_list.counter {
        let mvs = mv_list.move_list[mv_index].as_ref().unwrap();
        if mvs.from == mv.from && mvs.to == mv.to && mvs.move_type == mv.move_type {
            break;
        }
        mv_index += 1;
    }
    if mv_index < mv_list.counter {
        mv_index
    } else if contains {
        panic!("Type 2 error");
    } else {
        mv_index
    }
}
