use super::{bitboards,VERBOSE};
pub const ROOK_PIECE_VALUE_MG: f64= 710.0;
pub const ROOK_PIECE_VALUE_EG: f64= 920.0;
pub const ROOK_ON_OPEN_FILE_BONUS:f64=20.0;
pub const ROOK_ON_SEVENTH:f64=10.0;

pub fn rook_eval(mut rook:u64,all_pawns:u64,is_white:bool)->(f64,f64){
    let mut rook_count=0;
    let mut  seventh_bonus_cnt=0;
    let mut openfile_bonus_cnt=0;
    while rook!=0u64{
        let idx = rook.trailing_zeros() as usize;
        rook_count+=1;
        if is_white{
            if idx/8==6{
                seventh_bonus_cnt+=1;
            }
        }else{
            if idx/8==1{
                seventh_bonus_cnt+=1;
            }
        }
        if bitboards::FILES[idx%8]&all_pawns==0u64{
            openfile_bonus_cnt+=1;
        }
        rook ^= 1u64<<idx;
    }
    let mg= rook_count as f64*ROOK_PIECE_VALUE_MG+ seventh_bonus_cnt as f64*ROOK_ON_SEVENTH+openfile_bonus_cnt as f64*ROOK_ON_OPEN_FILE_BONUS;
    let eg= rook_count as f64*ROOK_PIECE_VALUE_EG+ seventh_bonus_cnt as f64*ROOK_ON_SEVENTH+openfile_bonus_cnt as f64*ROOK_ON_OPEN_FILE_BONUS;
    if VERBOSE{
        println!("------------------------------------------------");
        println!("\tRooks-MidGame");
        println!("\t\tAmount of Rooks:     \t{} -> {}",rook_count,rook_count as f64 * ROOK_PIECE_VALUE_MG);
        println!("\t\tRooks on Seventh:    \t{} -> {}",seventh_bonus_cnt,seventh_bonus_cnt as f64*ROOK_ON_SEVENTH);
        println!("\t\tRooks on Open File:  \t{} -> {}",openfile_bonus_cnt,openfile_bonus_cnt as f64*ROOK_ON_OPEN_FILE_BONUS);
        println!("\tSum: {}",mg);
        println!("------------------------------------------------");
        println!("------------------------------------------------");
        println!("\tRooks-EndGame");
        println!("\t\tAmount of Rooks:     \t{} -> {}",rook_count,rook_count as f64 * ROOK_PIECE_VALUE_EG);
        println!("\t\tRooks on Seventh:    \t{} -> {}",seventh_bonus_cnt,seventh_bonus_cnt as f64*ROOK_ON_SEVENTH);
        println!("\t\tRooks on Open File:  \t{} -> {}",openfile_bonus_cnt,openfile_bonus_cnt as f64*ROOK_ON_OPEN_FILE_BONUS);
        println!("\tSum: {}",eg);
        println!("------------------------------------------------");
    }
    (mg,eg)
}