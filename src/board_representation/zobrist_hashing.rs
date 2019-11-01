use crate::logging::log;
use rand::prelude::*;
use std::u64;

pub fn rand_u64(rng: &mut StdRng) -> u64 {
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

pub fn rand_array_64(rng: &mut StdRng) -> [u64; 64] {
    let mut res = [0u64; 64];
    for item in res.iter_mut() {
        *item = rand_u64(rng);
    }
    res
}

pub fn rand_array_8(rng: &mut StdRng) -> [u64; 8] {
    let mut res = [0u64; 8];
    for item in res.iter_mut() {
        *item = rand_u64(rng);
    }
    res
}

pub fn init_zobrist() -> Zobrist {
    let mut generator: StdRng = SeedableRng::from_seed([42; 32]);

    Zobrist {
        w_pawns: rand_array_64(&mut generator),
        w_knights: rand_array_64(&mut generator),
        w_bishops: rand_array_64(&mut generator),
        w_rooks: rand_array_64(&mut generator),
        w_queens: rand_array_64(&mut generator),
        w_king: rand_array_64(&mut generator),
        b_pawns: rand_array_64(&mut generator),
        b_knights: rand_array_64(&mut generator),
        b_bishops: rand_array_64(&mut generator),
        b_rooks: rand_array_64(&mut generator),
        b_queens: rand_array_64(&mut generator),
        b_king: rand_array_64(&mut generator),
        side_to_move: rand_u64(&mut generator),
        castle_w_kingside: rand_u64(&mut generator),
        castle_w_queenside: rand_u64(&mut generator),
        castle_b_kingside: rand_u64(&mut generator),
        castle_b_queenside: rand_u64(&mut generator),
        en_passant: rand_array_8(&mut generator),
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
