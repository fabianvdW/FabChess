use super::cache::{Cache, CacheEntry};
use super::statistics::SearchStatistics;
use super::super::GameState;
use super::GameMove;
use super::alphabeta::PrincipalVariation;
use super::alphabeta::principal_variation_search;

pub struct Search<'a> {
    pub cache: &'a mut Cache,
    pub principal_variation: [Option<CacheEntry>; 100],
    pub killer_moves: [[Option<GameMove>;2];100],
    pub search_statistics: SearchStatistics,
    pub game_state: &'a GameState,
}

impl<'a> Search<'a> {
    pub fn new(cache: &'a mut Cache, state: &'a GameState) -> Search<'a> {
        Search {
            cache,
            principal_variation: [None; 100],
            search_statistics: SearchStatistics::new(),
            killer_moves:[[None;2];100],
            game_state: state,
        }
    }

    pub fn search(&mut self, depth: isize) -> PrincipalVariation {
        let mut stats = SearchStatistics::new();
        let mut best_pv = PrincipalVariation::new(0);
        for d in 1..depth + 1 {
            let pv = principal_variation_search(-100000.0, 100000.0, d, &self.game_state, if self.game_state.color_to_move == 0 {
                1
            } else {
                -1
            }, &mut stats, 0, self);
            println!("{}",format!("Depth {} with nodes {} and pv: {}",d,stats.nodes_searched,pv));
            //Set PV in table
            let mut pv_stack = Vec::with_capacity(pv.pv.len());
            for (i, pair) in pv.pv.iter().enumerate() {
                if i == 0 {
                    pv_stack.push(crate::move_generation::movegen::make_move(self.game_state, &pair));
                } else {
                    pv_stack.push(crate::move_generation::movegen::make_move(&pv_stack[i - 1], &pair))
                }
            }
            for (i, pair) in pv.pv.iter().enumerate() {
                let state= if i==0{self.game_state}else{&pv_stack[i-1]};
                self.principal_variation[i] = Some(CacheEntry::new(state, d  - i as isize , pv.score, false, false, &pair));
            }
            best_pv = pv;
        }
        self.search_statistics = stats;
        self.search_statistics.refresh_time_elapsed();
        return best_pv;
    }
}