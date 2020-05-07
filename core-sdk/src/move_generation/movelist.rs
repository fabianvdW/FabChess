use crate::bitboards::bitboards::constants::square;
use crate::board_representation::game_state::{
    GameMove, GameMoveType, GameState, PieceType, WHITE,
};
use crate::search::GradedMove;

pub const MAX_MOVES: usize = 128;

pub struct MoveList {
    pub move_list: Vec<GradedMove>,
}
impl Default for MoveList {
    fn default() -> Self {
        let move_list = Vec::with_capacity(MAX_MOVES);
        MoveList { move_list }
    }
}

impl MoveList {
    //This deserializes a bitboard with target destinations for a certain piece with piece_type on the from square
    #[inline(always)]
    pub fn add_bb(&mut self, from: usize, piece_type: PieceType, mut bb: u64, state: &GameState) {
        while bb > 0 {
            let to = bb.trailing_zeros() as usize;
            self.add_move(GameMove::new(from, to, state.move_type_to(to), piece_type));
            bb ^= square(to);
        }
    }
    //This deserializes a bitboard with setwise target destinations for all pawns
    //Does not work for en-passants
    pub fn add_pawn_bb(&mut self, mut bb: u64, shift: i8, state: &GameState) {
        while bb > 0 {
            let to = bb.trailing_zeros() as usize;
            let is_promotion = to <= 7 || to >= 56;
            if is_promotion {
                let captured_pt = state.piecetype_on(to).map(|x| {
                    debug_assert!((1 - state.color_to_move == WHITE) == x.1);
                    x.0
                });
                for pt in [
                    PieceType::Queen,
                    PieceType::Rook,
                    PieceType::Bishop,
                    PieceType::Knight,
                ]
                .iter()
                {
                    self.add_move(GameMove::new(
                        (to as i8 + shift) as usize,
                        to,
                        GameMoveType::Promotion(*pt, captured_pt),
                        PieceType::Pawn,
                    ));
                }
            } else {
                self.add_move(GameMove::new(
                    (to as i8 + shift) as usize,
                    to,
                    state.move_type_to(to),
                    PieceType::Pawn,
                ));
            }
            bb ^= square(to)
        }
    }
    #[inline(always)]
    pub fn add_move(&mut self, mv: GameMove) {
        self.move_list.push(GradedMove(mv, None));
    }

    #[inline(always)]
    pub fn find_move(&self, mv: GameMove, contains: bool) -> usize {
        for (i, gmv) in self.move_list.iter().enumerate() {
            if gmv.0 == mv {
                return i;
            }
        }
        if contains {
            panic!("Type 2 error")
        }
        self.move_list.len()
    }

    #[inline(always)]
    pub fn highest_score(&mut self) -> Option<(usize, GradedMove)> {
        let mut best_index = self.move_list.len();
        let mut best_score = -1_000_000_000.;
        for (index, gmv) in self.move_list.iter().enumerate() {
            if gmv.1.is_some() && gmv.1.unwrap() > best_score {
                best_index = index;
                best_score = gmv.1.unwrap();
            }
        }
        if best_index == self.move_list.len() {
            None
        } else {
            Some((best_index, self.move_list[best_index]))
        }
    }
}
