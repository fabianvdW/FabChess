use rand::RngCore;
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::Path;
#[rustfmt::skip]
pub const OCCUPANCY_MASKS_ROOK : [u64;64] = [282578800148862u64, 565157600297596u64, 1130315200595066u64, 2260630401190006u64, 4521260802379886u64, 9042521604759646u64, 18085043209519166u64, 36170086419038334u64, 282578800180736u64, 565157600328704u64, 1130315200625152u64, 2260630401218048u64, 4521260802403840u64, 9042521604775424u64, 18085043209518592u64, 36170086419037696u64, 282578808340736u64, 565157608292864u64, 1130315208328192u64, 2260630408398848u64, 4521260808540160u64, 9042521608822784u64, 18085043209388032u64, 36170086418907136u64, 282580897300736u64, 565159647117824u64, 1130317180306432u64, 2260632246683648u64, 4521262379438080u64, 9042522644946944u64, 18085043175964672u64, 36170086385483776u64, 283115671060736u64, 565681586307584u64, 1130822006735872u64, 2261102847592448u64, 4521664529305600u64, 9042787892731904u64, 18085034619584512u64, 36170077829103616u64, 420017753620736u64, 699298018886144u64, 1260057572672512u64, 2381576680245248u64, 4624614895390720u64, 9110691325681664u64, 18082844186263552u64, 36167887395782656u64, 35466950888980736u64, 34905104758997504u64, 34344362452452352u64, 33222877839362048u64, 30979908613181440u64, 26493970160820224u64, 17522093256097792u64, 35607136465616896u64, 9079539427579068672u64, 8935706818303361536u64, 8792156787827803136u64, 8505056726876686336u64, 7930856604974452736u64, 6782456361169985536u64, 4485655873561051136u64, 9115426935197958144u64, ];
#[rustfmt::skip]
pub const OCCUPANCY_MASKS_BISHOP : [u64;64] = [18049651735527936u64, 70506452091904u64, 275415828992u64, 1075975168u64, 38021120u64, 8657588224u64, 2216338399232u64, 567382630219776u64, 9024825867763712u64, 18049651735527424u64, 70506452221952u64, 275449643008u64, 9733406720u64, 2216342585344u64, 567382630203392u64, 1134765260406784u64, 4512412933816832u64, 9024825867633664u64, 18049651768822272u64, 70515108615168u64, 2491752130560u64, 567383701868544u64, 1134765256220672u64, 2269530512441344u64, 2256206450263040u64, 4512412900526080u64, 9024834391117824u64, 18051867805491712u64, 637888545440768u64, 1135039602493440u64, 2269529440784384u64, 4539058881568768u64, 1128098963916800u64, 2256197927833600u64, 4514594912477184u64, 9592139778506752u64, 19184279556981248u64, 2339762086609920u64, 4538784537380864u64, 9077569074761728u64, 562958610993152u64, 1125917221986304u64, 2814792987328512u64, 5629586008178688u64, 11259172008099840u64, 22518341868716544u64, 9007336962655232u64, 18014673925310464u64, 2216338399232u64, 4432676798464u64, 11064376819712u64, 22137335185408u64, 44272556441600u64, 87995357200384u64, 35253226045952u64, 70506452091904u64, 567382630219776u64, 1134765260406784u64, 2832480465846272u64, 5667157807464448u64, 11333774449049600u64, 22526811443298304u64, 9024825867763712u64, 18049651735527936u64, ];
#[rustfmt::skip]
pub const MAGICS_BISHOP : [u64;64] = [9052302183530624u64, 3493106745918750722u64, 10378547575765598208u64, 1737267960881348624u64, 10173093901303832u64, 4666011823880819232u64, 595602155869570050u64, 4611897984056627264u64, 36249008850862404u64, 2216337449216u64, 2305851882628841472u64, 184651999957483520u64, 7494011856613818624u64, 1197984168606171392u64, 2256765064877074u64, 147774504575173632u64, 9232379519711904000u64, 1589780154182344962u64, 5843420671266299912u64, 2306970043015012930u64, 291610284032786432u64, 1412881035952660u64, 18577349571281920u64, 288265571328395280u64, 20398418977359873u64, 4616194017980600448u64, 2308105804345245712u64, 4611826893489045536u64, 9009398294841476u64, 2634606881531924610u64, 283674285703424u64, 1261300437177876736u64, 19333830213640194u64, 9225705209122014272u64, 36314674337548288u64, 5188148971919900801u64, 16289522094180425736u64, 81082939529527360u64, 5198622926656012808u64, 9656916352225543296u64, 2261180160746545u64, 40818338457190912u64, 1152932510729241088u64, 148919646538486784u64, 10134203572167168u64, 1135797138883072u64, 164383759939144704u64, 9233225930963681536u64, 100207325126067208u64, 1153207386539033088u64, 4611361466745472u64, 57139560060289058u64, 288248037091186432u64, 1301584865623408704u64, 75611525158570019u64, 146384586526490896u64, 1164255287713071617u64, 288338171259344900u64, 5764607534879117377u64, 1157495747864957184u64, 3222077704u64, 4616752605052544032u64, 2343072610411356416u64, 73218686973968530u64, ];
#[rustfmt::skip]
pub const MAGICS_ROOK : [u64;64] = [2630106718943609138u64, 18032010559799296u64, 180161586023891074u64, 2449967268337156128u64, 36037593179127810u64, 1297037861652529664u64, 216173881668150784u64, 144115755014179329u64, 9516246750663278729u64, 2392674749399056u64, 14777779876790404u64, 578853461412548608u64, 36169551687712896u64, 4925820690762752u64, 422225358362112u64, 10387834016590004226u64, 468374636126535876u64, 2305918051150733312u64, 1153062792119508996u64, 40532946536465424u64, 5770519597325746180u64, 9223662312756613184u64, 36103566096597521u64, 9228176902740050052u64, 1242995973202911360u64, 301811597467189376u64, 3103015342663795328u64, 5944769102463107204u64, 5764629515414798465u64, 3458766714999669760u64, 288232592363292688u64, 290483284066992324u64, 351855003566724u64, 1371381339630076098u64, 2307021687834566656u64, 576496040862028288u64, 2955521640369152u64, 24910690066104832u64, 149602367980033u64, 140738620818688u64, 140738562129952u64, 4620836158493032480u64, 1157636347922546704u64, 4802950260195336u64, 8800388317200u64, 297959129979814176u64, 9017713502715912u64, 360429292935315457u64, 2306267730627658240u64, 666534181534443776u64, 360596933493932288u64, 288250168435319296u64, 7036908795364608u64, 2307531895849878016u64, 864708755556017152u64, 11608168789920731776u64, 144255964230459459u64, 4719808153548554754u64, 36117037123772417u64, 4756118072021484801u64, 581245895669196801u64, 563037070164226u64, 4684025104663969825u64, 2256199512819778u64, ];
pub const ATTACKS_SIZE: usize = 107648;
pub fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let has_bmi2 = env::var("CARGO_CFG_TARGET_FEATURE").map_or(false, |x| x.contains("bmi2"));
    let magic_path = Path::new(&out_dir).join("magic_attacks.rs");
    let mut file = File::create(magic_path).unwrap();
    if has_bmi2 {
        write!(file, "{}\n", "//Tables for BMI2").unwrap();
    } else {
        write!(file, "{}\n", "//Tables for magics").unwrap();
    }
    //Rook magics
    let mut attacks = [0u64; ATTACKS_SIZE];
    let mut previous_offset = 0;
    for sq in 0..64 {
        let patterns = generate_rook_patterns(sq);
        let lookup;
        if has_bmi2 {
            lookup =
                fill_table(&patterns, |bb| pext(bb, OCCUPANCY_MASKS_ROOK[sq]) as usize).unwrap();
        } else {
            lookup = fill_table(&patterns, |bb| {
                apply_magic(
                    MAGICS_ROOK[sq],
                    bb,
                    OCCUPANCY_MASKS_ROOK[sq].count_ones() as usize,
                )
            })
            .unwrap();
        }
        for (i, val) in lookup.iter().enumerate() {
            attacks[previous_offset + i] = *val;
        }
        previous_offset += patterns.len();
    }
    for sq in 0..64 {
        let patterns = generate_bishop_patterns(sq);
        let lookup;
        if has_bmi2 {
            lookup = fill_table(&patterns, |bb| {
                pext(bb, OCCUPANCY_MASKS_BISHOP[sq]) as usize
            })
            .unwrap();
        } else {
            lookup = fill_table(&patterns, |bb| {
                apply_magic(
                    MAGICS_BISHOP[sq],
                    bb,
                    OCCUPANCY_MASKS_BISHOP[sq].count_ones() as usize,
                )
            })
            .unwrap();
        }
        for (i, val) in lookup.iter().enumerate() {
            attacks[previous_offset + i] = *val;
        }
        previous_offset += patterns.len();
    }
    write!(file, "{}", arr_to_string(&attacks, "ATTACKS")).unwrap();
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
pub fn pext(board: u64, mut mask: u64) -> u64 {
    let mut res = 0u64;
    let mut temp_index = 0;
    while mask > 0u64 {
        let idx = mask.trailing_zeros();
        if board & (1 << idx) > 0 {
            res |= 1 << temp_index;
        }
        mask ^= 1 << idx;
        temp_index += 1;
    }
    res
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

pub(crate) fn arr_to_string<T: Display>(arr: &[T], name: &str) -> String {
    let mut res_str: String = String::new();
    res_str.push_str(&format!(
        "#[rustfmt::skip]\npub const {} : [{};{}] = [",
        name,
        std::any::type_name::<T>(),
        arr.len()
    ));
    for i in arr {
        res_str.push_str(&format!("{}{}, ", *i, std::any::type_name::<T>()));
    }
    res_str.push_str("];");
    res_str
}

pub fn print_rook_magic_nums(file: &mut File) {
    let mut magics = [0u64; 64];
    for sq in 0..64 {
        let pattern = generate_rook_patterns(sq);
        magics[sq] = generate_single_magic(&pattern);
    }
    write!(file, "{}", arr_to_string(&magics, "MAGIC_NUMS_ROOK")).unwrap();
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

pub fn print_bishop_magic_nums(file: &mut File) {
    let mut magics = [0u64; 64];
    for sq in 0..64 {
        let pattern = generate_bishop_patterns(sq);
        magics[sq] = generate_single_magic(&pattern);
    }
    write!(file, "{}", arr_to_string(&magics, "MAGIC_NUMS_BISHOP")).unwrap()
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
