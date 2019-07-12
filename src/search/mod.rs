use crate::board_representation::game_state::GameMove;
use std::cmp::Ordering;

pub mod alphabeta;
pub mod cache;
pub mod quiescence;
pub mod search;
pub mod statistics;

pub fn init_constants() {
    quiescence::PIECE_VALUES.len();
}

#[derive(Clone)]
pub struct GradedMove {
    pub mv_index: usize,
    pub score: f64,
}

impl GradedMove {
    pub fn new(mv_index: usize, score: f64) -> GradedMove {
        GradedMove { mv_index, score }
    }
}

impl Eq for GradedMove {}

impl PartialEq for GradedMove {
    fn eq(&self, other: &GradedMove) -> bool {
        self.score == other.score
    }
}

impl Ord for GradedMove {
    fn cmp(&self, other: &GradedMove) -> Ordering {
        if self.score > other.score {
            return Ordering::Less;
        } else if self.score < other.score {
            return Ordering::Greater;
        }
        Ordering::Equal
    }
}

impl PartialOrd for GradedMove {
    fn partial_cmp(&self, other: &GradedMove) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
