use rand::Rng;
static mut ROOK_BITS: [usize; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12
];
static mut BISHOP_BITS: [usize; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6
];
const MAGIC_NUMS_ROOKS: [u64; 64] = [
    0x2180028011204008u64, 0x21400190002000c0u64, 0x8480100020000b80u64, 0x80100080061800u64, 0xc100080004030010u64, 0x200084430120001u64, 0x1880110006800200u64, 0x100006200804100u64, 0x4084800280400020u64, 0x8400400050002000u64, 0xd001020010342u64, 0x20040102008040u64, 0x8000802400810800u64, 0x2922002804100a01u64, 0x4109002a0099000cu64, 0x40020000840e0841u64, 0x800848000204011u64, 0x900444000201008u64, 0x5201010020049142u64, 0x80620009c20030u64, 0x45310004080100u64, 0x808002001400u64, 0x1090840002100328u64, 0x8022000184085bu64, 0x401080028020u64, 0x4910400080802000u64, 0x1400403100200100u64, 0x910008080180450u64, 0x200040080800800u64, 0x1052000200100429u64, 0x2004e80c000a1110u64, 0x1423000100005282u64, 0x8088834000800aa0u64, 0x10080c000806000u64, 0x8010200082805002u64, 0x82101003000u64, 0x7040080280800400u64, 0x610c004100401200u64, 0x5700010804001002u64, 0x1000008042000904u64, 0x21049460c0008000u64, 0x410014020004002u64, 0x20100a001410013u64, 0x401000810010020u64, 0x4008012400808008u64, 0x808200440480110u64, 0x1b81040200010100u64, 0x8020040040820003u64, 0x80002108508100u64, 0x10242082b008200u64, 0x801004090200100u64, 0x4003002008100100u64, 0x8180080440080u64, 0x890c0004800a0080u64, 0x104020108102400u64, 0x41111040840200u64, 0x1010201601008042u64, 0x127002440013181u64, 0x102104209220082u64, 0xa011009001242009u64, 0x81000228001085u64, 0x5022001008011c02u64, 0x100821000810804u64, 0x100002280d601u64, ];
const MAGIC_NUMS_BISHOPS: [u64; 64] = [
    0x6040422a14086080u64, 0x4084800408020u64, 0x400c080210440010u64, 0x44042080000000u64, 0x2610882002100a00u64, 0x9030080c0080u64, 0xc140841048148004u64, 0x200844c100442u64, 0x40042004042682u64, 0x4200100608410a28u64, 0x802010052a202108u64, 0x800c080601410802u64, 0xc001011040020004u64, 0x488811002902400u64, 0xa008024210106808u64, 0x1004a100e9042004u64, 0x2030002102e20800u64, 0x510a5810210042u64, 0x68010408089830u64, 0x41c800802044200u64, 0x4004080a04000u64, 0x4004080203040a00u64, 0x830642382040u64, 0x8200401101c23040u64, 0x8400248108110u64, 0x4440020289080u64, 0x3008040408004050u64, 0x8004004004010002u64, 0x921004024044016u64, 0x201840802020a00u64, 0x20222c0000414811u64, 0x5802068602404810u64, 0x2791200810a29848u64, 0x2650821000081000u64, 0x415000480322u64, 0x2041401820060200u64, 0x1020200240104u64, 0x10802000c2212u64, 0xe002880a00005200u64, 0x2001062200402100u64, 0x52105044000850u64, 0x204590820800818u64, 0x1201410082a00u64, 0x440004200810800u64, 0x20080100442401u64, 0x200b200214100880u64, 0x2810108100400100u64, 0x8824640052011048u64, 0x4281040114400060u64, 0x988824040208001du64, 0x806081c40c040909u64, 0x20090084110014u64, 0x304006022440118u64, 0x1011400304010004u64, 0xa1020010410a048u64, 0x8110020809002809u64, 0x40010022100c0413u64, 0x2800020125011014u64, 0x46082400u64, 0x60408400840401u64, 0x8001020010021204u64, 0x4c00100812084200u64, 0x20000420440c1098u64, 0x802200c01120060u64, ];
