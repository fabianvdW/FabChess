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
    pub snm_pruned: u64,
    pub static_eval_nodes: u64,
    pub cache_replace_eval: u64,
    pub iid_nodes: u64,
    pub futil_nodes: u64,
    pub history_pruned: u64,
}

impl Default for SearchStatistics {
    fn default() -> Self {
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
            snm_pruned: 0,
            static_eval_nodes: 0,
            cache_replace_eval: 0,
            iid_nodes: 0,
            futil_nodes: 0,
            history_pruned: 0,
        }
    }
}

impl SearchStatistics {
    #[inline(always)]
    pub fn refresh_time_elapsed(&mut self) {
        let now = Instant::now();
        let dur = now.duration_since(self.start_time);
        self.time_elapsed = dur.as_millis() as u64;
    }
    #[inline(always)]
    pub fn add_normal_node(&mut self, depth: usize) {
        self.nodes_searched += 1;
        self.normal_nodes_searched += 1;
        if depth > self.depth {
            self.depth = depth;
        }
    }
    #[inline(always)]
    pub fn add_history_pruned(&mut self) {
        self.history_pruned += 1;
    }
    #[inline(always)]
    pub fn add_q_node(&mut self, seldepth: usize) {
        self.nodes_searched += 1;
        self.q_nodes_searched += 1;
        if seldepth > self.seldepth {
            self.seldepth = seldepth;
        }
    }
    #[inline(always)]
    pub fn add_cache_hit_replace_eval(&mut self) {
        self.cache_replace_eval += 1;
    }
    #[inline(always)]
    pub fn add_futil_pruning(&mut self) {
        self.futil_nodes += 1;
    }
    #[inline(always)]
    pub fn add_iid_node(&mut self) {
        self.iid_nodes += 1;
    }
    #[inline(always)]
    pub fn add_static_eval_node(&mut self) {
        self.static_eval_nodes += 1;
    }
    #[inline(always)]
    pub fn add_static_null_move_node(&mut self) {
        self.snm_pruned += 1;
    }
    #[inline(always)]
    pub fn add_q_root(&mut self) {
        self.nodes_searched -= 1;
        self.normal_nodes_searched -= 1;
    }
    #[inline(always)]
    pub fn add_q_delta_cutoff(&mut self) {
        self.q_delta_cutoffs += 1;
    }
    #[inline(always)]
    pub fn add_q_see_cutoff(&mut self) {
        self.q_see_cutoffs += 1;
    }
    #[inline(always)]
    pub fn add_q_beta_cutoff(&mut self, index: usize) {
        self.q_beta_cutoffs += 1;
        self.q_beta_cutoffs_index[index] += 1;
    }
    #[inline(always)]
    pub fn add_q_beta_noncutoff(&mut self) {
        self.q_non_beta_cutoffs += 1;
    }
    #[inline(always)]
    pub fn add_normal_node_beta_cutoff(&mut self, index: usize) {
        self.normal_nodes_beta_cutoffs += 1;
        if index > 31 {
            self.normal_nodes_beta_cutoffs_index[31] += 1;
        } else {
            self.normal_nodes_beta_cutoffs_index[index] += 1;
        }
    }
    #[inline(always)]
    pub fn add_normal_node_non_beta_cutoff(&mut self) {
        self.normal_nodes_non_beta_cutoffs += 1;
    }
    #[inline(always)]
    pub fn add_cache_hit_ns(&mut self) {
        self.cache_hit_ns += 1;
    }
    #[inline(always)]
    pub fn add_cache_hit_replace_ns(&mut self) {
        self.cache_hit_replaces_ns += 1;
    }
    #[inline(always)]
    pub fn add_cache_hit_aj_replace_ns(&mut self) {
        self.cache_hit_aj_replaces_ns += 1;
    }
    pub fn add_cache_hit_qs(&mut self) {
        self.cache_hit_qs += 1;
    }
    #[inline(always)]
    pub fn add_cache_hit_replace_qs(&mut self) {
        self.cache_hit_replaces_qs += 1;
    }
    #[inline(always)]
    pub fn add_cache_hit_aj_replace_qs(&mut self) {
        self.cache_hit_aj_replaces_qs += 1;
    }
    #[inline(always)]
    pub fn add_nm_pruning(&mut self) {
        self.nm_pruned += 1;
    }

