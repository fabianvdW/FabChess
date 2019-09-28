use crate::logging::log;
use crate::move_generation::movegen::{bishop_attack, rook_attack};

lazy_static! {
    pub static ref FILES: [u64; 8] = initialize_files();
    pub static ref NOT_FILES: [u64; 8] = initialize_not_files();
    pub static ref RANKS: [u64; 8] = initialize_ranks();
    pub static ref SQUARES: [u64; 64] = initialize_squares();
    pub static ref NOT_SQUARES: [u64; 64] = initialize_not_squares();
    pub static ref KING_ATTACKS: [u64; 64] = init_king_attacks();
    pub static ref KNIGHT_ATTACKS: [u64; 64] = init_knight_attacks();
    pub static ref FILES_LESS_THAN: [u64; 8] = init_files_less_than();
    pub static ref FILES_GREATER_THAN: [u64; 8] = init_files_greater_than();
    pub static ref RANKS_LESS_THAN: [u64; 8] = init_ranks_less_than();
    pub static ref RANKS_GREATER_THAN: [u64; 8] = init_ranks_greater_than();
    pub static ref UPPER_HALF: u64 = init_upper_half();
    pub static ref LOWER_HALF: u64 = init_lower_half();
    pub static ref DIAGONALLY_ADJACENT: [u64; 64] = init_diagonally_adjacent();
    pub static ref SHIELDING_PAWNS_WHITE: [u64; 64] = init_shielding_pawns_white();
    pub static ref SHIELDING_PAWNS_BLACK: [u64; 64] = init_shielding_pawns_black();
    pub static ref CENTER: u64 = initialize_center();
    pub static ref INNER_CENTER: u64 = initialize_inner_center();
    pub static ref FREEFIELD_BISHOP_ATTACKS: [u64; 64] = initialize_freefield_bishop_attacks();
    pub static ref FREEFIELD_ROOK_ATTACKS: [u64; 64] = initialize_freefield_rook_attacks();
    pub static ref ROOK_RAYS: [[u64; 64]; 64] = initialize_rook_rays();
    pub static ref BISHOP_RAYS: [[u64; 64]; 64] = initialize_bishop_rays();
    pub static ref KING_ZONE_WHITE: [u64; 64] = initialize_king_zone_white();
    pub static ref KING_ZONE_BLACK: [u64; 64] = initialize_king_zone_black();
}

pub fn init_bitboards() {
    FILES.len();
    NOT_FILES.len();
    RANKS.len();
    SQUARES.len();
    NOT_SQUARES.len();
    KING_ATTACKS.len();
    KNIGHT_ATTACKS.len();
    FILES_LESS_THAN.len();
    FILES_GREATER_THAN.len();
    RANKS_LESS_THAN.len();
    RANKS_GREATER_THAN.len();
    UPPER_HALF.count_ones();
    LOWER_HALF.count_ones();
    DIAGONALLY_ADJACENT.len();
    SHIELDING_PAWNS_WHITE.len();
    SHIELDING_PAWNS_BLACK.len();
    (*CENTER).trailing_zeros();
    (*INNER_CENTER).trailing_zeros();
    FREEFIELD_BISHOP_ATTACKS.len();
    FREEFIELD_ROOK_ATTACKS.len();
    ROOK_RAYS.len();
    BISHOP_RAYS.len();
    KING_ZONE_WHITE.len();
    KING_ZONE_BLACK.len();
}

pub fn initialize_king_zone_black() -> [u64; 64] {
    let mut res = [0u64; 64];
    for king_sq in 0..64 {
        let zone = 1u64 << king_sq | KING_ATTACKS[king_sq];
        res[king_sq] = zone | south_one(zone);
    }
    res
}

pub fn initialize_king_zone_white() -> [u64; 64] {
    let mut res = [0u64; 64];
    for king_sq in 0..64 {
        let zone = 1u64 << king_sq | KING_ATTACKS[king_sq];
        res[king_sq] = zone | north_one(zone);
    }
    res
}

