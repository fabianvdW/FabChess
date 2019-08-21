use super::alphabeta::principal_variation_search;
use super::alphabeta::PrincipalVariation;
use super::alphabeta::MATED_IN_MAX;
use super::alphabeta::MAX_SEARCH_DEPTH;
use super::alphabeta::STANDARD_SCORE;
use super::cache::{Cache, CacheEntry};
use super::history::History;
use super::statistics::SearchStatistics;
use super::timecontrol::{TimeControl, TimeControlInformation};
use super::GameMove;
use crate::board_representation::game_state::{GameState, WHITE};
//use crate::logging::log;
use crate::move_generation::makemove::make_move;
use crate::move_generation::movegen;
use crate::move_generation::movegen::MoveList;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::RwLock;

pub struct SearchUtils<'a> {
    pub root_pliesplayed: usize,
    pub search: &'a mut Search,
    pub history: &'a mut History,
    pub stop: &'a Arc<AtomicBool>,
    pub cache: &'a mut Cache,
    pub move_list: &'a mut MoveList,
}
impl<'a> SearchUtils<'a> {
    pub fn new(
        root_pliesplayed: usize,
        search: &'a mut Search,
        history: &'a mut History,
        stop: &'a Arc<AtomicBool>,
        cache: &'a mut Cache,
        move_list: &'a mut MoveList,
    ) -> Self {
        SearchUtils {
            root_pliesplayed,
            search,
            history,
            stop,
            cache,
            move_list,
        }
    }
}
pub struct Search {
    pub principal_variation: [Option<CacheEntry>; MAX_SEARCH_DEPTH],
    pub pv_table: Vec<PrincipalVariation>,
    pub killer_moves: [[Option<GameMove>; 2]; MAX_SEARCH_DEPTH],
    pub quiets_tried: [[Option<GameMove>; 128]; MAX_SEARCH_DEPTH],
    pub hh_score: [[[usize; 64]; 64]; 2],
    pub bf_score: [[[usize; 64]; 64]; 2],
    pub history_score: [[[isize; 64]; 64]; 2],
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
            search_statistics: SearchStatistics::default(),
            killer_moves: [[None; 2]; MAX_SEARCH_DEPTH],
            quiets_tried: [[None; 128]; MAX_SEARCH_DEPTH],
            hh_score: [[[0; 64]; 64]; 2],
            bf_score: [[[1; 64]; 64]; 2],
            history_score: [[[0; 64]; 64]; 2],
            see_buffer: vec![0i16; MAX_SEARCH_DEPTH],
            tc,
            tc_information,
            stop: false,
        }
    }

    pub fn replace_pv(&mut self, game_state: &GameState, depth: i16, pv_score: i16) {
        let mut pv_stack = Vec::with_capacity(depth as usize);
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
                depth - index as i16,
                pv_score,
                false,
                false,
                &pair,
                None,
            ));
            index += 1;
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
        _last_score: i16,
    ) -> i16 {
        let root_plies_played = (game_state.full_moves - 1) * 2 + game_state.color_to_move;
        let cache = &mut (*cache_uc).write().unwrap();
        let mut hist: History = History::default();
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

        self.search_statistics = SearchStatistics::default();
        let mut move_list = movegen::MoveList::default();
        let mut best_pv_score = STANDARD_SCORE;

        for d in 1..=depth {
            let mut pv_score;
            if d == 1 {
                let mut searchutils = SearchUtils::new(
                    root_plies_played,
                    self,
                    &mut hist,
                    &stop_ref,
                    cache,
                    &mut move_list,
                );
                pv_score = principal_variation_search(
                    -16000,
                    16000,
                    d,
                    &game_state,
                    if game_state.color_to_move == WHITE {
                        1
                    } else {
                        -1
                    },
                    0,
                    &mut searchutils,
                );
            } else {
                //Aspiration Window
                let mut delta = 40;
                let mut alpha = best_pv_score - delta;
                let mut beta = best_pv_score + delta;
                loop {
                    let mut searchutils = SearchUtils::new(
                        root_plies_played,
                        self,
                        &mut hist,
                        &stop_ref,
                        cache,
                        &mut move_list,
                    );
                    pv_score = principal_variation_search(
                        alpha,
                        beta,
                        d,
                        &game_state,
                        if game_state.color_to_move == WHITE {
                            1
                        } else {
                            -1
                        },
                        0,
                        &mut searchutils,
                    );
                    if self.stop {
                        break;
                    }
                    /*if (pv_score - last_score).abs() > 150 && last_score.abs() < 600 {
                        self.tc_information.high_score_diff = true;
                    } else {
                        self.tc_information.high_score_diff = false;
                    }*/

                    if pv_score > alpha && pv_score < beta {
                        break;
                    }
                    //Else put pv in principal_variation table
                    //self.replace_pv(&game_state, depth, pv_score);

                    if pv_score <= alpha {
                        if alpha < -10000 || pv_score < MATED_IN_MAX {
                            alpha = -16000;
                            beta = 16000;
                        } else {
                            alpha -= delta;
                        }
                    }
                    if pv_score >= beta {
                        if beta > 10000 || pv_score > -MATED_IN_MAX {
                            beta = 16000;
                            alpha = -16000;
                        } else {
                            beta += delta;
                        }
                    }
                    delta = (f64::from(delta) * 1.5) as i16;
                }
            }
            if self.stop {
                break;
            }
            //println!("{}", format!("Depth {} with nodes {} and pv: {}", d, stats.nodes_searched, pv));
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
                    self.pv_table[0]
                )
            );
            //println!("{}", self.search_statistics);

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
            self.replace_pv(&game_state, depth, pv_score);
            best_pv_score = pv_score;
        }
        self.search_statistics.refresh_time_elapsed();
        /*log(&format!(
            "\nFinished calculating game_state with plies: {}\n",
            game_state.full_moves
        ));
        log(&format!("{}\n", self.tc.to_string(&self.tc_information)));
        log(&format!(
            "Time elapsed: {}\n",
            self.search_statistics.time_elapsed
        ));
        log(&format!(
            "Time saved in this: {}\n",
            self.tc.time_saved(
                self.search_statistics.time_elapsed,
                self.tc_information.time_saved
            )
        ));*/
        let mut new_timesaved: i64 = self.tc_information.time_saved as i64
            + self.tc.time_saved(
                self.search_statistics.time_elapsed,
                self.tc_information.time_saved,
            );
        new_timesaved = new_timesaved.max(0);
        saved_time.store(new_timesaved as u64, Ordering::Relaxed);
        /*log(&format!(
            "New total time saved: {}\n",
            saved_time.load(Ordering::Relaxed)
        ));*/
        best_pv_score
    }
}
