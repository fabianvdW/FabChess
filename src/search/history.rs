use super::alphabeta::MAX_SEARCH_DEPTH;
use crate::board_representation::game_state::GameState;
pub struct History {
    pub hist: Vec<u64>,
    pub is_unique: Vec<bool>,
    pub pointer: usize,
}
impl History {
    pub fn new() -> Self {
        History {
            hist: vec![0u64; MAX_SEARCH_DEPTH + 100],
            is_unique: vec![false; MAX_SEARCH_DEPTH + 100],
            pointer: 0,
        }
    }

    pub fn push(&mut self, hash: u64, is_unique: bool) {
        self.hist[self.pointer] = hash;
        self.is_unique[self.pointer] = is_unique;
        self.pointer += 1;
    }

    pub fn pop(&mut self) {
        self.pointer -= 1;
    }

    pub fn get_occurences(&self, game_state: &GameState) -> usize {
        let mut occurences = 0;
        let mut index = self.pointer as isize - 1;
        while index >= 0 {
            if self.hist[index as usize] == game_state.hash {
                occurences += 1;
            }
            if self.is_unique[index as usize] {
                break;
            }
            index -= 1;
        }
        occurences
    }
}
