use core_sdk::board_representation::game_state::{
    GameMove, GameMoveType, GameResult, GameState, Irreversible, WHITE,
};
use core_sdk::evaluation::eval_game_state;
use core_sdk::move_generation::makemove::{make_move, unmake_move};
use core_sdk::move_generation::movegen2;
use core_sdk::move_generation::movelist::MoveList;
use core_sdk::search::history::History;
use core_sdk::search::quiescence::{best_move_value, passes_delta_pruning, see, DELTA_PRUNING};
use core_sdk::search::reserved_memory::ReservedMoveList;
use core_sdk::search::SearchInstruction;
use core_sdk::search::{check_end_condition, check_for_draw, leaf_score};
use core_sdk::search::{MAX_SEARCH_DEPTH, STANDARD_SCORE};
use std::fs;
use tuning::loading::{
    load_positions, save_positions, FileFormatSupported, LabelledGameState, Statistics,
};

//const FEN_DIR: &str = "D:/FenCollection/Real";
const FEN_DIR: &str = "D:/FenCollection/Lichess";

fn main() {
    //2. Transform all FEN-Positions in Quiet positions
    //3. Save all positions just like loaded, all positions after q-search, all positions after q-search without stripped(no positions with >10 or <-10 eval)
    let mut positions: Vec<LabelledGameState> = Vec::with_capacity(8_000_000);
    let mut stats = Statistics::default();
    let paths = fs::read_dir(FEN_DIR).unwrap();
    for path in paths {
        load_positions(
            &format!("{}", path.unwrap().path().display()),
            FileFormatSupported::EPD,
            &mut positions,
            &mut stats,
        );
    }
    println!("{}", stats);
    println!("Positions: {}", positions.len());
    /*save_positions(
        &format!("{}/all_positions_noqsearch.txt", FEN_DIR),
        &positions,
    );*/

    let mut quiet_nonstripped: Vec<LabelledGameState> = Vec::with_capacity(positions.len());
    let mut quiet_stripped: Vec<LabelledGameState> = Vec::with_capacity(positions.len());

    let mut history = History::default();
    let mut move_list = ReservedMoveList::default();
    let mut see_buffer = vec![0i16; MAX_SEARCH_DEPTH];

    for position in positions {
        let mut other = position.game_state.clone();
        other.color_to_move = 1 - other.color_to_move;
        let (score, state) = stripped_q_search(
            -16000,
            16000,
            &mut position.game_state.clone(),
            if position.game_state.color_to_move == WHITE {
                1
            } else {
                -1
            },
            0,
            0,
            &mut history,
            &mut move_list,
            &mut see_buffer,
        );
        quiet_nonstripped.push(LabelledGameState {
            game_state: state.clone(),
            label: position.label,
        });
        if score.abs() < 1000 {
            quiet_stripped.push(LabelledGameState {
                game_state: state,
                label: position.label,
            });
        }
    }
    println!("Quiet positions: {}", quiet_nonstripped.len());
    println!("Quiet and stripped positions: {}", quiet_stripped.len());
    save_positions(
        &format!("{}/all_positions_qsearch.txt", FEN_DIR),
        &quiet_nonstripped,
    );
    /*save_positions(
        &format!("{}/all_positions_qsearchstripped.txt", FEN_DIR),
        &quiet_stripped,
    );*/
}

pub fn stripped_q_search(
    mut alpha: i16,
    beta: i16,
    game_state: &mut GameState,
    color: i16,
    current_depth: usize,
    depth_left: i16,
    history: &mut History,
    move_list: &mut ReservedMoveList,
    see_buffer: &mut Vec<i16>,
) -> (i16, GameState) {
    //Check for draw
    if let SearchInstruction::StopSearching(res) = check_for_draw(&game_state, history) {
        return (res, game_state.clone());
    }
    let incheck = game_state.in_check();
    let static_evaluation = eval_game_state(&game_state, -16000, 16000);
    //Standing pat pruning
    let stand_pat = static_evaluation.final_eval * color;
    if !incheck && stand_pat >= beta {
        return (stand_pat, game_state.clone());
    }
    if !incheck && stand_pat > alpha {
        alpha = stand_pat;
    }
    //Big Delta Pruning
    let diff = alpha - stand_pat - DELTA_PRUNING;
    if !incheck && diff > 0 && best_move_value(&game_state) < diff {
        return (stand_pat, game_state.clone());
    }
    history.push(
        game_state.irreversible.hash,
        game_state.irreversible.half_moves == 0,
    );

    let has_legal_move = make_moves(
        &game_state,
        &mut move_list.move_lists[current_depth],
        game_state.irreversible.phase.phase,
        stand_pat,
        alpha,
        see_buffer,
        incheck,
    );

    let mut current_max_score = if incheck { STANDARD_SCORE } else { stand_pat };
    let mut current_best_state: Option<GameState> = None;
    loop {
        let capture_move = move_list.move_lists[current_depth].highest_score();
        if capture_move.is_none() {
            break;
        }
        let (i, capture_move) = capture_move.unwrap();
        if capture_move.1.unwrap() < 0. {
            continue;
        }
        let capture_move = capture_move.0;
        move_list.move_lists[current_depth].move_list.remove(i);
        let mut irreversible = Irreversible::default();
        make_move(game_state, capture_move, &mut irreversible);
        let (score, other_state) = stripped_q_search(
            -beta,
            -alpha,
            game_state,
            -color,
            current_depth + 1,
            depth_left - 1,
            history,
            move_list,
            see_buffer,
        );
        unmake_move(game_state, capture_move, irreversible);

        if -score > current_max_score {
            current_max_score = -score;
            current_best_state = Some(other_state);
        }
        if -score >= beta {
            break;
        }
    }
    history.pop();
    let game_status = check_end_condition(&game_state, has_legal_move, incheck);
    if game_status != GameResult::Ingame {
        return (
            leaf_score(game_status, color, depth_left),
            game_state.clone(),
        );
    }
    if current_best_state.is_none() {
        return (stand_pat, game_state.clone());
    }
    (
        current_max_score,
        current_best_state.expect("Couldn't unwrap this"),
    )
}

pub fn make_moves(
    game_state: &GameState,
    move_list: &mut MoveList,
    phase: f64,
    stand_pat: i16,
    alpha: i16,
    see_buffer: &mut Vec<i16>,
    incheck: bool,
) -> bool {
    if incheck {
        movegen2::generate_pseudolegal_moves(&game_state, move_list);
        move_list
            .move_list
            .retain(|x| game_state.is_valid_move(x.0));
    } else {
        movegen2::generate_pseudolegal_captures(&game_state, move_list);
        move_list
            .move_list
            .retain(|x| game_state.is_valid_move(x.0));
    }
    for gmv in move_list.move_list.iter_mut() {
        let mv: GameMove = gmv.0;
        if let GameMoveType::EnPassant = mv.move_type {
            gmv.1 = Some(100.0);
        } else {
            if !incheck && !passes_delta_pruning(mv, phase, stand_pat, alpha) {
                gmv.1 = Some(-1.);
                continue;
            }
            if !incheck {
                let score = see(&game_state, mv, true, see_buffer);
                if score < 0 {
                    gmv.1 = Some(-1.);
                    continue;
                }
                gmv.1 = Some(f64::from(score));
            } else {
                gmv.1 = Some(0.);
            }
        }
    }
    if !move_list.move_list.is_empty() {
        true
    } else if !incheck {
        movegen2::generate_pseudolegal_quiets(game_state, move_list);
        !move_list.move_list.is_empty()
    } else {
        false
    }
}
