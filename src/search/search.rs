use super::cache::{Cache, CacheEntry};
use super::statistics::SearchStatistics;
use super::super::GameState;
use super::GameMove;
use super::alphabeta::PrincipalVariation;
use super::alphabeta::principal_variation_search;

pub struct Search {
    pub cache: Cache,
    pub principal_variation: [Option<CacheEntry>; 100],
    pub killer_moves: [[Option<GameMove>; 2]; 100],
    pub hh_score: [[usize; 64]; 64],
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
        return time_spent > (self.mytime as f64 / 30.0) as u64 + self.myinc;
    }
}

impl Search {
    pub fn new(tc: TimeControl) -> Search {
        Search {
            cache: super::cache::Cache::new(),
            principal_variation: [None; 100],
            search_statistics: SearchStatistics::new(),
            killer_moves: [[None; 2]; 100],
            hh_score: [[0; 64]; 64],
            tc,
            stop: false,
        }
    }

    pub fn search(&mut self, depth: isize, game_state: GameState, history: Vec<GameState>) -> PrincipalVariation {
        self.make_cache_from_history(history);
        let mut stats = SearchStatistics::new();
        let mut best_pv = PrincipalVariation::new(0);
        for d in 1..depth + 1 {
            let mut pv = PrincipalVariation::new(1);
            if d == 1 {
                pv = principal_variation_search(-100000.0, 100000.0, d, &game_state, if game_state.color_to_move == 0 {
                    1
                } else {
                    -1
                }, &mut stats, 0, self);
            } else {
                //Aspiration Window
                //Start with half window of last time
                let mut delta = 0.2;
                let mut alpha: f64 = best_pv.score - delta;
                let mut beta: f64 = best_pv.score + delta;
                loop {
                    pv = principal_variation_search(alpha, beta, d, &game_state, if game_state.color_to_move == 0 {
                        1
                    } else {
                        -1
                    }, &mut stats, 0, self);
                    if self.stop {
                        break;
                    }
                    if pv.score > alpha && pv.score < beta {
                        break;
                    }
                    if pv.score <= alpha {
                        alpha -= delta;
                    }
                    if pv.score >= beta {
                        beta += delta;
                    }
                    delta *= 1.5;
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
            let nps = stats.getnps();
            let sc = (pv.score * 100.0) as isize;
            println!("{}", format!("info depth {} seldepth {} nodes {} nps {} time {} score cp {} multipv 1 pv {}", stats.depth, stats.seldepth, stats.nodes_searched, nps, stats.time_elapsed, sc, pv_str));
            //Set PV in table
            let mut pv_stack = Vec::with_capacity(pv.pv.len());
            for (i, pair) in pv.pv.iter().enumerate() {
                if i == 0 {
                    pv_stack.push(crate::move_generation::movegen::make_move(&game_state, &pair));
                } else {
                    pv_stack.push(crate::move_generation::movegen::make_move(&pv_stack[i - 1], &pair))
                }
            }
            for (i, pair) in pv.pv.iter().enumerate() {
                let state = if i == 0 { &game_state } else { &pv_stack[i - 1] };
                self.principal_variation[i] = Some(CacheEntry::new(state, d - i as isize, pv.score, false, false, &pair));
            }
            best_pv = pv;
        }
        self.search_statistics = stats;
        self.search_statistics.refresh_time_elapsed();
        return best_pv;
    }

    pub fn make_cache_from_history(&mut self, history: Vec<GameState>) {
        for gs in history {
            let index = gs.hash as usize & super::cache::CACHE_MASK;
            let ce = self.cache.cache[index];
            if let Some(entry) = ce {
                let mut occ_entry = entry;
                while occ_entry.occurences > 0 && occ_entry.hash != gs.hash {
                    let next_entry = self.cache.cache[(occ_entry.hash + 1) as usize & super::cache::CACHE_MASK];
                    occ_entry = match next_entry {
                        Some(s) => s,
                        _ => panic!("Can't be!")
                    };
                }
                let occ_index = occ_entry.hash as usize & super::cache::CACHE_MASK;
                if occ_entry.hash == gs.hash {
                    match &mut self.cache.cache[occ_index] {
                        Some(s) => {
                            s.occurences += 1;
                        }
                        _ => panic!("Can't be"),
                    };
                } else if occ_entry.occurences == 0 {
                    self.cache.cache[occ_index] = Some(super::cache::CacheEntry::occ_entry(&gs));
                }
            } else {
                self.cache.cache[index] = Some(super::cache::CacheEntry::occ_entry(&gs));
            }
        }
    }
}