use super::super::GameState;
use super::alphabeta::principal_variation_search;
use super::alphabeta::PrincipalVariation;
use super::alphabeta::MAX_SEARCH_DEPTH;
use super::alphabeta::STANDARD_SCORE;
use super::cache::{Cache, CacheEntry};
use super::history::History;
use super::statistics::SearchStatistics;
use super::timecontrol::{TimeControl, TimeControlInformation};
use super::GameMove;
use crate::move_generation::makemove::make_move;
use crate::move_generation::movegen;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::RwLock;

pub struct Search {
    pub principal_variation: [Option<CacheEntry>; MAX_SEARCH_DEPTH],
    pub pv_table: Vec<PrincipalVariation>,
    pub killer_moves: [[Option<GameMove>; 2]; MAX_SEARCH_DEPTH],
    pub hh_score: [[usize; 64]; 64],
    pub bf_score: [[usize; 64]; 64],
    pub see_buffer: Vec<i16>,
    pub search_statistics: SearchStatistics,
    pub tc: TimeControl,
    pub tc_information: TimeControlInformation,
    pub stop: bool,
}

impl Search {
    pub fn new(tc: TimeControl, tc_information: TimeControlInformation) -> Search {
        let mut pv_table = Vec::with_capacity(MAX_SEARCH_DEPTH);
        for i in 0..MAX_SEARCH_DEPTH {
            pv_table.push(PrincipalVariation::new(MAX_SEARCH_DEPTH - i));
        }
        Search {
            principal_variation: [None; MAX_SEARCH_DEPTH],
            pv_table,
            search_statistics: SearchStatistics::new(),
            killer_moves: [[None; 2]; MAX_SEARCH_DEPTH],
            hh_score: [[1; 64]; 64],
            bf_score: [[1; 64]; 64],
            see_buffer: vec![0i16; MAX_SEARCH_DEPTH],
            tc,
            tc_information,
            stop: false,
        }
    }

    pub fn search(
        &mut self,
        depth: i16,
        game_state: GameState,
        history: Vec<GameState>,
        stop_ref: Arc<AtomicBool>,
        cache_uc: Arc<RwLock<Cache>>,
        saved_time: Arc<AtomicU64>,
    ) -> i16 {
        let root_plies_played = (game_state.full_moves - 1) * 2 + game_state.color_to_move;
        let cache = &mut (*cache_uc).write().unwrap();
        let mut hist: History = History::new();
        let mut relevant_hashes: Vec<u64> = Vec::with_capacity(100);
        for gs in history.iter().rev() {
            relevant_hashes.push(gs.hash);
            if gs.half_moves == 0 {
                break;
            }
        }
        for hashes in relevant_hashes.iter().rev() {
            hist.push(*hashes, false);
        }

        self.search_statistics = SearchStatistics::new();
        let mut move_list = movegen::MoveList::new();
        let mut best_pv_score = STANDARD_SCORE;
        for d in 1..(depth + 1) {
            let mut pv_score;
            if d == 1 {
                pv_score = principal_variation_search(
                    -16000,
                    16000,
                    d,
                    &game_state,
                    if game_state.color_to_move == 0 { 1 } else { -1 },
                    0,
                    self,
                    root_plies_played,
                    &mut hist,
                    &stop_ref,
                    cache,
                    &mut move_list,
                );
            } else {
                //Aspiration Window
                let mut delta = 20;
                let mut alpha = best_pv_score - delta;
                let mut beta = best_pv_score + delta;
                loop {
                    pv_score = principal_variation_search(
                        alpha,
                        beta,
                        d,
                        &game_state,
                        if game_state.color_to_move == 0 { 1 } else { -1 },
                        0,
                        self,
                        root_plies_played,
                        &mut hist,
                        &stop_ref,
                        cache,
                        &mut move_list,
                    );
                    if self.stop {
                        break;
                    }
                    if pv_score > alpha && pv_score < beta {
                        break;
                    }
                    if pv_score <= alpha {
                        if alpha < -10000 {
                            alpha = -16000;
                        } else {
                            alpha -= delta;
                        }
                    }
                    if pv_score >= beta {
                        if beta > 10000 {
                            beta = 16000;
                        } else {
                            beta += delta;
                        }
                    }
                    delta = (delta as f64 * 1.5) as i16;
                }
            }
            if self.stop {
                break;
            }
            //println!("{}", format!("Depth {} with nodes {} and pv: {}", d, stats.nodes_searched, pv));
            let mut pv_str = String::new();
            let mut index = 0;
            while let Some(mv) = self.pv_table[0].pv[index].as_ref() {
                pv_str.push_str(&format!("{:?} ", mv));
                index += 1;
            }
            let nps = self.search_statistics.getnps();
            println!(
                "{}",
                format!(
                    "info depth {} seldepth {} nodes {} nps {} time {} score cp {} multipv 1 pv {}",
                    d,
                    self.search_statistics.seldepth,
                    self.search_statistics.nodes_searched,
                    nps,
                    self.search_statistics.time_elapsed,
                    pv_score,
                    pv_str
                )
            );
            //Compare old pv to new pv
            if let Some(ce) = self.principal_variation[0].as_ref() {
                let old_mv: GameMove = CacheEntry::u16_to_mv(ce.mv, &game_state);
                let new_mv: &GameMove = self.pv_table[0].pv[0]
                    .as_ref()
                    .expect("Couldn't unwrap first move of new pv");
                if old_mv == (*new_mv) {
                    self.tc_information.stable_pv = true;
                } else {
                    self.tc_information.stable_pv = false;
                }
            }
            //println!("{}", self.search_statistics);
            //Set PV in table
            let mut pv_stack = Vec::with_capacity(d as usize);
            let mut index = 0;
            while let Some(pair) = self.pv_table[0].pv[index].as_ref() {
                if index == 0 {
                    pv_stack.push(make_move(&game_state, &pair));
                } else {
                    pv_stack.push(make_move(&pv_stack[index - 1], &pair));
                }
                index += 1;
            }
            index = 0;
            while let Some(pair) = self.pv_table[0].pv[index].as_ref() {
                let state = if index == 0 {
                    &game_state
                } else {
                    &pv_stack[index - 1]
                };
                self.principal_variation[index] = Some(CacheEntry::new(
                    state,
                    d - index as i16,
                    pv_score,
                    false,
                    false,
                    &pair,
                    None,
                ));
                index += 1;
            }
            best_pv_score = pv_score;
        }
        self.search_statistics.refresh_time_elapsed();
        //println!("{}", self.tc);
        //println!("Time elapsed: {}", self.search_statistics.time_elapsed);
        //println!(
        //    "Time saved in this: {}",
        //        self.tc.time_saved(self.search_statistics.time_elapsed)
        //);
        let mut new_timesaved: i64 = self.tc_information.time_saved as i64
            + self.tc.time_saved(self.search_statistics.time_elapsed);
        new_timesaved = new_timesaved.max(0);
        saved_time.store(new_timesaved as u64, Ordering::Relaxed);
        //println!(
        //    "New total time saved: {}",
        //    saved_time.load(Ordering::Relaxed)
        //);
        return best_pv_score;
    }
}