lazy_static! {
    pub static ref FILES: [u64;8] = initialize_files();
    pub static ref NOT_FILES: [u64;8] = initialize_not_files();
    pub static ref RANKS: [u64;8] = initialize_ranks();
    pub static ref SQUARES: [u64;64]= initialize_squares();
    pub static ref NOT_SQUARES: [u64;64]= initialize_not_squares();
    pub static ref MAGICS_ROOKS: Vec<Magic>= init_magics_rooks();
    pub static ref MAGICS_BISHOPS:Vec<Magic>= init_magics_bishops();
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
pub struct Magic {
    pub occupancy_mask: u64,
    pub shift: usize,
    pub magic_num: u64,
    pub lookup_table: Vec<u64>,
}

impl Magic {
    pub fn get_attacks(&self, all_pieces_board: u64) -> u64 {
        self.lookup_table[(((all_pieces_board & self.occupancy_mask).wrapping_mul(self.magic_num)) >> (64 - self.shift)) as usize]
    }
}

pub fn init_all() {
    FILES.len();
    NOT_FILES.len();
    RANKS.len();
    SQUARES.len();
    NOT_SQUARES.len();
    //Override Magics with better magic nums
    MAGICS_ROOKS.len();
    MAGICS_BISHOPS.len();
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

//Rook-specific magic
pub fn init_magics_rooks() -> Vec<Magic> {
    let mut res: Vec<Magic> = Vec::with_capacity(64);
    for square in 0..64 {
        let shift;
        unsafe {
            shift = ROOK_BITS[square];
        }
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
        //Calculate look-up table
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
    println!("Finished Initializing Rook Attacks!");
    res
}

pub fn occupancy_mask_rooks(square: usize) -> u64 {
    (((RANKS[square / 8] & !(FILES[0] | FILES[7])) | (FILES[square % 8] & !(RANKS[0] | RANKS[7]))) & NOT_SQUARES[square])
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
/*
pub fn is_valid_magic_square_rook(magic: u64, square: usize) -> bool {
    let shift;
    unsafe {
        shift = ROOK_BITS[square];
    }
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
*/
/*
pub fn generate_all_magic_nums_rook() {
    for square in 0..64 {
        let shift;
        unsafe {
            shift = ROOK_BITS[square];
        }
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
        let magic_num = generate_magic(&blockers_by_index, &attack_table, shift, occupancy_mask);
        print!("0x{:x}u64,", magic_num);
    }
}
*/
//Bishop-specific magic
pub fn init_magics_bishops() -> Vec<Magic> {
    let mut res: Vec<Magic> = Vec::with_capacity(64);
    for square in 0..64 {
        let shift;
        unsafe {
            shift = BISHOP_BITS[square];
        }

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
        //Calculate look-up table
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
    println!("Finished Initializing Bishop Attacks!");
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

/*
pub fn is_valid_magic_square_bishop(magic: u64, square: usize) -> bool {
    let shift;
    unsafe {
        shift = BISHOP_BITS[square];
    }
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
*/
/*
pub fn generate_all_magic_nums_bishop() {
    for square in 0..64 {
        let shift;
        unsafe {
            shift = BISHOP_BITS[square];
        }
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
        let magic_num = generate_magic(&blockers_by_index, &attack_table, shift, occupancy_mask);
        print!("0x{:x}u64,", magic_num);
    }
}
*/
//General Magic stuff
pub fn transform(blockers: u64, magic: u64, n_bits: usize) -> usize {
    ((blockers.wrapping_mul(magic)) >> (64 - n_bits)) as usize
}

pub fn generate_magic(blockers_by_index: &Vec<u64>, attack_table: &Vec<u64>, n_bits: usize, occ_mask: u64) -> u64 {
    for _iterations in 0..100000000 {
        let random_magic = random_u64_fewbits();
        if ((occ_mask.wrapping_mul(random_magic)) & 0xFF00000000000000u64) < 6 {
            continue;
        }
        if is_valid_magic(random_magic, n_bits, blockers_by_index, attack_table) {
            return random_magic;
        }
    }
    panic!("No Magic found!");
}

pub fn is_valid_magic(magic: u64, n_bits: usize, blockers_by_index: &Vec<u64>, attack_table: &Vec<u64>) -> bool {
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
    return true;
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
//Misc
pub fn to_string_board(board: u64) -> String {
    let mut res_str: String = String::new();
    res_str.push_str("+---+---+---+---+---+---+---+---+\n");
    for rank in 0..8 {
        res_str.push_str("| ");
        for file in 0..8 {
            let idx = 8 * (7 - rank) + file;
            if ((board >> idx) & 1) != 0 {
                res_str.push_str("X");
            } else {
                res_str.push_str(" ");
            }
            res_str.push_str(" | ");
        }
        res_str.push_str("\n+---+---+---+---+---+---+---+---+\n");
    }
    res_str
}
