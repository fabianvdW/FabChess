use super::game_state::GameState;
use super::bitboards;
use super::movegen;
use std::cmp;

const VERBOSE: bool = true;
//Pawn constants
const PAWN_PIECE_VALUE_MG: f64 = 100.0;
const PAWN_PIECE_VALUE_EG: f64 = 150.0;
const PAWN_DOUBLED_VALUE_MG: f64 = -8.0;
const PAWN_DOUBLED_VALUE_EG: f64 = -37.5;
const PAWN_ISOLATED_VALUE_MG: f64 = -5.0;
const PAWN_ISOLATED_VALUE_EG: f64 = -15.0;
const PAWN_BACKWARD_VALUE_MG: f64 = -10.0;
const PAWN_BACKWARD_VALUE_EG: f64 = -25.0;
const PAWN_PASSED_VALUES_MG: [f64; 7] = [0.0, -20.0, -10.0, 10.0, 70.0, 120.0, 200.0];
const PAWN_PASSED_NOT_BLOCKED_VALUES_MG: [f64; 7] = [0.0, 0.0, 0.0, 25.0, 40.0, 130.0, 210.0];
const PAWN_PASSED_VALUES_EG: [f64; 7] = [0.0, -40.0, -20.0, 20.0, 140.0, 240.0, 400.0];
const PAWN_PASSED_NOT_BLOCKED_VALUES_EG: [f64; 7] = [0.0, 0.0, 0.0, 50.0, 80.0, 260.0, 420.0];

