use crate::logging::log;
use rand::prelude::*;
use std::u64;

pub fn rand_u64() -> u64 {
    let mut rng = rand::thread_rng();
    let res: u64 = rng.gen();
    res
}
lazy_static! {
    pub static ref ZOBRIST_KEYS: Zobrist = init_zobrist();
}
pub fn init_at_program_start() {
    log(&format!("{} ", ZOBRIST_KEYS.side_to_move));
    log("Initialized Zobrist Keys!");
}

pub fn rand_array_64() -> [u64; 64] {
    let mut res = [0u64; 64];
    for i in 0..64 {
        res[i] = rand_u64();
    }
    res
}

pub fn rand_array_8() -> [u64; 8] {
    let mut res = [0u64; 8];
    for i in 0..8 {
        res[i] = rand_u64();
    }
    res
}

pub fn init_zobrist() -> Zobrist {
    Zobrist {
        w_pawns: rand_array_64(),
        w_knights: rand_array_64(),
        w_bishops: rand_array_64(),
        w_rooks: rand_array_64(),
        w_queens: rand_array_64(),
        w_king: rand_array_64(),
        b_pawns: rand_array_64(),
        b_knights: rand_array_64(),
        b_bishops: rand_array_64(),
        b_rooks: rand_array_64(),
        b_queens: rand_array_64(),
        b_king: rand_array_64(),
        side_to_move: rand_u64(),
        castle_w_kingside: rand_u64(),
        castle_w_queenside: rand_u64(),
        castle_b_kingside: rand_u64(),
        castle_b_queenside: rand_u64(),
        en_passant: rand_array_8(),
    }
}

pub struct Zobrist {
    pub w_pawns: [u64; 64],
    pub w_knights: [u64; 64],
    pub w_bishops: [u64; 64],
    pub w_rooks: [u64; 64],
    pub w_queens: [u64; 64],
    pub w_king: [u64; 64],
    pub b_pawns: [u64; 64],
    pub b_knights: [u64; 64],
    pub b_bishops: [u64; 64],
    pub b_rooks: [u64; 64],
    pub b_queens: [u64; 64],
    pub b_king: [u64; 64],
    pub side_to_move: u64,
    pub castle_w_kingside: u64,
    pub castle_w_queenside: u64,
    pub castle_b_kingside: u64,
    pub castle_b_queenside: u64,
    pub en_passant: [u64; 8],
}