pub fn initialize_bishop_rays() -> [[u64; 64]; 64] {
    let mut res = [[0u64; 64]; 64];
    for king_sq in 0..64 {
        for bishop_sq in 0..64 {
            res[king_sq][bishop_sq] =
                get_bishop_ray_slow(FREEFIELD_BISHOP_ATTACKS[king_sq], king_sq, bishop_sq);
        }
    }
    res
}

//Gets the ray of one bishop into a specific direction
pub fn get_bishop_ray_slow(
    bishop_attack_in_all_directions: u64,
    target_square: usize,
    bishop_square: usize,
) -> u64 {
    let diff = target_square as isize - bishop_square as isize;
    let target_rank = target_square / 8;
    let target_file = target_square % 8;
    let bishop_rank = bishop_square / 8;
    let bishop_file = bishop_square % 8;
    if diff > 0 {
        if diff % 9 == 0 {
            FILES_LESS_THAN[target_file]
                & FILES_GREATER_THAN[bishop_file]
                & RANKS_LESS_THAN[target_rank]
                & RANKS_GREATER_THAN[bishop_rank]
                & bishop_attack_in_all_directions
        } else {
            FILES_GREATER_THAN[target_file]
                & FILES_LESS_THAN[bishop_file]
                & RANKS_LESS_THAN[target_rank]
                & RANKS_GREATER_THAN[bishop_rank]
                & bishop_attack_in_all_directions
        }
    } else if diff % -9 == 0 {
        FILES_GREATER_THAN[target_file]
            & FILES_LESS_THAN[bishop_file]
            & RANKS_GREATER_THAN[target_rank]
            & RANKS_LESS_THAN[bishop_rank]
            & bishop_attack_in_all_directions
    } else {
        FILES_LESS_THAN[target_file]
            & FILES_GREATER_THAN[bishop_file]
            & RANKS_GREATER_THAN[target_rank]
            & RANKS_LESS_THAN[bishop_rank]
            & bishop_attack_in_all_directions
    }
}

pub fn initialize_rook_rays() -> [[u64; 64]; 64] {
    let mut res = [[0u64; 64]; 64];
    for king_sq in 0..64 {
        for rook_sq in 0..64 {
            res[king_sq][rook_sq] =
                get_rook_ray_slow(FREEFIELD_ROOK_ATTACKS[king_sq], king_sq, rook_sq);
        }
    }
    res
}

//Gets the ray of one rook into a specific direction
pub fn get_rook_ray_slow(
    rook_attacks_in_all_directions: u64,
    target_square: usize,
    rook_square: usize,
) -> u64 {
    let diff = target_square as isize - rook_square as isize;
    let target_rank = target_square / 8;
    let target_file = target_square % 8;
    let rook_rank = rook_square / 8;
    let rook_file = rook_square % 8;
    if diff > 0 {
        //Same vertical
        if target_rank == rook_rank {
            FILES_LESS_THAN[target_file]
                & FILES_GREATER_THAN[rook_file]
                & rook_attacks_in_all_directions
        } else {
            RANKS_LESS_THAN[target_rank]
                & RANKS_GREATER_THAN[rook_rank]
                & rook_attacks_in_all_directions
        }
    } else if target_rank == rook_rank {
        FILES_GREATER_THAN[target_file]
            & FILES_LESS_THAN[rook_file]
            & rook_attacks_in_all_directions
    } else {
        RANKS_GREATER_THAN[target_rank]
            & RANKS_LESS_THAN[rook_rank]
            & rook_attacks_in_all_directions
    }
}

pub fn initialize_freefield_rook_attacks() -> [u64; 64] {
    let mut res = [0u64; 64];
    for (sq, item) in res.iter_mut().enumerate() {
        *item = rook_attack(sq, 0u64);
    }
    res
}

pub fn initialize_freefield_bishop_attacks() -> [u64; 64] {
    let mut res = [0u64; 64];
    for (sq, item) in res.iter_mut().enumerate() {
        *item = bishop_attack(sq, 0u64);
    }
    res
}

