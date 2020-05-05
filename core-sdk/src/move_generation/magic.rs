use crate::bitboards::arr_to_string;
use crate::bitboards::bitboards::magic_constants::{
    MAGICS_BISHOP, MAGICS_ROOK, OCCUPANCY_MASKS_BISHOP, OCCUPANCY_MASKS_ROOK,
};
use rand::RngCore;
lazy_static! {
    pub static ref MAGIC_ROOK: Vec<Magic> = initialize_rook_magics();
    pub static ref MAGIC_BISHOP: Vec<Magic> = initialize_bishop_magics();
}
pub fn init_magics() {
    MAGIC_ROOK.len();
    MAGIC_BISHOP.len();
}
#[derive(Clone)]
pub struct Magic {
    pub occupancy_mask: u64,
    pub shift: usize,
    pub magic: u64,
    pub lookup: Vec<u64>,
}
impl Magic {
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    #[inline(always)]
    pub fn apply(&self, occ: u64) -> u64 {
        use std::arch::x86_64::_pext_u64;
        self.lookup[unsafe { _pext_u64(occ, self.occupancy_mask) } as usize]
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
    #[inline(always)]
    pub fn apply(&self, occ: u64) -> u64 {
        self.lookup[apply_magic(self.magic, occ & self.occupancy_mask, self.shift)]
    }
}

pub fn initialize_rook_magics() -> Vec<Magic> {
    let mut res = Vec::with_capacity(0);
    for sq in 0..64 {
        let patterns = generate_rook_patterns(sq);
        #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
        {
            let lookup = fill_table(&patterns, |bb| unsafe {
                std::arch::x86_64::_pext_u64(bb, OCCUPANCY_MASKS_ROOK[sq]) as usize
            })
            .unwrap();
            res.push(Magic {
                occupancy_mask: OCCUPANCY_MASKS_ROOK[sq],
                magic: MAGICS_ROOK[sq],
                shift: OCCUPANCY_MASKS_ROOK[sq].count_ones() as usize,
                lookup,
            });
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
        {
            let lookup = fill_table(&patterns, |bb| {
                apply_magic(
                    MAGICS_ROOK[sq],
                    bb,
                    OCCUPANCY_MASKS_ROOK[sq].count_ones() as usize,
                )
            })
            .unwrap();
            res.push(Magic {
                occupancy_mask: OCCUPANCY_MASKS_ROOK[sq],
                magic: MAGICS_ROOK[sq],
                shift: OCCUPANCY_MASKS_ROOK[sq].count_ones() as usize,
                lookup,
            });
        }
    }
    res
}
pub fn initialize_bishop_magics() -> Vec<Magic> {
    let mut res = Vec::with_capacity(0);
    for sq in 0..64 {
        let patterns = generate_bishop_patterns(sq);
        #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
        {
            let lookup = fill_table(&patterns, |bb| unsafe {
                std::arch::x86_64::_pext_u64(bb, OCCUPANCY_MASKS_BISHOP[sq]) as usize
            })
            .unwrap();
            res.push(Magic {
                occupancy_mask: OCCUPANCY_MASKS_BISHOP[sq],
                magic: MAGICS_BISHOP[sq],
                shift: OCCUPANCY_MASKS_BISHOP[sq].count_ones() as usize,
                lookup,
            });
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
        {
            let lookup = fill_table(&patterns, |bb| {
                apply_magic(
                    MAGICS_BISHOP[sq],
                    bb,
                    OCCUPANCY_MASKS_BISHOP[sq].count_ones() as usize,
                )
            })
            .unwrap();
            res.push(Magic {
                occupancy_mask: OCCUPANCY_MASKS_BISHOP[sq],
                magic: MAGICS_BISHOP[sq],
                shift: OCCUPANCY_MASKS_BISHOP[sq].count_ones() as usize,
                lookup,
            });
        }
    }
    res
}
pub fn fill_table<F: Fn(u64) -> usize>(pattern: &Vec<(u64, u64)>, f: F) -> Option<Vec<u64>> {
    let mut result = vec![std::u64::MAX; pattern.len()];
    for pattern in pattern {
        let index = f(pattern.0);
        if result[index] == std::u64::MAX || result[index] == pattern.1 {
            result[index] = pattern.1;
        } else {
            return None;
        }
    }
    Some(result)
}
pub fn generate_single_magic(pattern: &Vec<(u64, u64)>) -> u64 {
    let mut rand = rand::thread_rng();
    let bits = pattern.len().trailing_zeros() as usize;
    loop {
        let magic = rand.next_u64() & rand.next_u64() & rand.next_u64();
        if fill_table(pattern, |bb| apply_magic(magic, bb, bits)).is_some() {
            return magic;
        }
    }
}

#[inline(always)]
pub fn apply_magic(magic: u64, bb: u64, bits: usize) -> usize {
    (bb.wrapping_mul(magic) >> (64 - bits)) as usize
}
pub fn pdep(mut mask: u64, temp: u64) -> u64 {
    let mut res = 0u64;
    let mut temp_index = 0;
    while mask > 0u64 {
        let idx = mask.trailing_zeros();
        if (temp & (1 << temp_index)) > 0 {
            res |= 1 << idx;
        }
        temp_index += 1;
        mask ^= 1 << idx;
    }
    res
}

pub fn print_rook_magics() {
    let mut magics = [0u64; 64];
    for sq in 0..64 {
        let pattern = generate_rook_patterns(sq);
        magics[sq] = generate_single_magic(&pattern);
    }
    println!("{}", arr_to_string(&magics, "MAGICS_ROOK"));
}
pub fn generate_rook_patterns(square: usize) -> Vec<(u64, u64)> {
    let occupancy_mask = OCCUPANCY_MASKS_ROOK[square];
    let patterns = 1 << occupancy_mask.count_ones();
    let mut pattern_res = Vec::with_capacity(patterns);
    for i in 0..patterns {
        let actual_occ = pdep(occupancy_mask, i as u64);
        let actual_attacks = rook_attacks_slow(square, actual_occ);
        pattern_res.push((actual_occ, actual_attacks));
    }
    pattern_res
}
pub fn rook_attacks_slow(square: usize, blocks: u64) -> u64 {
    let mut res = 0u64;
    let rank: isize = (square / 8) as isize;
    let file: isize = (square % 8) as isize;
    let dirs: [(isize, isize); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
    for dir in dirs.iter() {
        let (file_i, rank_i) = dir;
        let mut rn = rank + rank_i;
        let mut fnn = file + file_i;
        while rn >= 0 && rn <= 7 && fnn >= 0 && fnn <= 7 {
            res |= 1u64 << (rn * 8 + fnn);
            if (blocks & (1u64 << (rn * 8 + fnn))) != 0 {
                break;
            }
            rn += rank_i;
            fnn += file_i;
        }
    }
    res
}

pub fn print_bishop_magics() {
    let mut magics = [0u64; 64];
    for sq in 0..64 {
        let pattern = generate_bishop_patterns(sq);
        magics[sq] = generate_single_magic(&pattern);
    }
    println!("{}", arr_to_string(&magics, "MAGICS_BISHOP"))
}
pub fn generate_bishop_patterns(square: usize) -> Vec<(u64, u64)> {
    let occupancy_mask = OCCUPANCY_MASKS_BISHOP[square];
    let patterns = 1 << occupancy_mask.count_ones();
    let mut pattern_res = Vec::with_capacity(patterns);
    for i in 0..patterns {
        let actual_occ = pdep(occupancy_mask, i as u64);
        let actual_attacks = bishop_attacks_slow(square, actual_occ);
        pattern_res.push((actual_occ, actual_attacks))
    }
    pattern_res
}
pub fn bishop_attacks_slow(square: usize, blocks: u64) -> u64 {
    let mut res = 0u64;
    let rank: isize = (square / 8) as isize;
    let file: isize = (square % 8) as isize;
    let dirs: [(isize, isize); 4] = [(1, 1), (-1, -1), (1, -1), (-1, 1)];
    for dir in dirs.iter() {
        let (file_i, rank_i) = dir;
        let mut rn = rank + rank_i;
        let mut fnn = file + file_i;
        while rn >= 0 && rn <= 7 && fnn >= 0 && fnn <= 7 {
            res |= 1u64 << (rn * 8 + fnn);
            if (blocks & (1u64 << (rn * 8 + fnn))) != 0 {
                break;
            }
            rn += rank_i;
            fnn += file_i;
        }
    }
    res
}
