use super::magic::{self};
use crate::bitboards::bitboards;
use crate::bitboards::bitboards::constants::RANKS;
use crate::board_representation::game_state::WHITE;

//Movegen
//King - piecewise by lookup
//Knight - piecewise by lookup
//Bishop/Queen/Rook - piecewise by lookup in magic
//Pawn - setwise by shift

#[inline(always)]
pub fn bishop_attacks(square: usize, all_pieces: u64) -> u64 {
    magic::MAGIC_BISHOP[square].apply(all_pieces)
}

#[inline(always)]
pub fn rook_attacks(square: usize, all_pieces: u64) -> u64 {
    magic::MAGIC_ROOK[square].apply(all_pieces)
}

//Pawn single pushes

#[inline(always)]
pub fn single_push_pawn_targets(side: usize, pawns: u64, empty: u64) -> u64 {
    if side == WHITE {
        w_single_push_pawn_targets(pawns, empty)
    } else {
        b_single_push_pawn_targets(pawns, empty)
    }
}

#[inline(always)]
pub fn w_single_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::north_one(pawns) & empty
}

#[inline(always)]
pub fn b_single_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(pawns) & empty
}

//Pawn double pushes
#[inline(always)]
pub fn double_push_pawn_targets(side: usize, pawns: u64, empty: u64) -> u64 {
    if side == WHITE {
        w_double_push_pawn_targets(pawns, empty)
    } else {
        b_double_push_pawn_targets(pawns, empty)
    }
}

#[inline(always)]
pub fn w_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::north_one(bitboards::north_one(pawns & RANKS[1]) & empty) & empty
}

#[inline(always)]
pub fn b_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(bitboards::south_one(pawns & RANKS[6]) & empty) & empty
}

//All targets
#[inline(always)]
pub fn pawn_targets(side: usize, pawns: u64) -> u64 {
    pawn_east_targets(side, pawns) | pawn_west_targets(side, pawns)
}
//Pawn east targets
#[inline(always)]
pub fn pawn_east_targets(side: usize, pawns: u64) -> u64 {
    if side == WHITE {
        w_pawn_east_targets(pawns)
    } else {
        b_pawn_east_targets(pawns)
    }
}

//NorthEast = +9
#[inline(always)]
pub fn w_pawn_east_targets(pawns: u64) -> u64 {
    bitboards::north_east_one(pawns)
}

//SouthEast = -7
#[inline(always)]
pub fn b_pawn_east_targets(pawns: u64) -> u64 {
    bitboards::south_west_one(pawns)
}

//Pawn west targets
#[inline(always)]
pub fn pawn_west_targets(side: usize, pawns: u64) -> u64 {
    if side == WHITE {
        w_pawn_west_targets(pawns)
    } else {
        b_pawn_west_targets(pawns)
    }
}

//NorthWest = +7
#[inline(always)]
pub fn w_pawn_west_targets(pawns: u64) -> u64 {
    bitboards::north_west_one(pawns)
}

//NorthWest = -9
#[inline(always)]
pub fn b_pawn_west_targets(pawns: u64) -> u64 {
    bitboards::south_east_one(pawns)
}

#[inline(always)]
pub fn xray_rook_attacks(
    attacks: u64,
    occupied_squares: u64,
    my_pieces: u64,
    rook_square: usize,
) -> u64 {
    attacks ^ rook_attacks(rook_square, occupied_squares ^ (my_pieces & attacks))
}

#[inline(always)]
pub fn xray_bishop_attacks(
    attacks: u64,
    occupied_squares: u64,
    my_pieces: u64,
    bishop_square: usize,
) -> u64 {
    attacks ^ bishop_attacks(bishop_square, occupied_squares ^ (my_pieces & attacks))
}
