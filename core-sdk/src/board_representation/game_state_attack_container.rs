use super::game_state::*;
use crate::bitboards::bitboards::constants::{KING_ATTACKS, KNIGHT_ATTACKS};
use crate::move_generation::movegen;

pub const MGSA_KNIGHT: usize = 0;
pub const MGSA_BISHOP: usize = 1;
pub const MGSA_ROOKS: usize = 2;
pub const MGSA_QUEEN: usize = 3;

pub struct GameStateAttackContainer {
    pub attack: Vec<Vec<Vec<u64>>>,
    pub king_attacks: [u64; 2],
    pub pawn_west_attacks: [u64; 2],
    pub pawn_east_attacks: [u64; 2],
    pub pawn_attacks: [u64; 2],
    pub attacks_minor_sum: [u64; 2],
    //Attacked by only pawns, knights, bishops
    pub attacks_major_sum: [u64; 2],
    // Attacked by only rooks, queens
    pub attacks_sum: [u64; 2],
    //Attacked by pawns | knights | bishops | rooks | queens |kings
    pub knights: [usize; 2],
    pub bishops: [usize; 2],
    pub rooks: [usize; 2],
    pub queens: [usize; 2],
}

impl GameStateAttackContainer {
    pub fn from_state(game_state: &GameState) -> Self {
        let mut res = GameStateAttackContainer::default();
        res.write_state(game_state);
        res
    }

    pub fn write_state(&mut self, game_state: &GameState) {
        let all_pieces_without_stmking = game_state
            .get_pieces_from_side_without_king(game_state.color_to_move)
            | game_state.get_pieces_from_side(1 - game_state.color_to_move); // Enemy pieces can xray through my king
        let all_pieces = all_pieces_without_stmking
            | game_state.pieces[PieceType::King as usize][game_state.color_to_move];

        for side in 0..2 {
            self.attacks_minor_sum[side] = 0u64;
            self.attacks_major_sum[side] = 0u64;
            self.attacks_sum[side] = 0u64;
            let occupancy_squares = if side == game_state.color_to_move {
                all_pieces
            } else {
                all_pieces_without_stmking
            };
            // King Attacks
            self.king_attacks[side] = KING_ATTACKS
                [game_state.pieces[PieceType::King as usize][side].trailing_zeros() as usize];
            //Pawn attacks
            self.pawn_west_attacks[side] =
                movegen::pawn_west_targets(side, game_state.pieces[PieceType::Pawn as usize][side]);
            self.pawn_east_attacks[side] =
                movegen::pawn_east_targets(side, game_state.pieces[PieceType::Pawn as usize][side]);
            self.pawn_attacks[side] = self.pawn_west_attacks[side] | self.pawn_east_attacks[side];
            let mut attacks_minor_sum = self.pawn_attacks[side];
            //Knight attacks
            let mut knights = game_state.pieces[PieceType::Knight as usize][side];
            self.knights[side] = 0;
            while knights != 0u64 {
                let knight_index = knights.trailing_zeros() as usize;
                let attack = KNIGHT_ATTACKS[knight_index];
                self.attack[MGSA_KNIGHT][side][self.knights[side]] = attack;
                attacks_minor_sum |= attack;
                self.knights[side] += 1;
                knights ^= 1u64 << knight_index;
            }
            //Bishop attack
            let mut bishops = game_state.pieces[PieceType::Bishop as usize][side];
            self.bishops[side] = 0;
            while bishops != 0u64 {
                let bishop_index = bishops.trailing_zeros() as usize;
                let attack = movegen::bishop_attack(bishop_index, occupancy_squares);
                self.attack[MGSA_BISHOP][side][self.bishops[side]] = attack;
                attacks_minor_sum |= attack;
                self.bishops[side] += 1;
                bishops ^= 1u64 << bishop_index;
            }
            //Rooks
            let mut attacks_major_sum = 0u64;
            let mut rooks = game_state.pieces[PieceType::Rook as usize][side];
            self.rooks[side] = 0;
            while rooks != 0u64 {
                let rook_index = rooks.trailing_zeros() as usize;
                let attack = movegen::rook_attack(rook_index, occupancy_squares);
                self.attack[MGSA_ROOKS][side][self.rooks[side]] = attack;
                attacks_major_sum |= attack;
                self.rooks[side] += 1;
                rooks ^= 1u64 << rook_index;
            }
            //Queens
            let mut queens = game_state.pieces[PieceType::Queen as usize][side];
            self.queens[side] = 0;
            while queens != 0u64 {
                let queen_index = queens.trailing_zeros() as usize;
                let attack = movegen::rook_attack(queen_index, occupancy_squares)
                    | movegen::bishop_attack(queen_index, occupancy_squares);
                self.attack[MGSA_QUEEN][side][self.queens[side]] = attack;
                attacks_major_sum |= attack;
                self.queens[side] += 1;
                queens ^= 1u64 << queen_index;
            }
            self.attacks_minor_sum[side] = attacks_minor_sum;
            self.attacks_major_sum[side] = attacks_major_sum;
            self.attacks_sum[side] =
                attacks_minor_sum | attacks_major_sum | self.king_attacks[side];
        }
    }
}

impl Default for GameStateAttackContainer {
    fn default() -> GameStateAttackContainer {
        let mut attack: Vec<Vec<Vec<u64>>> = Vec::with_capacity(4);
        for i in 0..4 {
            attack.push(Vec::with_capacity(2));
            for _ in 0..2 {
                attack[i].push(vec![0u64; if i <= 2 { 10 } else { 9 }]);
            }
        }
        GameStateAttackContainer {
            attack,
            king_attacks: [0u64; 2],
            pawn_west_attacks: [0u64; 2],
            pawn_east_attacks: [0u64; 2],
            pawn_attacks: [0u64; 2],
            attacks_minor_sum: [0u64; 2],
            attacks_major_sum: [0u64; 2],
            attacks_sum: [0u64; 2],
            knights: [0; 2],
            bishops: [0; 2],
            rooks: [0; 2],
            queens: [0; 2],
        }
    }
}
