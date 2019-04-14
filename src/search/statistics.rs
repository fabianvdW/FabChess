use std::fmt::{Display, Formatter, Result};
use std::time::Instant;

pub struct SearchStatistics {
    pub depth: usize,
    pub seldepth: usize,
    pub nodes_searched: u64,
    pub q_nodes_searched: u64,
    pub normal_nodes_searched: u64,
    pub q_delta_cutoffs: u64,
    pub q_see_cutoffs: u64,
    pub time_elapsed: u64,
    pub start_time: Instant,
}

impl SearchStatistics {
    pub fn new() -> SearchStatistics {
        SearchStatistics {
            depth: 0,
            seldepth: 0,
            nodes_searched: 0,
            q_nodes_searched: 0,
            normal_nodes_searched: 0,
            q_delta_cutoffs: 0,
            q_see_cutoffs: 0,
            time_elapsed: 0,
            start_time: Instant::now(),
        }
    }
    pub fn refresh_time_elapsed(&mut self) {
        let now = Instant::now();
        let dur = now.duration_since(self.start_time);
        self.time_elapsed = dur.as_millis() as u64;
    }
    pub fn add_normal_node(&mut self, depth: usize) {
        self.nodes_searched += 1;
        self.normal_nodes_searched += 1;
        if depth > self.depth {
            self.depth = depth;
        }
    }
    pub fn add_q_node(&mut self, seldepth: usize) {
        self.nodes_searched += 1;
        self.q_nodes_searched += 1;
        if seldepth > self.seldepth {
            self.seldepth = seldepth;
        }
    }
    pub fn add_q_root(&mut self) {
        self.nodes_searched -= 1;
    }
    pub fn add_q_delta_cutoff(&mut self) {
        self.q_delta_cutoffs += 1;
    }
    pub fn add_q_see_cutoff(&mut self) {
        self.q_see_cutoffs += 1;
    }
}

impl Display for SearchStatistics {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!("Time elapsed: {}ms\n", self.time_elapsed));
        res_str.push_str(&format!("Nodes searched: {}\n", self.nodes_searched));
        res_str.push_str(&format!("Depth reached: {}/{}\n", self.depth, self.seldepth));
        res_str.push_str(&format!("NPS: {}\n", self.nodes_searched as f64 / (self.time_elapsed as f64 / 1000.0)));
        res_str.push_str(&format!("Quiesence nodes: {} ({}%)\n", self.q_nodes_searched, (self.q_nodes_searched as f64 / self.nodes_searched as f64 * 100.0)));
        res_str.push_str(&format!("Q-Search Delta cutoffs: {} ({}%)\n", self.q_delta_cutoffs, (self.q_delta_cutoffs as f64 / self.q_nodes_searched as f64 * 100.0)));
        res_str.push_str(&format!("Q-Search SEE   cutoffs: {} ({}%)\n", self.q_see_cutoffs, (self.q_see_cutoffs as f64 / self.q_nodes_searched as f64 * 100.0)));
        write!(formatter, "{}", res_str)
    }
}