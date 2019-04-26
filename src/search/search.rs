use super::super::GameState;
use super::alphabeta::principal_variation_search;
use super::alphabeta::PrincipalVariation;
use super::cache::{Cache, CacheEntry};
use super::statistics::SearchStatistics;
use super::GameMove;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::RwLock;

pub struct Search {
    pub principal_variation: [Option<CacheEntry>; 100],
    pub killer_moves: [[Option<GameMove>; 2]; 100],
    pub hh_score: [[usize; 64]; 64],
    pub bf_score: [[usize; 64]; 64],
    pub search_statistics: SearchStatistics,
    pub tc: TimeControl,
    pub stop: bool,
}

pub struct TimeControl {
    pub mytime: u64,
    pub myinc: u64,
}

impl TimeControl {
    pub fn time_over(&self, time_spent: u64) -> bool {
        return time_spent > self.mytime - 40
            || time_spent > (self.mytime as f64 / 30.0) as u64 + self.myinc - 20;
    }
}

impl Search {
    pub fn new(tc: TimeControl) -> Search {
        Search {
            principal_variation: [None; 100],
            search_statistics: SearchStatistics::new(),
            killer_moves: [[None; 2]; 100],
            hh_score: [[1; 64]; 64],
            bf_score: [[1; 64]; 64],
            tc,
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
    ) -> PrincipalVariation {
        let root_plies_played = (game_state.full_moves - 1) * 2 + game_state.color_to_move;
        let cache = &mut (*cache_uc).write().unwrap();
        let mut hist: Vec<u64> = Vec::with_capacity(history.len());
        for gs in history.iter().rev() {
            hist.push(gs.hash);
            if gs.half_moves == 0 {
                break;
            }
        }
        self.search_statistics = SearchStatistics::new();

        let mut best_pv = PrincipalVariation::new(0);
        for d in 1..(depth + 1) {
            let mut pv;
            if d == 1 {
                pv = principal_variation_search(
                    -30000,
                    30000,
                    d,
                    &game_state,
                    if game_state.color_to_move == 0 { 1 } else { -1 },
                    0,
                    self,
                    root_plies_played,
                    &mut hist,
                    &stop_ref,
                    cache,
                );
            } else {
                //Aspiration Window
                //Start with half window of last time
                let mut delta = 20;
                let mut alpha = best_pv.score - delta;
                let mut beta = best_pv.score + delta;
                loop {
                    pv = principal_variation_search(
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
                    );
                    if self.stop {
                        break;
                    }
                    if pv.score > alpha && pv.score < beta && pv.pv.len() > 0 {
                        break;
                    }
                    if pv.score <= alpha {
                        if alpha < -10000 {
                            alpha = -32000;
                        } else {
                            alpha -= delta;
                        }
                    }
                    if pv.score >= beta {
                        if beta > 10000 {
                            beta = 32000;
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
            for mv in &pv.pv {
                pv_str.push_str(&format!("{:?} ", mv));
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
                    pv.score,
                    pv_str
                )
            );
            //Set PV in table
            let mut pv_stack = Vec::with_capacity(pv.pv.len());
            for (i, pair) in pv.pv.iter().enumerate() {
                if i == 0 {
                    pv_stack.push(crate::move_generation::movegen::make_move(
                        &game_state,
                        &pair,
                    ));
                } else {
                    pv_stack.push(crate::move_generation::movegen::make_move(
                        &pv_stack[i - 1],
                        &pair,
                    ))
                }
            }
            for (i, pair) in pv.pv.iter().enumerate() {
                let state = if i == 0 {
                    &game_state
                } else {
                    &pv_stack[i - 1]
                };
                self.principal_variation[i] = Some(CacheEntry::new(
                    state,
                    d - i as i16,
                    pv.score,
                    false,
                    false,
                    &pair,
                ));
            }
            best_pv = pv;
        }
        self.search_statistics.refresh_time_elapsed();
        return best_pv;
    }
}
