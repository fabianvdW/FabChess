use super::game_state::*;
use crate::move_generation::movegen;
pub const MGSA_KNIGHT: usize = 0;
pub const MGSA_BISHOP: usize = 1;
pub const MGSA_ROOKS: usize = 2;
pub const MGSA_QUEEN: usize = 3;
pub struct MutableGameStateAttackContainer {
    pub attack: Vec<Vec<Vec<u64>>>,
    pub king_attacks: [u64; 2],
    pub pawn_attacks: [u64; 2],
    pub attacks_minor_sum: [u64; 2],
    pub attacks_sum: [u64; 2],
    pub knights: [usize; 2],
    pub bishops: [usize; 2],
    pub rooks: [usize; 2],
    pub queens: [usize; 2],
}

impl MutableGameStateAttackContainer {
    pub fn write_state(&mut self, game_state: &GameState) {
        for side in 0..2 {
            // King Attacks
            self.king_attacks[side] =
                movegen::king_attack(game_state.pieces[KING][side].trailing_zeros() as usize);
            let mut attacks_minor_sum = 0u64;
            //Pawn attacks
            self.pawn_attacks[side] = if side == WHITE {
                movegen::w_pawn_west_targets(game_state.pieces[PAWN][side])
                    | movegen::w_pawn_east_targets(game_state.pieces[PAWN][side])
            } else {
                movegen::b_pawn_west_targets(
                    game_state.pieces[PAWN][side]
                        | movegen::b_pawn_east_targets(game_state.pieces[PAWN][side]),
                )
            };
        }
    }
}

impl Default for MutableGameStateAttackContainer {
    fn default() -> MutableGameStateAttackContainer {
        let mut attack: Vec<Vec<Vec<u64>>> = Vec::with_capacity(4);
        for i in 0..4 {
            attack.push(Vec::with_capacity(2));
            for _ in 0..2 {
                attack[i].push(vec![0u64; if i <= 2 { 10 } else { 9 }]);
            }
        }
        MutableGameStateAttackContainer {
            attack,
            king_attacks: [0u64; 2],
            pawn_attacks: [0u64; 2],
            attacks_minor_sum: [0u64; 2],
            attacks_sum: [0u64; 2],
            knights: [0; 2],
            bishops: [0; 2],
            rooks: [0; 2],
            queens: [0; 2],
        }
    }
}
