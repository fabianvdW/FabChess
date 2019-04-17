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
    pub q_beta_cutoffs: u64,
    pub q_beta_cutoffs_index: [usize; 32],
    pub q_non_beta_cutoffs: u64,
    pub normal_nodes_beta_cutoffs: u64,
    pub normal_nodes_beta_cutoffs_index: [usize; 32],
    pub normal_nodes_non_beta_cutoffs: u64,
    pub time_elapsed: u64,
    pub start_time: Instant,
    pub cache_hit_ns: u64,
    pub cache_hit_replaces_ns: u64,
    pub cache_hit_aj_replaces_ns: u64,
    pub cache_hit_qs: u64,
    pub cache_hit_replaces_qs: u64,
    pub cache_hit_aj_replaces_qs: u64,
    pub nm_pruned: u64,
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
            q_beta_cutoffs: 0,
            q_beta_cutoffs_index: [0; 32],
            q_non_beta_cutoffs: 0,
            normal_nodes_beta_cutoffs: 0,
            normal_nodes_non_beta_cutoffs: 0,
            normal_nodes_beta_cutoffs_index: [0; 32],
            time_elapsed: 0,
            start_time: Instant::now(),
            cache_hit_ns: 0,
            cache_hit_replaces_ns: 0,
            cache_hit_aj_replaces_ns: 0,
            cache_hit_qs: 0,
            cache_hit_replaces_qs: 0,
            cache_hit_aj_replaces_qs: 0,
            nm_pruned: 0,
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
        self.normal_nodes_searched -= 1;
    }
    pub fn add_q_delta_cutoff(&mut self) {
        self.q_delta_cutoffs += 1;
    }
    pub fn add_q_see_cutoff(&mut self) {
        self.q_see_cutoffs += 1;
    }
    pub fn add_q_beta_cutoff(&mut self, index: usize) {
        self.q_beta_cutoffs += 1;
        self.q_beta_cutoffs_index[index] += 1;
    }
    pub fn add_q_beta_noncutoff(&mut self) {
        self.q_non_beta_cutoffs += 1;
    }
    pub fn add_normal_node_beta_cutoff(&mut self, index: usize) {
        self.normal_nodes_beta_cutoffs += 1;
        if index > 31 {
            self.normal_nodes_beta_cutoffs_index[31] += 1;
        } else {
            self.normal_nodes_beta_cutoffs_index[index] += 1;
        }
    }
    pub fn add_normal_node_non_beta_cutoff(&mut self) {
        self.normal_nodes_non_beta_cutoffs += 1;
    }
    pub fn add_cache_hit_ns(&mut self) {
        self.cache_hit_ns += 1;
    }
    pub fn add_cache_hit_replace_ns(&mut self) {
        self.cache_hit_replaces_ns += 1;
    }
    pub fn add_cache_hit_aj_replace_ns(&mut self) {
        self.cache_hit_aj_replaces_ns += 1;
    }
    pub fn add_cache_hit_qs(&mut self) {
        self.cache_hit_qs += 1;
    }
    pub fn add_cache_hit_replace_qs(&mut self) {
        self.cache_hit_replaces_qs += 1;
    }
    pub fn add_cache_hit_aj_replace_qs(&mut self) {
        self.cache_hit_aj_replaces_qs += 1;
    }
    pub fn add_nm_pruning(&mut self) {
        self.nm_pruned += 1;
    }

    pub fn getnps(&mut self) -> u64 {
        self.refresh_time_elapsed();
        (self.nodes_searched as f64 / (self.time_elapsed as f64 / 1000.0)) as u64
    }
}

