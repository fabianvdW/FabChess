use super::MAX_SEARCH_DEPTH;
use crate::board_representation::game_state_attack_container::GameStateAttackContainer;
use crate::move_generation::movegen::MoveList;

pub struct ReserveMemory {
    pub reserved_movelist: ReservedMoveList,
    pub reserved_attack_container: ReservedAttackContainer,
}

impl Default for ReserveMemory {
    fn default() -> ReserveMemory {
        ReserveMemory {
            reserved_movelist: ReservedMoveList::default(),
            reserved_attack_container: ReservedAttackContainer::default(),
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

pub struct ReservedAttackContainer {
    pub attack_containers: Vec<GameStateAttackContainer>,
}

impl Default for ReservedAttackContainer {
    fn default() -> ReservedAttackContainer {
        let mut attack_containers = Vec::with_capacity(MAX_SEARCH_DEPTH);
        for _ in 0..MAX_SEARCH_DEPTH {
            attack_containers.push(GameStateAttackContainer::default());
        }
        ReservedAttackContainer { attack_containers }
    }
}
