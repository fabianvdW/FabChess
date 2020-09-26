use std::fmt::{Display, Formatter, Result};

pub struct SearchStatistics {
    pub improving2: [u64; 2],
    pub depth: usize,
    pub seldepth: usize,
    pub nodes_searched: u64,
    pub q_nodes_searched: u64,
    pub normal_nodes_searched: u64,
    pub nodes_improving: [u64; 2],
    pub q_delta_cutoffs: u64,
    pub q_see_cutoffs: u64,
    pub q_beta_cutoffs: u64,
    pub q_beta_cutoffs_index: [usize; 32],
    pub q_non_beta_cutoffs: u64,
    pub normal_nodes_beta_cutoffs: u64,
    pub normal_nodes_beta_cutoffs_index: [usize; 32],
    pub normal_nodes_non_beta_cutoffs: u64,
    pub normal_nodes_fail_lows: u64,
    pub normal_nodes_improv_cutoffs: [u64; 2],
    pub normal_nodes_improv_faillows: [u64; 2],
    pub cache_hit: u64,
    pub cache_hit_aj_replaces: u64,
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
            improving2: [0; 2],
            depth: 0,
            seldepth: 0,
            nodes_searched: 0,
            q_nodes_searched: 0,
            normal_nodes_searched: 0,
            nodes_improving: [0; 2],
            q_delta_cutoffs: 0,
            q_see_cutoffs: 0,
            q_beta_cutoffs: 0,
            q_beta_cutoffs_index: [0; 32],
            q_non_beta_cutoffs: 0,
            normal_nodes_beta_cutoffs: 0,
            normal_nodes_non_beta_cutoffs: 0,
            normal_nodes_beta_cutoffs_index: [0; 32],
            normal_nodes_fail_lows: 0,
            normal_nodes_improv_cutoffs: [0; 2],
            normal_nodes_improv_faillows: [0; 2],
            cache_hit: 0,
            cache_hit_aj_replaces: 0,
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
        self.cache_hit += 1;
    }
    #[inline(always)]
    pub fn add_cache_hit_aj_replace_ns(&mut self) {
        self.cache_hit += 1;
        self.cache_hit_aj_replaces += 1;
    }
    #[inline(always)]
    pub fn add_nm_pruning(&mut self) {
        self.nm_pruned += 1;
    }
}

impl Display for SearchStatistics {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let mut res_str: String = String::new();
        res_str.push_str(&format!("Improving2: {:?}\n", self.improving2));
        res_str.push_str(&format!("Nodes searched: {}\n", self.nodes_searched));
        res_str.push_str(&format!("Depth reached: {}/{}\n", self.depth, self.seldepth));
        res_str.push_str("\n");
        res_str.push_str(&format!(
            "Normal nodes: {} ({}%)\n",
            self.normal_nodes_searched,
            (self.normal_nodes_searched as f64 / self.nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!("Nodes improving: {:?}\n", self.nodes_improving));
        res_str.push_str(&format!(
            "Normal-Search Beta  cutoffs: {} ({}%)\n",
            self.normal_nodes_beta_cutoffs,
            (self.normal_nodes_beta_cutoffs as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!("Normal-Search Beta  cutoffs: {:?}\n", self.normal_nodes_beta_cutoffs_index));
        res_str.push_str(&format!(
            "Normal-Search No    cutoffs: {} ({}%)\n",
            self.normal_nodes_non_beta_cutoffs,
            (self.normal_nodes_non_beta_cutoffs as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!("Normal-Search Beta cutoffs by improv: {:?}\n", self.normal_nodes_improv_cutoffs));
        res_str.push_str(&format!("Normal-Search Faillows: {:?}\n", self.normal_nodes_fail_lows));
        res_str.push_str(&format!("Normal-Search Faillows by improv: {:?}\n", self.normal_nodes_improv_faillows));
        res_str.push_str(&format!(
            "Cache-Hits:    {} ({}%)\n",
            self.cache_hit,
            (self.cache_hit as f64 / self.normal_nodes_searched as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Cache-Hit-Adj-Replace: {} ({}%)\n",
            self.cache_hit_aj_replaces,
            (self.cache_hit_aj_replaces as f64 / self.cache_hit as f64 * 100.0)
        ));
        res_str.push_str(&format!(
            "Normal-Search Cache-Hit-Replace-Eval: {} ({}%)\n",
            self.cache_replace_eval,
            (self.cache_replace_eval as f64 / self.cache_hit as f64 * 100.0)
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
        res_str.push_str(&format!("Q-Search Beta  cutoffs: {:?}\n", self.q_beta_cutoffs_index));
        res_str.push_str(&format!(
            "Q-Search No    cutoffs: {} ({}%)\n",
            self.q_non_beta_cutoffs,
            (self.q_non_beta_cutoffs as f64 / self.q_nodes_searched as f64 * 100.0)
        ));
        write!(formatter, "{}", res_str)
    }
}
