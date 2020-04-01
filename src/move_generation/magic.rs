use super::super::bitboards::{FILES, NOT_SQUARES, RANKS};
use crate::logging::log;
use rand::Rng;

static ROOK_BITS: [usize; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
];
static BISHOP_BITS: [usize; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
];

const MAGIC_NUMS_ROOKS: [u64; 64] = [
    0x2180_0280_1120_4008u64,
    0x2140_0190_0020_00c0u64,
    0x8480_1000_2000_0b80u64,
    0x0080_1000_8006_1800u64,
    0xc100_0800_0403_0010u64,
    0x0200_0844_3012_0001u64,
    0x1880_1100_0680_0200u64,
    0x0100_0062_0080_4100u64,
    0x4084_8002_8040_0020u64,
    0x8400_4000_5000_2000u64,
    0x000d_0010_2001_0342u64,
    0x0020_0401_0200_8040u64,
    0x8000_8024_0081_0800u64,
    0x2922_0028_0410_0a01u64,
    0x4109_002a_0099_000cu64,
    0x4002_0000_840e_0841u64,
    0x0800_8480_0020_4011u64,
    0x0900_4440_0020_1008u64,
    0x5201_0100_2004_9142u64,
    0x0080_6200_09c2_0030u64,
    0x0045_3100_0408_0100u64,
    0x8080_0200_1400u64,
    0x1090_8400_0210_0328u64,
    0x0080_2200_0184_085bu64,
    0x4010_8002_8020u64,
    0x4910_4000_8080_2000u64,
    0x1400_4031_0020_0100u64,
    0x0910_0080_8018_0450u64,
    0x0200_0400_8080_0800u64,
    0x1052_0002_0010_0429u64,
    0x2004_e80c_000a_1110u64,
    0x1423_0001_0000_5282u64,
    0x8088_8340_0080_0aa0u64,
    0x0100_80c0_0080_6000u64,
    0x8010_2000_8280_5002u64,
    0x0821_0100_3000u64,
    0x7040_0802_8080_0400u64,
    0x610c_0041_0040_1200u64,
    0x5700_0108_0400_1002u64,
    0x1000_0080_4200_0904u64,
    0x2104_9460_c000_8000u64,
    0x0410_0140_2000_4002u64,
    0x0201_00a0_0141_0013u64,
    0x0401_0008_1001_0020u64,
    0x4008_0124_0080_8008u64,
    0x0808_2004_4048_0110u64,
    0x1b81_0402_0001_0100u64,
    0x8020_0400_4082_0003u64,
    0x0080_0021_0850_8100u64,
    0x0102_4208_2b00_8200u64,
    0x0801_0040_9020_0100u64,
    0x4003_0020_0810_0100u64,
    0x0008_1800_8044_0080u64,
    0x890c_0004_800a_0080u64,
    0x0104_0201_0810_2400u64,
    0x0041_1110_4084_0200u64,
    0x1010_2016_0100_8042u64,
    0x0127_0024_4001_3181u64,
    0x0102_1042_0922_0082u64,
    0xa011_0090_0124_2009u64,
    0x0081_0002_2800_1085u64,
    0x5022_0010_0801_1c02u64,
    0x0100_8210_0081_0804u64,
    0x0001_0000_2280_d601u64,
];
const MAGIC_NUMS_BISHOPS: [u64; 64] = [
    0x6040_422a_1408_6080u64,
    0x0004_0848_0040_8020u64,
    0x400c_0802_1044_0010u64,
    0x0044_0420_8000_0000u64,
    0x2610_8820_0210_0a00u64,
    0x9030_080c_0080u64,
    0xc140_8410_4814_8004u64,
    0x0002_0084_4c10_0442u64,
    0x0040_0420_0404_2682u64,
    0x4200_1006_0841_0a28u64,
    0x8020_1005_2a20_2108u64,
    0x800c_0806_0141_0802u64,
    0xc001_0110_4002_0004u64,
    0x0488_8110_0290_2400u64,
    0xa008_0242_1010_6808u64,
    0x1004_a100_e904_2004u64,
    0x2030_0021_02e2_0800u64,
    0x0051_0a58_1021_0042u64,
    0x0068_0104_0808_9830u64,
    0x041c_8008_0204_4200u64,
    0x0004_0040_80a0_4000u64,
    0x4004_0802_0304_0a00u64,
    0x8306_4238_2040u64,
    0x8200_4011_01c2_3040u64,
    0x0008_4002_4810_8110u64,
    0x0004_4400_2028_9080u64,
    0x3008_0404_0800_4050u64,
    0x8004_0040_0401_0002u64,
    0x0921_0040_2404_4016u64,
    0x0201_8408_0202_0a00u64,
    0x2022_2c00_0041_4811u64,
    0x5802_0686_0240_4810u64,
    0x2791_2008_10a2_9848u64,
    0x2650_8210_0008_1000u64,
    0x4150_0048_0322u64,
    0x2041_4018_2006_0200u64,
    0x0001_0202_0024_0104u64,
    0x0001_0802_000c_2212u64,
    0xe002_880a_0000_5200u64,
    0x2001_0622_0040_2100u64,
    0x0052_1050_4400_0850u64,
    0x0204_5908_2080_0818u64,
    0x0001_2014_1008_2a00u64,
    0x0440_0042_0081_0800u64,
    0x0020_0801_0044_2401u64,
    0x200b_2002_1410_0880u64,
    0x2810_1081_0040_0100u64,
    0x8824_6400_5201_1048u64,
    0x4281_0401_1440_0060u64,
    0x9888_2404_0208_001du64,
    0x8060_81c4_0c04_0909u64,
    0x0020_0900_8411_0014u64,
    0x0304_0060_2244_0118u64,
    0x1011_4003_0401_0004u64,
    0x0a10_2001_0410_a048u64,
    0x8110_0208_0900_2809u64,
    0x4001_0022_100c_0413u64,
    0x2800_0201_2501_1014u64,
    0x4608_2400u64,
    0x0060_4084_0084_0401u64,
    0x8001_0200_1002_1204u64,
    0x4c00_1008_1208_4200u64,
    0x2000_0420_440c_1098u64,
    0x0802_200c_0112_0060u64,
];

