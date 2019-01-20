lazy_static! {
    pub static ref FILES: [u64;8] = initialize_files();
    pub static ref NOT_FILES: [u64;8] = initialize_not_files();
    pub static ref RANKS: [u64;8] = initialize_ranks();
    pub static ref SQUARES: [u64;64]= initialize_squares();
    pub static ref NOT_SQUARES: [u64;64]= initialize_not_squares();
    pub static ref KING_ATTACKS:[u64;64] = init_king_attacks();
    pub static ref KNIGHT_ATTACKS:[u64;64] = init_knight_attacks();
    pub static ref FILES_LESS_THAN:[u64;8]= init_files_less_than();
    pub static ref FILES_GREATER_THAN:[u64;8]= init_files_greater_than();
    pub static ref RANKS_LESS_THAN:[u64;8]= init_ranks_less_than();
    pub static ref RANKS_GREATER_THAN:[u64;8]= init_ranks_greater_than();
    pub static ref UPPER_HALF:u64 = init_upper_half();
    pub static ref LOWER_HALF:u64 = init_lower_half();
    pub static ref DIAGONALLY_ADJACENT:[u64;64] = init_diagonally_adjacent();
    pub static ref SHIELDING_PAWNS_WHITE:[u64;64]= init_shielding_pawns_white();
    pub static ref SHIELDING_PAWNS_BLACK:[u64;64]= init_shielding_pawns_black();
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
}
//Initializing General BitBoards
pub fn initialize_files() -> [u64; 8] {
    let mut res = [0u64; 8];
    for file in 0..8 {
        if file == 0 {
            res[0] = 1u64 << 0 | 1u64 << 8 | 1u64 << 16 | 1u64 << 24 | 1u64 << 32 | 1u64 << 40 | 1u64 << 48 | 1u64 << 56;
        } else {
            res[file] = res[file - 1] << 1;
        }
    }
    println!("Finished Initializing Files!");
    res
}

pub fn initialize_not_files() -> [u64; 8] {
    let mut res = [0u64; 8];
    for file in 0..8 {
        res[file] = !FILES[file];
    }
    println!("Finished Initializing NOT Files!");
    res
}

pub fn initialize_ranks() -> [u64; 8] {
    let mut res = [0u64; 8];
    for rank in 0..8 {
        if rank == 0 {
            res[0] = 1u64 << 0 | 1u64 << 1 | 1u64 << 2 | 1u64 << 3 | 1u64 << 4 | 1u64 << 5 | 1u64 << 6 | 1u64 << 7;
        } else {
            res[rank] = res[rank - 1] << 8;
        }
    }
    println!("Finished Initializing Ranks!");
    res
}

pub fn initialize_squares() -> [u64; 64] {
    let mut res = [0u64; 64];
    for squares in 0..64 {
        res[squares] = 1u64 << squares;
    }
    println!("Finished Initializing Squares!");
    res
}

pub fn initialize_not_squares() -> [u64; 64] {
    let mut res = [0u64; 64];
    for squares in 0..64 {
        res[squares] = !(1u64 << squares);
    }
    println!("Finished Initializing NOT_Squares!");
    res
}

pub fn nort_fill(mut gen:u64) -> u64{
    gen |= gen<<8;
    gen |= gen<<16;
    gen |= gen<<32;
    gen
}

pub fn sout_fill(mut gen:u64) -> u64{
    gen |= gen>>8;
    gen |= gen>>16;
    gen |= gen>>32;
    gen
}

pub fn file_fill(gen:u64)->u64{
    nort_fill(gen)|sout_fill(gen)
}

pub fn w_front_span(wpawns:u64)->u64{
    north_one(nort_fill(wpawns))
}

pub fn b_front_span(bpawns:u64)->u64{
    south_one(sout_fill(bpawns))
}

pub fn w_rear_span(wpawns:u64)->u64{
    south_one(sout_fill(wpawns))
}

pub fn b_rear_span(bpawns:u64)->u64{
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
    for square in 0..64 {
        res[square] = king_attack(1u64 << square);
    }
    println!("Finished Initializing King Attacks!");
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
    for square in 0..64 {
        res[square] = knight_attack(1u64 << square);
    }
    println!("Finished Initializing Knight Attacks!");
    res
}

pub fn init_files_less_than() -> [u64; 8] {
    let mut res = [0u64; 8];
    for files in 0..8 {
        for files_less_than in 0..files {
            res[files] |= FILES[files_less_than];
        }
    }
    println!("Finished Initializing FilesLessThan!");
    res
}

pub fn init_ranks_less_than() -> [u64; 8] {
    let mut res = [0u64; 8];
    for ranks in 0..8 {
        for ranks_less_than in 0..ranks {
            res[ranks] |= RANKS[ranks_less_than];
        }
    }
    println!("Finished Initializing RanksLessThan!");
    res
}

pub fn init_files_greater_than() -> [u64; 8] {
    let mut res = [0u64; 8];
    for files in 0..8 {
        res[files] = !FILES_LESS_THAN[files] & !FILES[files];
    }
    println!("Finished Initializing FilesGreaterThan!");
    res
}

pub fn init_ranks_greater_than() -> [u64; 8] {
    let mut res = [0u64; 8];
    for ranks in 0..8 {
        res[ranks] = !RANKS_LESS_THAN[ranks] & !RANKS[ranks];
    }
    println!("Finished Initializing FilesGreaterThan!");
    res
}

pub fn init_upper_half() -> u64{
    RANKS_GREATER_THAN[3]
}

pub fn init_lower_half() -> u64{
    RANKS_LESS_THAN[4]
}

pub fn init_diagonally_adjacent() ->[u64;64]{
    let mut res  =[0u64;64];
    for sq in 0..64{
        let board = 1u64<<sq;
        res[sq]= north_east_one(board)|north_west_one(board)|south_east_one(board)|south_west_one(board);
    }
    println!("Finished Initializing Diagonally Adjacent Board!");
    res
}

pub fn init_shielding_pawns_white() -> [u64;64]{
    let mut res = [0u64;64];
    for sq in 0..64{
        let king = 1u64<<sq;
        let shield= king<<8|north_west_one(king)|north_east_one(king);
        res[sq]= shield|shield<<8;
    }
    println!("Finished Initializing Shielding PawnsWhite Board!");
    res
}

pub fn init_shielding_pawns_black() -> [u64;64]{
    let mut res = [0u64;64];
    for sq in 0..64{
        let king = 1u64<<sq;
        let shield= king>>8|south_west_one(king)|south_east_one(king);
        res[sq]= shield|shield>>8;
    }
    println!("Finished Initializing Shielding PawnsBlack Board!");
    res
}
