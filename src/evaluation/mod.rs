pub mod pawn_evaluation;
pub mod knight_evaluation;
pub mod bishop_evaluation;
pub mod rook_evaluation;
pub mod queen_evaluation;
pub mod king_evaluation;
const VERBOSE: bool = true;

use super::move_generation::movegen;
use super::board_representation::game_state::GameState;
use super::bitboards;
use self::pawn_evaluation::{pawn_eval_white,pawn_eval_black};
use self::knight_evaluation::{knight_eval,KNIGHT_PIECE_VALUE_MG};
use self::bishop_evaluation::{bishop_eval,BISHOP_PIECE_VALUE_MG};
use self::rook_evaluation::{rook_eval,ROOK_PIECE_VALUE_MG};
use self::queen_evaluation::{queen_eval,QUEEN_PIECE_VALUE_MG};
use self::king_evaluation::king_eval;

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

    //Calculate the Phase of the game
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

        let p_w_eval = pawn_eval_white(w_pawns, w_pawns_front_span, w_pawns_attack_span, b_pawns_all_front_spans, black_pieces,black_pawn_attacks);
        let p_b_eval = pawn_eval_black(b_pawns, b_pawns_front_span, b_pawns_attack_span, w_pawns_all_front_spans, white_pieces,white_pawn_attacks);
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
    let res= (mg_eval*phase+eg_eval*(128.0-phase))/128.0;
    if VERBOSE{
        println!("Phase: {}",phase);
        println!("=> ({} * {} + {}*(128-{}))/128={}",mg_eval,phase,eg_eval,phase,res);
    }
    res/100.0
}