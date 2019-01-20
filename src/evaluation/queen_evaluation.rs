use super::VERBOSE;
pub const QUEEN_PIECE_VALUE_MG: f64= 1500.0;
pub const QUEEN_PIECE_VALUE_EG: f64= 1600.0;

pub fn queen_eval(queen:u64)->(f64,f64){
    let queens= queen.count_ones();
    let mg= queens as f64*QUEEN_PIECE_VALUE_MG;
    let eg= queens as f64*QUEEN_PIECE_VALUE_EG;
    if VERBOSE{
        println!("------------------------------------------------");
        println!("\tQueens-MidGame");
        println!("\t\tAmount of Queens:     \t{} -> {}",queens,queens as f64 * QUEEN_PIECE_VALUE_MG);
        println!("\tSum: {}",mg);
        println!("------------------------------------------------");
        println!("------------------------------------------------");
        println!("\tQueens-EndGame");
        println!("\t\tAmount of Queens:     \t{} -> {}",queens,queens as f64 * QUEEN_PIECE_VALUE_EG);
        println!("\tSum: {}",eg);
        println!("------------------------------------------------");
    }
    (mg,eg)
}