pub fn initialize_inner_center() -> u64 {
    (FILES[3] | FILES[4]) & (RANKS[3] | RANKS[4])
}

pub fn initialize_center() -> u64 {
    (FILES[1] | FILES[2] | FILES[3] | FILES[4] | FILES[5] | FILES[6])
        & (RANKS[1] | RANKS[2] | RANKS[3] | RANKS[4] | RANKS[5] | RANKS[6])
}

pub fn initialize_files() -> [u64; 8] {
    let mut res = [0u64; 8];
    for file in 0..8 {
        if file == 0 {
            res[0] = 1u64
                | 1u64 << 8
                | 1u64 << 16
                | 1u64 << 24
                | 1u64 << 32
                | 1u64 << 40
                | 1u64 << 48
                | 1u64 << 56;
        } else {
            res[file] = res[file - 1] << 1;
        }
    }
    log("Finished Initializing Files!");
    res
}

pub fn initialize_not_files() -> [u64; 8] {
    let mut res = [0u64; 8];
    for file in 0..8 {
        res[file] = !FILES[file];
    }
    log("Finished Initializing NOT Files!");
    res
}

pub fn initialize_ranks() -> [u64; 8] {
    let mut res = [0u64; 8];
    for rank in 0..8 {
        if rank == 0 {
            res[0] = 1u64
                | 1u64 << 1
                | 1u64 << 2
                | 1u64 << 3
                | 1u64 << 4
                | 1u64 << 5
                | 1u64 << 6
                | 1u64 << 7;
        } else {
            res[rank] = res[rank - 1] << 8;
        }
    }
    log("Finished Initializing Ranks!");
    res
}

pub fn initialize_squares() -> [u64; 64] {
    let mut res = [0u64; 64];
    for (squares, item) in res.iter_mut().enumerate() {
        *item = 1u64 << squares;
    }
    log("Finished Initializing Squares!");
    res
}

pub fn initialize_not_squares() -> [u64; 64] {
    let mut res = [0u64; 64];
    for (squares, item) in res.iter_mut().enumerate() {
        *item = !(1u64 << squares);
    }
    log("Finished Initializing NOT_Squares!");
    res
}

pub fn nort_fill(mut gen: u64) -> u64 {
    gen |= gen << 8;
    gen |= gen << 16;
    gen |= gen << 32;
    gen
}

pub fn sout_fill(mut gen: u64) -> u64 {
    gen |= gen >> 8;
    gen |= gen >> 16;
    gen |= gen >> 32;
    gen
}

pub fn file_fill(gen: u64) -> u64 {
    nort_fill(gen) | sout_fill(gen)
}

pub fn w_front_span(wpawns: u64) -> u64 {
    north_one(nort_fill(wpawns))
}

pub fn b_front_span(bpawns: u64) -> u64 {
    south_one(sout_fill(bpawns))
}

pub fn w_rear_span(wpawns: u64) -> u64 {
    south_one(sout_fill(wpawns))
}

pub fn b_rear_span(bpawns: u64) -> u64 {
    north_one(nort_fill(bpawns))
}

#[inline(always)]
pub fn north_one(board: u64) -> u64 {
    board << 8
}

#[inline(always)]
pub fn north_east_one(board: u64) -> u64 {
    (board & NOT_FILES[7]) << 9
}

#[inline(always)]
pub fn north_west_one(board: u64) -> u64 {
    (board & NOT_FILES[0]) << 7
}

#[inline(always)]
pub fn south_one(board: u64) -> u64 {
    board >> 8
}

#[inline(always)]
pub fn south_east_one(board: u64) -> u64 {
    (board & NOT_FILES[7]) >> 7
}

#[inline(always)]
pub fn south_west_one(board: u64) -> u64 {
    (board & NOT_FILES[0]) >> 9
}

#[inline(always)]
pub fn west_one(board: u64) -> u64 {
    (board & NOT_FILES[0]) >> 1
}

#[inline(always)]
pub fn east_one(board: u64) -> u64 {
    (board & NOT_FILES[7]) << 1
}

