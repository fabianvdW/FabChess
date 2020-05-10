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
        ATTACKS[self.offset + apply_magic(self.magic, occ & self.occupancy_mask, self.shift)]
    }
}
impl Display for Magic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res_str = String::new();
        res_str.push_str(&format!(
            "Magic{{occupancy_mask: {}, shift: {}, magic: {}, offset: {}}}",
            self.occupancy_mask, self.shift, self.magic, self.offset
        ));
        write!(f, "{}", res_str)
    }
}
#[inline(always)]
pub fn apply_magic(magic: u64, bb: u64, bits: usize) -> usize {
    (bb.wrapping_mul(magic) >> (64 - bits)) as usize
}