lazy_static! {
    pub static ref MAGICS_ROOKS: Vec<Magic> = init_magics_rooks();
    pub static ref MAGICS_BISHOPS: Vec<Magic> = init_magics_bishops();
}

pub struct Magic {
    pub occupancy_mask: u64,
    pub shift: usize,
    pub magic_num: u64,
    pub lookup_table: Vec<u64>,
}

impl Magic {
    pub fn get_attacks(&self, all_pieces_board: u64) -> u64 {
        self.lookup_table[(((all_pieces_board & self.occupancy_mask).wrapping_mul(self.magic_num))
            >> (64 - self.shift)) as usize]
    }
}

pub fn init_magics() {
    MAGICS_ROOKS.len();
    MAGICS_BISHOPS.len();
}

//Rook-specific magic
pub fn init_magics_rooks() -> Vec<Magic> {
    let mut res: Vec<Magic> = Vec::with_capacity(64);
    for square in 0..64 {
        let shift = ROOK_BITS[square];
        let occupancy_mask = occupancy_mask_rooks(square);
        if occupancy_mask.count_ones() as usize != shift {
            panic!("Not good!");
        }
        let mut blockers_by_index: Vec<u64> = Vec::with_capacity(1 << shift);
        let mut attack_table: Vec<u64> = Vec::with_capacity(1 << shift);
        //Initialize lookup table
        for i in 0..(1 << shift) {
            //i is index of lookup table
            blockers_by_index.push(blockers_to_bitboard(i, shift, occupancy_mask));
            attack_table.push(rook_attacks_slow(square, blockers_by_index[i]));
        }
        let magic_num = MAGIC_NUMS_ROOKS[square];
        let mut lookup_table = Vec::with_capacity(1 << shift);
        for _i in 0..(1 << shift) {
            lookup_table.push(0u64);
        }
        //Calculate lookup table
        for i in 0..(1 << shift) {
            let j = transform(blockers_by_index[i], magic_num, shift);
            if lookup_table[j] == 0u64 {
                lookup_table[j] = attack_table[i];
            } else {
                panic!("Isn't valid num {:x}!", magic_num)
            }
        }
        res.push(Magic {
            occupancy_mask,
            shift,
            magic_num,
            lookup_table,
        })
    }
    log("Finished Initializing Rook Attacks!");
    res
}

