use super::VERBOSE;
pub const KNIGHT_PIECE_VALUE_MG: f64 = 500.0;
pub const KNIGHT_PIECE_VALUE_EG: f64 = 500.0;
pub const KNIGHT_VALUE_WITH_PAWNS: [f64; 17] = [-30.0, -27.5, -25.0, -22.5, -20.0, -17.5, -15.0, -12.5, -10.0, -7.5, -5.0, -2.5, 0.0, 2.5, 5.0, 7.5, 10.0];
pub const KNIGHT_SUPPORTED_BY_PAWN: f64 = 30.0;
pub const PSQT_KNIGHT_MG: [[f64; 8]; 8] = [
    [ -50.0,-40.0,-30.0,-30.0,-30.0,-30.0,-40.0, -50.0],
    [ -40.0,-20.0,  0.0,  5.0,  5.0,  0.0,-20.0, -40.0],
    [ -30.0,  0.0, 10.0, 20.0, 20.0, 10.0,  0.0, -30.0],
    [ -30.0,  5.0, 20.0, 40.0, 40.0, 20.0,  5.0, -30.0],
    [ -30.0,  5.0, 20.0, 40.0, 40.0, 20.0,  5.0, -30.0],
    [ -30.0,  0.0, 10.0, 20.0, 20.0, 10.0,  0.0, -30.0],
    [ -40.0,-20.0, 0.0,  5.0,  5.0,   0.0,-20.0, -40.0],
    [ -50.0,-40.0,-30.0,-30.0,-30.0,-30.0,-40.0, -50.0],
];
pub const PSQT_KNIGHT_EG: [[f64;8];8]=[
    [ -50.0,-40.0,-30.0,-30.0,-30.0,-30.0,-40.0, -50.0],
    [ -40.0,-25.0,-10.0,  0.0,  0.0,-10.0,-25.0, -40.0],
    [ -30.0,-10.0,  5.0, 10.0, 10.0,  5.0,-10.0, -30.0],
    [ -30.0,  0.0, 10.0, 20.0, 20.0, 10.0,  0.0, -30.0],
    [ -30.0,  0.0, 10.0, 20.0, 20.0, 10.0,  0.0, -30.0],
    [ -30.0,-10.0,  5.0, 10.0, 10.0,  5.0,-10.0, -30.0],
    [ -40.0,-25.0,-10.0,  0.0,  0.0,-10.0,-25.0, -40.0],
    [ -50.0,-40.0,-30.0,-30.0,-30.0,-30.0,-40.0, -50.0],
];

pub fn knight_eval(mut knight:u64,my_pawns_attacks:u64,my_pawns:u64,enemy_pawns:u64)->(f64,f64){
    let pawns_insg= (my_pawns.count_ones()+enemy_pawns.count_ones()) as usize;
    let supported_knights= (my_pawns_attacks&knight).count_ones();
    //PSQT Values
    let mut amount_of_knights=0;
    let mut mg_psqt=0.0;
    let mut eg_psqt=0.0;
    while knight!=0u64{
        let idx= knight.trailing_zeros() as usize;
        mg_psqt+=PSQT_KNIGHT_MG[idx/8usize][idx%8usize];
        eg_psqt+=PSQT_KNIGHT_EG[idx/8usize][idx%8usize];
        amount_of_knights+=1;
        knight ^=1u64<<idx;
    }
    let mg= mg_psqt+amount_of_knights as f64 * (KNIGHT_PIECE_VALUE_MG+KNIGHT_VALUE_WITH_PAWNS[pawns_insg])+supported_knights as f64*KNIGHT_SUPPORTED_BY_PAWN;
    let eg= eg_psqt+amount_of_knights as f64 *(KNIGHT_PIECE_VALUE_EG+KNIGHT_VALUE_WITH_PAWNS[pawns_insg])+supported_knights as f64*KNIGHT_SUPPORTED_BY_PAWN;
    if VERBOSE{
        println!("------------------------------------------------");
        println!("\tKnights-MidGame");
        println!("\t\tAmount of Knights:     \t{} -> {}",amount_of_knights,amount_of_knights as f64 * KNIGHT_PIECE_VALUE_MG);
        println!("\t\tSupported Knights:     \t{} -> {}",supported_knights,supported_knights as f64*KNIGHT_SUPPORTED_BY_PAWN);
        println!("\t\tKnight decreased Value:\t{} -> {}",KNIGHT_VALUE_WITH_PAWNS[pawns_insg],amount_of_knights as f64* KNIGHT_VALUE_WITH_PAWNS[pawns_insg]);
        println!("\t\tPSQT-Value:            \t{}",mg_psqt);
        println!("\tSum: {}",mg);
        println!("------------------------------------------------");
        println!("------------------------------------------------");
        println!("\tKnights-EndGame");
        println!("\t\tAmount of Knights:     \t{} -> {}",amount_of_knights,amount_of_knights as f64 * KNIGHT_PIECE_VALUE_EG);
        println!("\t\tSupported Knights:     \t{} -> {}",supported_knights,supported_knights as f64*KNIGHT_SUPPORTED_BY_PAWN);
        println!("\t\tKnight decreased Value:\t{}",KNIGHT_VALUE_WITH_PAWNS[pawns_insg]);
        println!("\t\tPSQT-Value:            \t{}",eg_psqt);
        println!("\tSum: {}",eg);
        println!("------------------------------------------------");
    }
    (mg,eg)
}