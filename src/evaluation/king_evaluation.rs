use super::{bitboards,VERBOSE};
const PSQT_KING_MG: [[f64;8];8]=[
    [40.0,60.0,20.0,0.0,0.0,20.0,60.0,40.0],
    [40.0,40.0,0.0,0.0,0.0,0.0,20.0,20.0],
    [-20.0,-40.0,-40.0,-40.0,-40.0,-40.0,-40.0,-20.0],
    [-40.0,-60.0,-60.0,-80.0,-80.0,-60.0,-60.0,-40.0],
    [-60.0,-80.0,-80.0,-100.0,-100.0,-80.0,-80.0,-60.0],
    [-60.0,-80.0,-80.0,-100.0,-100.0,-80.0,-80.0,-60.0],
    [-60.0,-80.0,-80.0,-100.0,-100.0,-80.0,-80.0,-60.0],
    [-60.0,-80.0,-80.0,-100.0,-100.0,-80.0,-80.0,-60.0],
];
const PSQT_KING_EG: [[f64;8];8]=[
    [-100.0,-60.0,-60.0,-60.0,-60.0,-60.0,-60.0,-100.0],
    [-60.0,-60.0,0.0,0.0,0.0,0.0,-60.0,-60.0],
    [-60.0,-20.0,40.0,60.0,60.0,40.0,-20.0,-60.0],
    [-60.0,-20.0,60.0,80.0,80.0,60.0,-20.0,-60.0],
    [-60.0,-20.0,60.0,80.0,80.0,60.0,-20.0,-60.0],
    [-60.0,-20.0,40.0,60.0,60.0,40.0,-20.0,-60.0],
    [-60.0,-40.0,-20.0,0.0,0.0,-20.0,-40.0,-60.0],
    [-100.0,-80.0,-60.0,-40.0,-40.0,-60.0,-80.0,-100.0]
];
const SHIELDING_PAWN_MISSING_MG:f64= -20.0;
const SHIELDING_PAWN_MISSING_ON_OPEN_FILE:f64= -40.0;

pub fn king_eval(king:u64,is_white:bool,my_pawns:u64,enemy_pawns:u64)->(f64,f64){
    let mut king_index=king.trailing_zeros() as usize;
    if !is_white{
        king_index=63-king_index;
    }
    let king_rank=king_index/8usize;
    let king_file= king_index%8usize;
    let psqt_mg=PSQT_KING_MG[king_rank][king_file];
    let psqt_eg=PSQT_KING_EG[king_rank][king_file];
    let mut shield= if is_white{bitboards::SHIELDING_PAWNS_WHITE[king_index]}else{bitboards::SHIELDING_PAWNS_BLACK[63-king_index]};
    let mut shields_missing=0;
    let mut shields_on_open_missing=0;
    while shield!=0u64{
        let idx = shield.trailing_zeros() as usize;
        //Block out whole file
        let file= bitboards::FILES[idx%8];
        if my_pawns&shield&file==0u64{
            shields_missing+=1;
            if enemy_pawns&file==0u64{
                shields_on_open_missing+=1;
            }
        }
        shield&=!file;
    }
    let mg= psqt_mg+shields_missing as f64*SHIELDING_PAWN_MISSING_MG+shields_on_open_missing as f64*SHIELDING_PAWN_MISSING_ON_OPEN_FILE;
    let eg=psqt_eg;
    if VERBOSE{
        println!("------------------------------------------------");
        println!("\tKing-MidGame");
        println!("\t\tShielding Pawns missing:             \t{} -> {}",shields_missing,shields_missing as f64* SHIELDING_PAWN_MISSING_MG);
        println!("\t\tShielding Pawns on open file missing:\t{} -> {}",shields_on_open_missing, shields_on_open_missing as f64 *SHIELDING_PAWN_MISSING_ON_OPEN_FILE);
        println!("\t\tPSQT:                                \t{}",psqt_mg);
        println!("\tSum: {}",mg);
        println!("------------------------------------------------");
        println!("------------------------------------------------");
        println!("\tKing-EndGame");
        println!("\t\tPSQT:     \t{}",psqt_eg);
        println!("\tSum: {}",eg);
        println!("------------------------------------------------");
    }
    (mg,eg)
}