pub fn king_attack(mut king_board: u64) -> u64 {
    let mut attacks = east_one(king_board) | west_one(king_board);
    king_board |= attacks;
    attacks |= south_one(king_board) | north_one(king_board);
    attacks
}

pub fn init_king_attacks() -> [u64; 64] {
    let mut res = [0u64; 64];
    for (square, item) in res.iter_mut().enumerate() {
        *item = king_attack(1u64 << square);
    }
    log("Finished Initializing King Attacks!");
    res
}

pub fn knight_attack(knight: u64) -> u64 {
    let mut attacks;
    let mut east = east_one(knight);
    let mut west = west_one(knight);
    attacks = (east | west) << 16;
    attacks |= (east | west) >> 16;
    east = east_one(east);
    west = west_one(west);
    attacks |= (east | west) << 8;
    attacks |= (east | west) >> 8;
    attacks
}

pub fn init_knight_attacks() -> [u64; 64] {
    let mut res = [0u64; 64];
    for (square, item) in res.iter_mut().enumerate() {
        *item = knight_attack(1u64 << square);
    }
    log("Finished Initializing Knight Attacks!");
    res
}

pub fn init_files_less_than() -> [u64; 8] {
    let mut res = [0u64; 8];
    for (files, item) in res.iter_mut().enumerate() {
        for files_less_than in 0..files {
            *item |= FILES[files_less_than];
        }
    }
    log("Finished Initializing FilesLessThan!");
    res
}

pub fn init_ranks_less_than() -> [u64; 8] {
    let mut res = [0u64; 8];
    for (ranks, item) in res.iter_mut().enumerate() {
        for ranks_less_than in 0..ranks {
            *item |= RANKS[ranks_less_than];
        }
    }
    log("Finished Initializing RanksLessThan!");
    res
}

pub fn init_files_greater_than() -> [u64; 8] {
    let mut res = [0u64; 8];
    for files in 0..8 {
        res[files] = !FILES_LESS_THAN[files] & !FILES[files];
    }
    log("Finished Initializing FilesGreaterThan!");
    res
}

pub fn init_ranks_greater_than() -> [u64; 8] {
    let mut res = [0u64; 8];
    for ranks in 0..8 {
        res[ranks] = !RANKS_LESS_THAN[ranks] & !RANKS[ranks];
    }
    log("Finished Initializing FilesGreaterThan!");
    res
}

pub fn init_upper_half() -> u64 {
    RANKS_GREATER_THAN[3]
}

pub fn init_lower_half() -> u64 {
    RANKS_LESS_THAN[4]
}

pub fn init_diagonally_adjacent() -> [u64; 64] {
    let mut res = [0u64; 64];
    for (sq, item) in res.iter_mut().enumerate() {
        let board = 1u64 << sq;
        *item = north_east_one(board)
            | north_west_one(board)
            | south_east_one(board)
            | south_west_one(board);
    }
    log("Finished Initializing Diagonally Adjacent Board!");
    res
}

pub fn init_shielding_pawns_white() -> [u64; 64] {
    let mut res = [0u64; 64];
    for (sq, item) in res.iter_mut().enumerate() {
        let king = 1u64 << sq;
        let shield = king << 8 | north_west_one(king) | north_east_one(king);
        *item = shield | shield << 8;
    }
    for rank in 0..8 {
        res[8 * rank] = res[8 * rank + 1];
        res[8 * rank + 7] = res[8 * rank + 6];
    }
    log("Finished Initializing Shielding PawnsWhite Board!");
    res
}

pub fn init_shielding_pawns_black() -> [u64; 64] {
    let mut res = [0u64; 64];
    for (sq, item) in res.iter_mut().enumerate() {
        let king = 1u64 << sq;
        let shield = king >> 8 | south_west_one(king) | south_east_one(king);
        *item = shield | shield >> 8;
    }
    for rank in 0..8 {
        res[8 * rank] = res[8 * rank + 1];
        res[8 * rank + 7] = res[8 * rank + 6];
    }
    log("Finished Initializing Shielding PawnsBlack Board!");
    res
}