impl Display for SearchStatistics {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!("Time elapsed: {}ms\n", self.time_elapsed));
        res_str.push_str(&format!("Nodes searched: {}\n", self.nodes_searched));
        res_str.push_str(&format!("Depth reached: {}/{}\n", self.depth, self.seldepth));
        res_str.push_str(&format!("NPS: {}\n", self.nodes_searched as f64 / (self.time_elapsed as f64 / 1000.0)));

        res_str.push_str("\n");
        res_str.push_str(&format!("Normal nodes: {} ({}%)\n", self.normal_nodes_searched, (self.normal_nodes_searched as f64 / self.nodes_searched as f64 * 100.0)));
        res_str.push_str(&format!("Normal-Search Beta  cutoffs: {} ({}%)\n", self.normal_nodes_beta_cutoffs, (self.normal_nodes_beta_cutoffs as f64 / self.normal_nodes_searched as f64 * 100.0)));
        res_str.push_str(&format!("Normal-Search Beta  cutoffs: {:?}\n", self.normal_nodes_beta_cutoffs_index));
        res_str.push_str(&format!("Normal-Search No    cutoffs: {} ({}%)\n", self.normal_nodes_non_beta_cutoffs, (self.normal_nodes_non_beta_cutoffs as f64 / self.normal_nodes_searched as f64 * 100.0)));
        res_str.push_str(&format!("Normal-Search Cache-Hits:    {} ({}%)\n", self.cache_hit_ns, (self.cache_hit_ns as f64 / self.normal_nodes_searched as f64 * 100.0)));
        res_str.push_str(&format!("Normal-Search Cache-Hit-Replace: {} ({}%)\n", self.cache_hit_replaces_ns, (self.cache_hit_replaces_ns as f64 / self.cache_hit_ns as f64 * 100.0)));
        res_str.push_str(&format!("Normal-Search Cache-Hit-Adj-Replace: {} ({}%)\n", self.cache_hit_aj_replaces_ns, (self.cache_hit_aj_replaces_ns as f64 / self.cache_hit_ns as f64 * 100.0)));
        res_str.push_str(&format!("Normal-Search NullMove-Pruned : {} ({}%)\n", self.nm_pruned, (self.nm_pruned as f64 / self.normal_nodes_searched as f64 * 100.0)));

        res_str.push_str("\n");
        res_str.push_str(&format!("Quiesence nodes: {} ({}%)\n", self.q_nodes_searched, (self.q_nodes_searched as f64 / self.nodes_searched as f64 * 100.0)));
        res_str.push_str(&format!("Q-Search Delta cutoffs: {} ({}%)\n", self.q_delta_cutoffs, (self.q_delta_cutoffs as f64 / self.q_nodes_searched as f64 * 100.0)));
        res_str.push_str(&format!("Q-Search SEE   cutoffs: {} ({}%)\n", self.q_see_cutoffs, (self.q_see_cutoffs as f64 / self.q_nodes_searched as f64 * 100.0)));
        res_str.push_str(&format!("Q-Search Beta  cutoffs: {} ({}%)\n", self.q_beta_cutoffs, (self.q_beta_cutoffs as f64 / self.q_nodes_searched as f64 * 100.0)));
        res_str.push_str(&format!("Q-Search Beta  cutoffs: {:?}\n", self.q_beta_cutoffs_index));
        res_str.push_str(&format!("Q-Search No    cutoffs: {} ({}%)\n", self.q_non_beta_cutoffs, (self.q_non_beta_cutoffs as f64 / self.q_nodes_searched as f64 * 100.0)));
        res_str.push_str(&format!("Q-Search Cache-Hits:    {} ({}%)\n", self.cache_hit_qs, (self.cache_hit_qs as f64 / self.normal_nodes_searched as f64 * 100.0)));
        res_str.push_str(&format!("Q-Search Cache-Hit-Replace: {} ({}%)\n", self.cache_hit_replaces_qs, (self.cache_hit_replaces_qs as f64 / self.cache_hit_qs as f64 * 100.0)));
        res_str.push_str(&format!("Q-Search Cache-Hit-Adj-Replace: {} ({}%)\n", self.cache_hit_aj_replaces_qs, (self.cache_hit_aj_replaces_qs as f64 / self.cache_hit_qs as f64 * 100.0)));
        write!(formatter, "{}", res_str)
    }
}