const PSQT_PAWN_MG: [[f64; 8]; 8] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [-5.0, -5.0, -5.0, -5.0, -5.0, -5.0, -5.0, -5.0],
    [-7.0, 3.0, 6.0, 10.0, 10.0, 6.0, 3.0, -7.0],
    [-14.0, -7.0, 15.0, 20.0, 20.0, 15.0, -7.0, -14.0],
    [-10.0, -2.0, 1.0, 12.0, 12.0, 1.0, -2.0, -10.0],
    [-7.0, -1.0, 0.0, 5.0, 5.0, 0.0, -1.0, -7.0],
    [-3.0, 10.0, 5.0, 5.0, 5.0, 5.0, 10.0, -3.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
];
const PSQT_PAWN_EG: [[f64; 8]; 8] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [-20.0, -20.0, -20.0, -20.0, -20.0, -20.0, -20.0, -20.0],
    [-10.0, -10.0, -10.0, -10.0, -10.0, -10.0, -10.0, -10.0],
    [-5.0, -5.0, -5.0, -5.0, -5.0, -5.0, -5.0, -5.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
    [10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
];
//Knight constants
const KNIGHT_PIECE_VALUE_MG: f64 = 500.0;
const KNIGHT_PIECE_VALUE_EG: f64 = 500.0;
const KNIGHT_VALUE_WITH_PAWNS: [f64; 17] = [-30.0, -27.5, -25.0, -22.5, -20.0, -17.5, -15.0, -12.5, -10.0, -7.5, -5.0, -2.5, 0.0, 2.5, 5.0, 7.5, 10.0];
const KNIGHT_SUPPORTED_BY_PAWN: f64 = 30.0;
const PSQT_KNIGHT_MG: [[f64; 8]; 8] = [
    [-150.0,-95.0,-75.0,-75.0,-75.0,-75.0,-95.0,-150.0],
    [ -95.0,-50.0,-20.0,  0.0,  0.0,-20.0,-50.0, -95.0],
    [ -75.0,-20.0, 10.0, 20.0, 20.0, 10.0,-20.0, -75.0],
    [ -75.0,  0.0, 20.0, 40.0, 40.0, 20.0,  0.0, -75.0],
    [ -75.0,  0.0, 20.0, 40.0, 40.0, 20.0,  0.0, -75.0],
    [ -75.0,-20.0, 10.0, 20.0, 20.0, 10.0,-20.0, -75.0],
    [ -95.0,-50.0,-20.0,  0.0,  0.0,-20.0,-50.0, -95.0],
    [-150.0,-95.0,-75.0,-75.0,-75.0,-75.0,-95.0,-150.0],
];
const PSQT_KNIGHT_EG: [[f64;8];8]=[
    [ -75.0,-95.0,-75.0,-75.0,-75.0,-75.0,-95.0, -75.0],
    [ -47.5,-25.0,-10.0,  0.0,  0.0,-10.0,-25.0, -47.5],
    [ -37.5,-10.0,  5.0, 10.0, 10.0,  5.0,-10.0, -37.5],
    [ -37.5,  0.0, 10.0, 20.0, 20.0, 10.0,  0.0, -37.5],
    [ -37.5,  0.0, 10.0, 20.0, 20.0, 10.0,  0.0, -37.5],
    [ -37.5,-10.0,  5.0, 10.0, 10.0,  5.0,-10.0, -37.5],
    [ -47.5,-25.0,-10.0,  0.0,  0.0,-10.0,-25.0, -47.5],
    [ -75.0,-95.0,-75.0,-75.0,-75.0,-75.0,-95.0, -75.0],
];
//Bishop constants
const BISHOP_PIECE_VALUE_EG: f64 = 510.0;
const BISHOP_PIECE_VALUE_MG: f64 = 510.0;
const BISHOP_PAIR_BONUS:f64 =50.0;
const DIAGONALLY_ADJACENT_SQUARES_WITH_OWN_PAWNS:[f64;5]= [30.0,15.0,0.0,-15.0,-30.0];

const PSQT_BISHOP_MG: [[f64;8];8]=[
    [ -50.0,-10.0,-10.0,-30.0,-30.0,-10.0,-10.0, -50.0],
    [ -30.0, 10.0, 15.0,  0.0,  0.0, 15.0, 10.0, -30.0],
    [ -10.0, 10.0, 15.0, 10.0, 10.0, 15.0, 10.0, -10.0],
    [ -10.0, 15.0, 20.0, 25.0, 25.0, 20.0,  0.0, -10.0],
    [ -10.0, 10.0, 20.0, 25.0, 25.0, 20.0,  0.0, -10.0],
    [ -10.0, 10.0, 15.0, 10.0, 10.0, 15.0, 10.0, -10.0],
    [ -30.0, 10.0, 15.0,  0.0,  0.0, 15.0, 10.0, -30.0],
    [ -50.0,-10.0,-10.0,-30.0,-30.0,-10.0,-10.0, -50.0],
];

const PSQT_BISHOP_EG: [[f64;8];8]=[
    [ -50.0, -30.0, -30.0, -20.0, -20.0, -30.0, -30.0, -50.0],
    [ -30.0, -10.0, -10.0,   5.0,   5.0, -10.0, -10.0, -30.0],
    [ -20.0,   0.0,   0.0,  12.0,  12.0,   0.0,  0.0,  -20.0],
    [ -20.0,   0.0,   0.0,  12.0,  12.0,   0.0,  0.0,  -20.0],
    [ -20.0,   0.0,   0.0,  12.0,  12.0,   0.0,  0.0,  -20.0],
    [ -20.0,   0.0,   0.0,  12.0,  12.0,   0.0,  0.0,  -20.0],
    [ -30.0, -10.0, -10.0,   5.0,   5.0, -10.0, -10.0, -30.0],
    [ -50.0, -30.0, -30.0, -20.0, -20.0, -30.0, -30.0, -50.0],
];
//Rook constants
const ROOK_PIECE_VALUE_MG: f64= 710.0;
const ROOK_PIECE_VALUE_EG: f64= 920.0;
const ROOK_ON_OPEN_FILE_BONUS:f64=20.0;
const ROOK_ON_SEVENTH:f64=10.0;
//Queen constants
const QUEEN_PIECE_VALUE_MG: f64= 1500.0;
const QUEEN_PIECE_VALUE_EG: f64= 1600.0;
//King constants
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
//Missing center control
//Missing square control
//Missing more analysis on king
pub fn eval_game_state(g: &GameState)->f64 {
    //White
    let w_pawns = g.pieces[0][0];
    let w_knights = g.pieces[1][0];
    let w_bishops = g.pieces[2][0];
    let w_rooks = g.pieces[3][0];
    let w_queens = g.pieces[4][0];
    let w_king = g.pieces[5][0];

    let white_pieces = w_pawns | w_knights | w_bishops | w_rooks | w_queens | w_king;
    //Black
    let b_pawns = g.pieces[0][1];
    let b_knights = g.pieces[1][1];
    let b_bishops = g.pieces[2][1];
    let b_rooks = g.pieces[3][1];
    let b_queens = g.pieces[4][1];
    let b_king = g.pieces[5][1];

    let black_pieces = b_pawns | b_knights | b_bishops | b_rooks | b_queens | b_king;
    let mut mg_eval = 0.0;
    let mut eg_eval = 0.0;

    let white_pawn_attacks=movegen::w_pawn_west_targets(w_pawns)|movegen::w_pawn_east_targets(w_pawns);
    let black_pawn_attacks=movegen::b_pawn_west_targets(b_pawns)|movegen::b_pawn_east_targets(b_pawns);
    let all_pawns=w_pawns|b_pawns;
    //Pawns
    {
        //White general bitboards
        let w_pawns_front_span = bitboards::w_front_span(w_pawns);
        let w_pawns_west_attack_front_span = bitboards::west_one(w_pawns_front_span);
        let w_pawns_east_attack_front_span = bitboards::east_one(w_pawns_front_span);
        let w_pawns_attack_span = w_pawns_east_attack_front_span | w_pawns_west_attack_front_span;
        let w_pawns_all_front_spans = w_pawns_front_span | w_pawns_attack_span;
        //Black general bitboards
        let b_pawns_front_span = bitboards::b_front_span(b_pawns);
        let b_pawns_west_attack_front_span = bitboards::west_one(b_pawns_front_span);
        let b_pawns_east_attack_front_span = bitboards::east_one(b_pawns_front_span);
        let b_pawns_attack_span = b_pawns_east_attack_front_span | b_pawns_west_attack_front_span;
        let b_pawns_all_front_spans = b_pawns_front_span | b_pawns_attack_span;

        let p_w_eval = pawn_eval_white(w_pawns, b_pawns, w_pawns_front_span, w_pawns_attack_span, b_pawns_all_front_spans, black_pieces,black_pawn_attacks);
        let p_b_eval = pawn_eval_black(b_pawns, w_pawns, b_pawns_front_span, b_pawns_attack_span, w_pawns_all_front_spans, white_pieces,white_pawn_attacks);
        if VERBOSE {
            println!("MG Sum: {} + {} - {} = {}", mg_eval, p_w_eval.0, p_b_eval.0, mg_eval + p_w_eval.0 - p_b_eval.0);
            println!("EG Sum: {} + {} - {} = {}", eg_eval, p_w_eval.1, p_b_eval.1, eg_eval + p_w_eval.1 - p_b_eval.1);
        }
        mg_eval += p_w_eval.0 - p_b_eval.0;
        eg_eval += p_w_eval.1 - p_b_eval.1;
    }
    //Knights
    {
        println!("White:");
        let k_w_eval= knight_eval(w_knights,white_pawn_attacks,w_pawns,b_pawns);
        println!("Black:");
        let k_b_eval= knight_eval(b_knights,black_pawn_attacks,b_pawns,w_pawns);
        if VERBOSE {
            println!("MG Sum: {} + {} - {} = {}", mg_eval, k_w_eval.0, k_b_eval.0, mg_eval + k_w_eval.0 - k_b_eval.0);
            println!("EG Sum: {} + {} - {} = {}", eg_eval, k_w_eval.1, k_b_eval.1, eg_eval + k_w_eval.1 - k_b_eval.1);
        }
        mg_eval += k_w_eval.0- k_b_eval.0;
        eg_eval += k_w_eval.1- k_b_eval.1;

    }
    //Bishops
    {
        println!("White: ");
        let b_w_eval=bishop_eval(w_bishops,w_pawns);
        println!("Black: ");
        let b_b_eval= bishop_eval(b_bishops,b_pawns);
        if VERBOSE {
            println!("MG Sum: {} + {} - {} = {}", mg_eval, b_w_eval.0, b_b_eval.0, mg_eval + b_w_eval.0 - b_b_eval.0);
            println!("EG Sum: {} + {} - {} = {}", eg_eval, b_w_eval.1, b_b_eval.1, eg_eval + b_w_eval.1 - b_b_eval.1);
        }
        mg_eval += b_w_eval.0- b_b_eval.0;
        eg_eval += b_w_eval.1- b_b_eval.1;
    }
    //Rooks
    {
        println!("White: ");
        let r_w_eval=rook_eval(w_rooks,all_pawns,true);
        println!("Black: ");
        let r_b_eval= rook_eval(b_rooks,all_pawns,false);
        if VERBOSE {
            println!("MG Sum: {} + {} - {} = {}", mg_eval, r_w_eval.0, r_b_eval.0, mg_eval + r_w_eval.0 - r_b_eval.0);
            println!("EG Sum: {} + {} - {} = {}", eg_eval, r_w_eval.1, r_b_eval.1, eg_eval + r_w_eval.1 - r_b_eval.1);
        }
        mg_eval += r_w_eval.0- r_b_eval.0;
        eg_eval += r_w_eval.1- r_b_eval.1;
    }
    //Queen(s)
    {
        println!("White: ");
        let q_w_eval=queen_eval(w_queens);
        println!("Black: ");
        let q_b_eval= queen_eval(b_queens);
        if VERBOSE {
            println!("MG Sum: {} + {} - {} = {}", mg_eval, q_w_eval.0, q_b_eval.0, mg_eval + q_w_eval.0 - q_b_eval.0);
            println!("EG Sum: {} + {} - {} = {}", eg_eval, q_w_eval.1, q_b_eval.1, eg_eval + q_w_eval.1 - q_b_eval.1);
        }
        mg_eval += q_w_eval.0- q_b_eval.0;
        eg_eval += q_w_eval.1- q_b_eval.1;
    }
    //King Safety
    {
        println!("White: ");
        let k_w_eval=king_eval(w_king,true,w_pawns,b_pawns);
        println!("Black: ");
        let k_b_eval= king_eval(b_king,false,b_pawns,w_pawns);
        if VERBOSE {
            println!("MG Sum: {} + {} - {} = {}", mg_eval, k_w_eval.0, k_b_eval.0, mg_eval + k_w_eval.0 - k_b_eval.0);
            println!("EG Sum: {} + {} - {} = {}", eg_eval, k_w_eval.1, k_b_eval.1, eg_eval + k_w_eval.1 - k_b_eval.1);
        }
        mg_eval += k_w_eval.0- k_b_eval.0;
        eg_eval += k_w_eval.1- k_b_eval.1;
    }

    //Phase
    let mg_limit= 9100.0;
    let eg_limit= 2350.0;
    let mut npm= (w_queens|b_queens).count_ones()as f64 *QUEEN_PIECE_VALUE_MG+
            (w_bishops|b_bishops).count_ones()as f64 *BISHOP_PIECE_VALUE_MG+
            (w_rooks|b_rooks).count_ones()as f64 *ROOK_PIECE_VALUE_MG+
            (w_knights|b_knights).count_ones()as f64 *KNIGHT_PIECE_VALUE_MG;
    if npm<eg_limit{
        npm=eg_limit;
    }
    if npm>mg_limit{
        npm=mg_limit;
    }
    let phase= (npm-eg_limit)*128.0/(mg_limit-eg_limit);
    let res= (mg_eval*phase+eg_eval*(128.0-phase))/128.0;
    if VERBOSE{
        println!("Phase: {}",phase);
        println!("=> ({} * {} + {}*(128-{}))/128={}",mg_eval,phase,eg_eval,phase,res);
    }
    res/100.0
}
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
    let mg= mg_psqt+amount_of_knights as f64 * KNIGHT_PIECE_VALUE_MG+supported_knights as f64*KNIGHT_SUPPORTED_BY_PAWN+KNIGHT_VALUE_WITH_PAWNS[pawns_insg];
    let eg= eg_psqt+amount_of_knights as f64 * KNIGHT_PIECE_VALUE_EG+supported_knights as f64*KNIGHT_SUPPORTED_BY_PAWN+KNIGHT_VALUE_WITH_PAWNS[pawns_insg];
    if VERBOSE{
        println!("------------------------------------------------");
        println!("\tKnights-MidGame");
        println!("\t\tAmount of Knights:     \t{} -> {}",amount_of_knights,amount_of_knights as f64 * KNIGHT_PIECE_VALUE_MG);
        println!("\t\tSupported Knights:     \t{} -> {}",supported_knights,supported_knights as f64*KNIGHT_SUPPORTED_BY_PAWN);
        println!("\t\tKnight decreased Value:\t{}",KNIGHT_VALUE_WITH_PAWNS[pawns_insg]);
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

pub fn pawns_mg_linear_combination(mut pawns:u64,amount_of_pawns: u32, doubled_pawns: u32, isolated_pawns: u32, backwards_pawns: u32, mut passed_pawns: u64, mut passed_not_blocked: u64, is_white: bool) -> f64 {
    let mut res: f64 = amount_of_pawns as f64 * PAWN_PIECE_VALUE_MG + doubled_pawns as f64 * PAWN_DOUBLED_VALUE_MG + isolated_pawns as f64 * PAWN_ISOLATED_VALUE_MG +
        backwards_pawns as f64 * PAWN_BACKWARD_VALUE_MG;
    let passed_pawns_amt = passed_pawns.count_ones();
    let passed_pawns_nb_amt = passed_not_blocked.count_ones();
    let mut passer_score = 0.0;
    while passed_pawns != 0u64 {
        let idx = passed_pawns.trailing_zeros() as usize;
        passer_score += PAWN_PASSED_VALUES_MG[if is_white { idx / 8 } else { 7 - idx / 8 }];
        passed_pawns ^= 1u64 << idx;
    }
    let mut passer_not_blocked = 0.0;
    while passed_not_blocked != 0u64 {
        let idx = passed_not_blocked.trailing_zeros() as usize;
        passer_not_blocked += PAWN_PASSED_NOT_BLOCKED_VALUES_MG[if is_white { idx / 8 } else { 7 - idx / 8 }];
        passed_not_blocked ^= 1u64 << idx;
    }
    res += passer_score;
    res += passer_not_blocked;
    //PSQT
    let mut psqt=0.0;
    while pawns!=0u64{
        let mut idx= pawns.trailing_zeros() as usize;
        pawns ^= 1u64<<idx;
        if !is_white{
            idx=63-idx;
        }
        psqt+=PSQT_PAWN_MG[idx/8][idx%8];
    }
    res+=psqt;
    if VERBOSE {
        println!("------------------------------------------------");
        println!("\tPawns MidGame --{}", if is_white { "white" } else { "black" });
        println!("\t\tAmount of Pawns:         \t{} -> {}", amount_of_pawns, amount_of_pawns as f64 * PAWN_PIECE_VALUE_MG);
        println!("\t\tDoubled Pawns:           \t{} -> {}", doubled_pawns, doubled_pawns as f64 * PAWN_DOUBLED_VALUE_MG);
        println!("\t\tIsolated Pawns:          \t{} -> {}", isolated_pawns, isolated_pawns as f64 * PAWN_ISOLATED_VALUE_MG);
        println!("\t\tBackwards Pawns:         \t{} -> {}", backwards_pawns, backwards_pawns as f64 * PAWN_BACKWARD_VALUE_MG);
        println!("\t\tPassed Pawns:            \t{} -> {}", passed_pawns_amt, passer_score);
        println!("\t\tNot Blocked Passed Pawns:\t{} -> {}", passed_pawns_nb_amt, passer_not_blocked);
        println!("\t\tPSQT-Value:              \t{}",psqt);
        println!("\tSum: {}", res);
        println!("------------------------------------------------");
    }
    res
}

pub fn pawns_eg_linear_combination(mut pawns:u64,amount_of_pawns: u32, doubled_pawns: u32, isolated_pawns: u32, backwards_pawns: u32, mut passed_pawns: u64, mut passed_not_blocked: u64, is_white: bool) -> f64 {
    let mut res: f64 = amount_of_pawns as f64 * PAWN_PIECE_VALUE_EG + doubled_pawns as f64 * PAWN_DOUBLED_VALUE_EG + isolated_pawns as f64 * PAWN_ISOLATED_VALUE_EG +
        backwards_pawns as f64 * PAWN_BACKWARD_VALUE_EG;
    let passed_pawns_amt = passed_pawns.count_ones();
    let passed_pawns_nb_amt = passed_not_blocked.count_ones();
    let mut passer_score = 0.0;
    let mut passer_not_blocked = 0.0;
    while passed_pawns != 0u64 {
        let idx = passed_pawns.trailing_zeros() as usize;
        passer_score += PAWN_PASSED_VALUES_EG[if is_white { idx / 8 } else { 7 - idx / 8 }];
        passed_pawns ^= 1u64 << idx;
    }
    while passed_not_blocked != 0u64 {
        let idx = passed_not_blocked.trailing_zeros() as usize;
        passer_not_blocked += PAWN_PASSED_NOT_BLOCKED_VALUES_EG[if is_white { idx / 8 } else { 7 - idx / 8 }];
        passed_not_blocked ^= 1u64 << idx;
    }
    res += passer_score;
    res += passer_not_blocked;
    let mut psqt=0.0;
    while pawns!=0u64{
        let mut idx= pawns.trailing_zeros() as usize;
        pawns ^= 1u64<<idx;
        if !is_white{
            idx=63-idx;
        }
        psqt+=PSQT_PAWN_EG[idx/8][idx%8];
    }
    res+=psqt;
    if VERBOSE {
        println!("------------------------------------------------");
        println!("\tPawns EndGame --{}", if is_white { "white" } else { "black" });
        println!("\t\tAmount of Pawns:         \t{} -> {}", amount_of_pawns, amount_of_pawns as f64 * PAWN_PIECE_VALUE_EG);
        println!("\t\tDoubled Pawns:           \t{} -> {}", doubled_pawns, doubled_pawns as f64 * PAWN_DOUBLED_VALUE_EG);
        println!("\t\tIsolated Pawns:          \t{} -> {}", isolated_pawns, isolated_pawns as f64 * PAWN_ISOLATED_VALUE_EG);
        println!("\t\tBackwards Pawns:         \t{} -> {}", backwards_pawns, backwards_pawns as f64 * PAWN_BACKWARD_VALUE_EG);
        println!("\t\tPassed Pawns:            \t{} -> {}", passed_pawns_amt, passer_score);
        println!("\t\tNot Blocked Passed Pawns:\t{} -> {}", passed_pawns_nb_amt, passer_not_blocked);
        println!("\t\tPSQT-Value:              \t{}",psqt);
        println!("\tSum: {}", res);
        println!("------------------------------------------------");
    }
    res
}

pub fn pawn_eval_white(w_pawns: u64, b_pawns: u64, w_pawns_front_span: u64, w_pawn_attack_span: u64, b_pawns_all_front_spans: u64, enemy_pieces: u64,black_pawn_attacks:u64) -> (f64, f64) {
    let file_fill = bitboards::file_fill(w_pawns);
    //Evaluation parameters
    let amount_of_pawns = w_pawns.count_ones();
    let doubled_pawns = pawns_behind_own(w_pawns, w_pawns_front_span);
    let isolated_pawns = isolated_pawns(w_pawns, file_fill);
    let backwards_pawns = w_backwards(w_pawns, w_pawn_attack_span,black_pawn_attacks);
    //Doubled Pawns aren't doubled passed
    let (passed_pawns, passed_not_blocked) = w_passed_pawns(w_pawns&!bitboards::w_rear_span(w_pawns), b_pawns_all_front_spans, enemy_pieces);
    (pawns_mg_linear_combination(w_pawns,amount_of_pawns, doubled_pawns, isolated_pawns, backwards_pawns, passed_pawns, passed_not_blocked, true),
     pawns_eg_linear_combination(w_pawns,amount_of_pawns, doubled_pawns, isolated_pawns, backwards_pawns, passed_pawns, passed_not_blocked, true))
}

pub fn pawn_eval_black(b_pawns: u64, w_pawns: u64, b_pawns_front_span: u64, b_pawns_attack_span: u64, w_pawns_all_front_spans: u64, enemy_pieces: u64,white_pawn_attacks:u64) -> (f64, f64) {
    let file_fill = bitboards::file_fill(b_pawns);
    let amount_of_pawns = b_pawns.count_ones();
    let doubled_pawns = pawns_behind_own(b_pawns, b_pawns_front_span);
    let isolated_pawns = isolated_pawns(b_pawns, file_fill);
    let backwards_pawns = b_backwards(b_pawns, b_pawns_attack_span,white_pawn_attacks);
    //Doubled Pawns aren't doubled passed
    let (passed_pawns, passed_not_blocked) = b_passed_pawns(b_pawns&!bitboards::b_rear_span(b_pawns), w_pawns_all_front_spans, enemy_pieces);
    (pawns_mg_linear_combination(b_pawns,amount_of_pawns, doubled_pawns, isolated_pawns, backwards_pawns, passed_pawns, passed_not_blocked, false),
     pawns_eg_linear_combination(b_pawns,amount_of_pawns, doubled_pawns, isolated_pawns, backwards_pawns, passed_pawns, passed_not_blocked, false))
}

pub fn w_backwards(w_pawns: u64, w_pawn_attack_span: u64,black_pawn_attacks:u64) -> u32 {
    let stops = w_pawns << 8;
    (stops & black_pawn_attacks & !w_pawn_attack_span).count_ones()
}

pub fn b_backwards(b_pawns: u64, b_pawn_attack_span: u64,white_pawn_attacks:u64) -> u32 {
    let stops = b_pawns >> 8;
    (stops & white_pawn_attacks & !b_pawn_attack_span).count_ones()
}

pub fn pawns_behind_own(pawns: u64, front_span: u64) -> u32 {
    (pawns & front_span).count_ones()
}

pub fn isolated_pawns(pawns: u64, file_fill: u64) -> u32 {
    (pawns & !bitboards::west_one(file_fill) & !bitboards::east_one(file_fill)).count_ones()
}

pub fn w_passed_pawns(w_pawns: u64, b_pawns_all_front_spans: u64, enemy_pieces: u64) -> (u64, u64) {
    let mut passed_board = w_pawns & !b_pawns_all_front_spans;
    let passed_board_cl = passed_board.clone();
    let mut passed_not_blocked = 0u64;
    while passed_board != 0u64 {
        let idx = passed_board.trailing_zeros() as usize;
        let piece = 1u64 << idx;
        if bitboards::w_front_span(piece) & enemy_pieces == 0u64 {
            passed_not_blocked |= piece;
        }
        passed_board ^= piece;
    }
    (passed_board_cl, passed_not_blocked)
}

pub fn b_passed_pawns(b_pawns: u64, w_pawns_all_front_spans: u64, enemy_pieces: u64) -> (u64, u64) {
    let mut passed_board = b_pawns & !w_pawns_all_front_spans;
    let passed_board_cl = passed_board.clone();
    let mut passed_not_blocked = 0u64;
    while passed_board != 0u64 {
        let idx = passed_board.trailing_zeros() as usize;
        let piece = 1u64 << idx;
        if bitboards::b_front_span(piece) & enemy_pieces == 0u64 {
            passed_not_blocked |= piece;
        }
        passed_board ^= piece;
    }
    (passed_board_cl, passed_not_blocked)
}