pub fn occupancy_mask_rooks(square: usize) -> u64 {
    ((RANKS[square / 8] & !(FILES[0] | FILES[7])) | (FILES[square % 8] & !(RANKS[0] | RANKS[7])))
        & NOT_SQUARES[square]
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

//Bishop-specific magic
pub fn init_magics_bishops() -> Vec<Magic> {
    let mut res: Vec<Magic> = Vec::with_capacity(64);
    for square in 0..64 {
        let shift = BISHOP_BITS[square];

        let occupancy_mask = occupancy_mask_bishops(square);
        if occupancy_mask.count_ones() as usize != shift {
            panic!("Not good!");
        }

        let mut blockers_by_index: Vec<u64> = Vec::with_capacity(1 << shift);
        let mut attack_table: Vec<u64> = Vec::with_capacity(1 << shift);
        //Initialize lookup table
        for i in 0..(1 << shift) {
            //i is index of lookup table
            blockers_by_index.push(blockers_to_bitboard(i, shift, occupancy_mask));
            attack_table.push(bishop_attacks_slow(square, blockers_by_index[i]));
        }

        let magic_num = MAGIC_NUMS_BISHOPS[square];
        let mut lookup_table = Vec::with_capacity(1 << shift);
        for _i in 0..(1 << shift) {
            lookup_table.push(0u64);
        }
        //Calculate lookup table
        for i in 0..(1 << shift) {
            let j = transform(blockers_by_index[i], magic_num, shift);
            if lookup_table[j] == 0u64 {
                lookup_table[j] = attack_table[i];
            } else {
                panic!("Isn't valid num!")
            }
        }
        res.push(Magic {
            occupancy_mask,
            shift,
            magic_num,
            lookup_table,
        })
    }
    log("Finished Initializing Bishop Attacks!");
    res
}

pub fn occupancy_mask_bishops(square: usize) -> u64 {
    let mut res = 0u64;
    let rk = (square / 8) as isize;
    let fl = (square % 8) as isize;
    let dirs: [(isize, isize); 4] = [(1, 1), (-1, -1), (1, -1), (-1, 1)];
    for dir in dirs.iter() {
        let (file_i, rank_i) = dir;
        let mut rn = rk + rank_i;
        let mut fnn = fl + file_i;
        while rn >= 1 && rn <= 6 && fnn >= 1 && fnn <= 6 {
            res |= 1u64 << (rn * 8 + fnn);
            rn += rank_i;
            fnn += file_i;
        }
    }
    res
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

//General magic stuff
pub fn transform(blockers: u64, magic: u64, n_bits: usize) -> usize {
    ((blockers.wrapping_mul(magic)) >> (64 - n_bits)) as usize
}

pub fn generate_magic(
    blockers_by_index: &[u64],
    attack_table: &[u64],
    n_bits: usize,
    occ_mask: u64,
) -> u64 {
    for _iterations in 0..100_000_000 {
        let random_magic = random_u64_fewbits();
        if ((occ_mask.wrapping_mul(random_magic)) & 0xFF00_0000_0000_0000u64) < 6 {
            continue;
        }
        if is_valid_magic(random_magic, n_bits, blockers_by_index, attack_table) {
            return random_magic;
        }
    }
    panic!("No Magic found!");
}

pub fn is_valid_magic(
    magic: u64,
    n_bits: usize,
    blockers_by_index: &[u64],
    attack_table: &[u64],
) -> bool {
    let mut used = Vec::with_capacity(1 << n_bits);
    for _i in 0..(1 << n_bits) {
        used.push(0u64);
    }
    for i in 0..(1 << n_bits) {
        let j = transform(blockers_by_index[i], magic, n_bits);
        if used[j] == 0u64 {
            used[j] = attack_table[i];
        } else if used[j] != attack_table[i] {
            return false;
        }
    }
    true
}

pub fn random_u64() -> u64 {
    rand::thread_rng().gen::<u64>()
}

pub fn random_u64_fewbits() -> u64 {
    random_u64() & random_u64() & random_u64()
}

pub fn blockers_to_bitboard(block_index: usize, n_bits: usize, mut mask: u64) -> u64 {
    let mut res = 0u64;
    for i in 0..n_bits {
        let j = mask.trailing_zeros();
        mask &= !(1 << j);
        if (block_index & (1 << i)) != 0 {
            res |= 1u64 << j;
        }
    }
    res
}

#[allow(dead_code)]
pub fn is_valid_magic_square_rook(magic: u64, square: usize) -> bool {
    let shift = ROOK_BITS[square];
    let occupancy_mask = occupancy_mask_rooks(square);
    let mut blockers_by_index: Vec<u64> = Vec::with_capacity(1 << shift);
    let mut attack_table: Vec<u64> = Vec::with_capacity(1 << shift);
    //Initialize Attack table
    for i in 0..(1 << shift) {
        //i is index of Attack table
        blockers_by_index.push(blockers_to_bitboard(i, shift, occupancy_mask));
        attack_table.push(rook_attacks_slow(square, blockers_by_index[i]));
    }
    is_valid_magic(magic, shift, &blockers_by_index, &attack_table)
}

#[allow(dead_code)]
pub fn generate_all_magic_nums_rook() {
    for (square, shift) in ROOK_BITS.iter().enumerate() {
        let occupancy_mask = occupancy_mask_rooks(square);
        if occupancy_mask.count_ones() as usize != *shift {
            panic!("Not good!");
        }
        let mut blockers_by_index: Vec<u64> = Vec::with_capacity(1 << *shift);
        let mut attack_table: Vec<u64> = Vec::with_capacity(1 << *shift);
        //Initialize lookup table
        for i in 0..(1 << *shift) {
            //i is index of lookup table
            blockers_by_index.push(blockers_to_bitboard(i, *shift, occupancy_mask));
            attack_table.push(rook_attacks_slow(square, blockers_by_index[i]));
        }
        let magic_num = generate_magic(&blockers_by_index, &attack_table, *shift, occupancy_mask);
        print!("0x{:x}u64,", magic_num);
    }
}

#[allow(dead_code)]
pub fn is_valid_magic_square_bishop(magic: u64, square: usize) -> bool {
    let shift = BISHOP_BITS[square];

    let occupancy_mask = occupancy_mask_bishops(square);
    let mut blockers_by_index: Vec<u64> = Vec::with_capacity(1 << shift);
    let mut attack_table: Vec<u64> = Vec::with_capacity(1 << shift);
    //Initialize Attack table
    for i in 0..(1 << shift) {
        //i is index of Attack table
        blockers_by_index.push(blockers_to_bitboard(i, shift, occupancy_mask));
        attack_table.push(bishop_attacks_slow(square, blockers_by_index[i]));
    }
    is_valid_magic(magic, shift, &blockers_by_index, &attack_table)
}

#[allow(dead_code)]
pub fn generate_all_magic_nums_bishop() {
    for (square, shift) in BISHOP_BITS.iter().enumerate() {
        let occupancy_mask = occupancy_mask_bishops(square);
        if occupancy_mask.count_ones() as usize != *shift {
            panic!("Not good!");
        }
        let mut blockers_by_index: Vec<u64> = Vec::with_capacity(1 << *shift);
        let mut attack_table: Vec<u64> = Vec::with_capacity(1 << *shift);
        //Initialize lookup table
        for i in 0..(1 << *shift) {
            //i is index of lookup table
            blockers_by_index.push(blockers_to_bitboard(i, *shift, occupancy_mask));
            attack_table.push(bishop_attacks_slow(square, blockers_by_index[i]));
        }
        let magic_num = generate_magic(&blockers_by_index, &attack_table, *shift, occupancy_mask);
        print!("0x{:x}u64,", magic_num);
    }
}
