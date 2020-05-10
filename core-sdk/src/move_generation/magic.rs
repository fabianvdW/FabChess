use crate::bitboards::bitboards::magic_constants::*;
include!(concat!(env!("OUT_DIR"), "/magic_attacks.rs"));

use std::fmt::Display;
#[derive(Clone)]
pub struct Magic {
    pub occupancy_mask: u64,
    pub shift: usize,
    pub magic: u64,
    pub offset: usize,
}
impl Magic {
    #[inline(always)]
    pub fn bishop(square: usize, occ: u64) -> u64 {
        MAGIC_BISHOP[square].apply(occ)
    }

    #[inline(always)]
    pub fn rook(square: usize, occ: u64) -> u64 {
        MAGIC_ROOK[square].apply(occ)
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    #[inline(always)]
    pub fn apply(&self, occ: u64) -> u64 {
        use std::arch::x86_64::_pext_u64;
        ATTACKS[self.offset + unsafe { _pext_u64(occ, self.occupancy_mask) } as usize]
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
    #[inline(always)]
    pub fn apply(&self, occ: u64) -> u64 {
        self.lookup[apply_magic(self.magic, occ & self.occupancy_mask, self.shift)]
    }
}
pub fn initialize_magics() {
    initialize_bishop_magics();
    initialize_rook_magics();
}
pub fn initialize_rook_magics() {
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
    unsafe { MAGIC_ROOK = res }
}
pub fn initialize_bishop_magics() {
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
    unsafe { MAGIC_BISHOP = res }
}
pub fn fill_table<F: Fn(u64) -> usize>(pattern: &Vec<(u64, u64)>, f: F) -> Option<Vec<u64>> {
#[inline(always)]
pub fn apply_magic(magic: u64, bb: u64, bits: usize) -> usize {
    (bb.wrapping_mul(magic) >> (64 - bits)) as usize
}
