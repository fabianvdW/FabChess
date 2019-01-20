use super::{bitboards,VERBOSE};
pub const BISHOP_PIECE_VALUE_EG: f64 = 510.0;
pub const BISHOP_PIECE_VALUE_MG: f64 = 510.0;
pub const BISHOP_PAIR_BONUS:f64 =50.0;
pub const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS:[f64;5]= [30.0,15.0,0.0,-15.0,-30.0];

pub const PSQT_BISHOP_MG: [[f64;8];8]=[
    [ -50.0,-10.0,-10.0,-30.0,-30.0,-10.0,-10.0, -50.0],
    [ -30.0, 10.0, 15.0,  0.0,  0.0, 15.0, 10.0, -30.0],
    [ -10.0, 10.0, 15.0, 10.0, 10.0, 15.0, 10.0, -10.0],
    [ -10.0, 15.0, 20.0, 25.0, 25.0, 20.0,  0.0, -10.0],
    [ -10.0, 10.0, 20.0, 25.0, 25.0, 20.0,  0.0, -10.0],
    [ -10.0, 10.0, 15.0, 10.0, 10.0, 15.0, 10.0, -10.0],
    [ -30.0, 10.0, 15.0,  0.0,  0.0, 15.0, 10.0, -30.0],
    [ -50.0,-10.0,-10.0,-30.0,-30.0,-10.0,-10.0, -50.0],
];

pub const PSQT_BISHOP_EG: [[f64;8];8]=[
    [ -50.0, -30.0, -30.0, -20.0, -20.0, -30.0, -30.0, -50.0],
    [ -30.0, -10.0, -10.0,   5.0,   5.0, -10.0, -10.0, -30.0],
    [ -20.0,   0.0,   0.0,  12.0,  12.0,   0.0,  0.0,  -20.0],
    [ -20.0,   0.0,   0.0,  12.0,  12.0,   0.0,  0.0,  -20.0],
    [ -20.0,   0.0,   0.0,  12.0,  12.0,   0.0,  0.0,  -20.0],
    [ -20.0,   0.0,   0.0,  12.0,  12.0,   0.0,  0.0,  -20.0],
    [ -30.0, -10.0, -10.0,   5.0,   5.0, -10.0, -10.0, -30.0],
    [ -50.0, -30.0, -30.0, -20.0, -20.0, -30.0, -30.0, -50.0],
];

pub fn bishop_eval(mut bishop:u64,my_pawns:u64)->(f64,f64){
    let mut mg_psqt=0.0;
    let mut eg_psqt=0.0;
    let mut diagonally_adj_bonus=0.0;
    let mut bishop_count=0;
    while bishop!=0u64{
        let idx= bishop.trailing_zeros() as usize;
        bishop^= 1u64<<idx;
        mg_psqt+=PSQT_BISHOP_MG[idx/8usize][idx%8usize];
        eg_psqt+=PSQT_BISHOP_EG[idx/8usize][idx%8usize];
        diagonally_adj_bonus+=DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS[(bitboards::DIAGONALLY_ADJACENT[idx]&my_pawns).count_ones() as usize];
        bishop_count+=1;
    }
    let mg= mg_psqt+bishop_count as f64*BISHOP_PIECE_VALUE_MG+diagonally_adj_bonus+if bishop_count>1{BISHOP_PAIR_BONUS}else { 0.0 };
    let eg= eg_psqt+bishop_count as f64*BISHOP_PIECE_VALUE_EG+diagonally_adj_bonus+if bishop_count>1{BISHOP_PAIR_BONUS}else { 0.0 };
    if VERBOSE{
        println!("------------------------------------------------");
        println!("\tBishops-MidGame");
        println!("\t\tAmount of Bishops:     \t{} -> {}",bishop_count,bishop_count as f64 * BISHOP_PIECE_VALUE_MG);
        println!("\t\tDiagonal Blocks:       \t{}",diagonally_adj_bonus);
        println!("\t\tBishop Pair:           \t{}",if bishop_count>1{BISHOP_PAIR_BONUS}else{0.0});
        println!("\t\tPSQT-Value:            \t{}",mg_psqt);
        println!("\tSum: {}",mg);
        println!("------------------------------------------------");
        println!("------------------------------------------------");
        println!("\tBishops-EndGame");
        println!("\t\tAmount of Bishops:     \t{} -> {}",bishop_count,bishop_count as f64 * BISHOP_PIECE_VALUE_EG);
        println!("\t\tDiagonal Blocks:       \t{}",diagonally_adj_bonus);
        println!("\t\tBishop Pair:           \t{}",if bishop_count>1{BISHOP_PAIR_BONUS}else{0.0});
        println!("\t\tPSQT-Value:            \t{}",eg_psqt);
        println!("\tSum: {}",eg);
        println!("------------------------------------------------");
    }
    (mg,eg)
}