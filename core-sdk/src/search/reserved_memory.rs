use super::MAX_SEARCH_DEPTH;
use crate::move_generation::movegen::MoveList;

pub struct ReserveMemory {
    pub reserved_movelist: ReservedMoveList,
}

impl Default for ReserveMemory {
    fn default() -> ReserveMemory {
        ReserveMemory {
            reserved_movelist: ReservedMoveList::default(),
        }
    }
}

pub struct ReservedMoveList {
    pub move_lists: Vec<MoveList>,
}

impl Default for ReservedMoveList {
    fn default() -> ReservedMoveList {
        let mut move_lists = Vec::with_capacity(MAX_SEARCH_DEPTH);
        for _ in 0..MAX_SEARCH_DEPTH {
            move_lists.push(MoveList::default());
        }
        ReservedMoveList { move_lists }
    }
}
