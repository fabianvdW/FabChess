pub struct SearchStatistics {
    pub depth: usize,
    pub nodes_searched: u64,
    pub q_nodes_searched: u64,
    pub normal_nodes_searched: u64,
    pub q_delta_cutoffs: u64,
    pub q_see_cutoffs: u64,
    pub time_elapsed: f64,
}

impl SearchStatistics {
    pub fn new() -> SearchStatistics {
        SearchStatistics {
            depth: 0,
            nodes_searched: 0,
            q_nodes_searched: 0,
            normal_nodes_searched: 0,
            q_delta_cutoffs: 0,
            q_see_cutoffs: 0,
            time_elapsed: 0.0,
        }
    }
    pub fn add_normal_node(&mut self) {
        self.nodes_searched += 1;
        self.normal_nodes_searched += 1;
    }
    pub fn add_q_node(&mut self) {
        self.nodes_searched += 1;
        self.q_nodes_searched += 1;
    }
    pub fn add_q_delta_cutoff(&mut self) {
        self.q_delta_cutoffs += 1;
    }
    pub fn add_q_see_cutoff(&mut self) {
        self.q_see_cutoffs += 1;
    }
}