    #[inline(always)]
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
        res_str.push_str(&format!(
            "Depth reached: {}/{}\n",
            self.depth, self.seldepth
        ));
        res_str.push_str(&format!(
            "NPS: {}\n",
            self.nodes_searched as f64 / (self.time_elapsed as f64 / 1000.0)
        ));

        res_str.push_str("\n");
        res_str.push_str(&format!(
            "Normal nodes: {} ({}%)\n",
            self.normal_nodes_searched,
            (self.normal_nodes_searched as f64 / self.nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search Beta  cutoffs: {} ({}%)\n",
            self.normal_nodes_beta_cutoffs,
            (self.normal_nodes_beta_cutoffs as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search Beta  cutoffs: {:?}\n",
            self.normal_nodes_beta_cutoffs_index
        ));
        res_str.push_str(&format!(
            "Normal-Search No    cutoffs: {} ({}%)\n",
            self.normal_nodes_non_beta_cutoffs,
            (self.normal_nodes_non_beta_cutoffs as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search Cache-Hits:    {} ({}%)\n",
            self.cache_hit_ns,
            (self.cache_hit_ns as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search Cache-Hit-Replace: {} ({}%)\n",
            self.cache_hit_replaces_ns,
            (self.cache_hit_replaces_ns as f64 / self.cache_hit_ns as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search Cache-Hit-Adj-Replace: {} ({}%)\n",
            self.cache_hit_aj_replaces_ns,
            (self.cache_hit_aj_replaces_ns as f64 / self.cache_hit_ns as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search Cache-Hit-Replace-Eval: {} ({}%)\n",
            self.cache_replace_eval,
            (self.cache_replace_eval as f64 / self.cache_hit_ns as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search IID Nodes : {} ({}%)\n",
            self.iid_nodes,
            (self.iid_nodes as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search Static Eval Nodes : {} ({}%)\n",
            self.static_eval_nodes,
            (self.static_eval_nodes as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search NullMove-Pruned : {} ({}%)\n",
            self.nm_pruned,
            (self.nm_pruned as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search Staticnullmove-Pruned : {} ({}%)\n",
            self.snm_pruned,
            (self.snm_pruned as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search Futil-Pruned : {} ({}%)\n",
            self.futil_nodes,
            (self.futil_nodes as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search History-Pruned : {} ({}%)\n",
            self.history_pruned,
            (self.history_pruned as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));

        res_str.push_str("\n");
        res_str.push_str(&format!(
            "Quiescence nodes: {} ({}%)\n",
            self.q_nodes_searched,
            (self.q_nodes_searched as f64 / self.nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Q-Search Delta cutoffs: {} ({}%)\n",
            self.q_delta_cutoffs,
            (self.q_delta_cutoffs as f64 / self.q_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Q-Search SEE   cutoffs: {} ({}%)\n",
            self.q_see_cutoffs,
            (self.q_see_cutoffs as f64 / self.q_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Q-Search Beta  cutoffs: {} ({}%)\n",
            self.q_beta_cutoffs,
            (self.q_beta_cutoffs as f64 / self.q_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Q-Search Beta  cutoffs: {:?}\n",
            self.q_beta_cutoffs_index
        ));
        res_str.push_str(&format!(
            "Q-Search No    cutoffs: {} ({}%)\n",
            self.q_non_beta_cutoffs,
            (self.q_non_beta_cutoffs as f64 / self.q_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Q-Search Cache-Hits:    {} ({}%)\n",
            self.cache_hit_qs,
            (self.cache_hit_qs as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Q-Search Cache-Hit-Replace: {} ({}%)\n",
            self.cache_hit_replaces_qs,
            (self.cache_hit_replaces_qs as f64 / self.cache_hit_qs as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Q-Search Cache-Hit-Adj-Replace: {} ({}%)\n",
            self.cache_hit_aj_replaces_qs,
            (self.cache_hit_aj_replaces_qs as f64 / self.cache_hit_qs as f64 * 100.0)
        ));
        write!(formatter, "{}", res_str)
    }
}
