use crate::async_communication::{stderr_listener, write_all};
use crate::engine::{EndConditionInformation, EngineReaction, EngineStatus, PlayTask, TaskResult};
use core_sdk::board_representation::game_state::*;
use core_sdk::move_generation::makemove::make_move;
use core_sdk::move_generation::movegen;
use log::warn;
use std::time::Duration;
use tokio::process::Child;
use tokio::task::JoinHandle;
use tokio::time::delay_for;

pub async fn cleanup(mut e1: Child, mut e2: Child, e1_err: JoinHandle<()>, e2_err: JoinHandle<()>) {
    let _ = e1.kill();
    let _ = e2.kill();
    e1_err
        .await
        .unwrap_or_else(|msg| warn!("Could not join e1_err task: {}", msg));
    e2_err
        .await
        .unwrap_or_else(|msg| warn!("Could not join e2_err task: {}", msg));
}
pub async fn play_game(mut task: PlayTask) -> TaskResult {
    let mut movelist = movegen::MoveList::default();
    //-------------------------------------------------------------
    //Set game up
    let opening_fen = task.opening.to_fen();
    let agsi = movegen::generate_moves(&task.opening, false, &mut movelist);
    let mut history: Vec<GameState> = Vec::with_capacity(100);
    let mut status = check_end_condition(
        &task.opening,
        !movelist.move_list.is_empty(),
        agsi.stm_incheck,
        &history,
    )
    .0;
    assert_eq!(status, GameResult::Ingame);
    history.push(task.opening.clone());
    let mut move_history: Vec<GameMove> = Vec::with_capacity(100);
    let mut endcondition = None;
    //-------------------------------------------------------------
    //Set players up

    //Check uci and isready
    let (mut e1, mut e1_input, mut e1_output, e1_err) = task.engine1.get_handles().await;
    let e1_err = tokio::spawn(stderr_listener(e1_err));
    let reaction = task
        .engine1
        .valid_uci_isready_reaction(&mut e1_input, &mut e1_output, task.id)
        .await;
    if let EngineReaction::DisqualifyEngine = reaction {
        e1.kill()
            .unwrap_or_else(|msg| warn!("Unable to kill engine 1: {}", msg));
        e1_err.await.unwrap_or_else(|msg| {
            warn!("Could not join err reading task: {:?}", msg);
        });
        return TaskResult::disq(task, true, move_history, status);
    }

    let (e2, mut e2_input, mut e2_output, e2_err) = task.engine2.get_handles().await;
    let e2_err = tokio::spawn(stderr_listener(e2_err));
    let reaction = task
        .engine2
        .valid_uci_isready_reaction(&mut e2_input, &mut e2_output, task.id)
        .await;
    if let EngineReaction::DisqualifyEngine = reaction {
        cleanup(e1, e2, e1_err, e2_err).await;
        return TaskResult::disq(task, false, move_history, status);
    }
    //-------------------------------------------------------------
    //Adjudications
    let mut draw_adjudication = 0usize;
    let mut win_adjudication = 0usize;
    let mut win_adjudication_for_p1 = true;

    while let GameResult::Ingame = status {
        //Request move
        let latest_state = &history[history.len() - 1];
        let player1_move = task.p1_is_white && latest_state.get_color_to_move() == 0
            || !task.p1_is_white && latest_state.get_color_to_move() == 1;
        //Prepare position string
        let mut position_string = String::new();
        position_string.push_str("position fen ");
        position_string.push_str(&opening_fen);
        if !move_history.is_empty() {
            position_string.push_str(" moves ");
            for mv in &move_history {
                position_string.push_str(&format!("{:?} ", mv));
            }
        }
        position_string.push_str("\n");
        //Prepare go command
        let mut go_string = String::new();
        go_string.push_str(&format!(
            "go {} {}\n",
            if task.p1_is_white {
                task.engine1.time_control.to_go(true)
            } else {
                task.engine2.time_control.to_go(true)
            },
            if task.p1_is_white {
                task.engine2.time_control.to_go(false)
            } else {
                task.engine1.time_control.to_go(false)
            }
        ));
        let game_move: GameMove;
        if player1_move {
            let reaction = task
                .engine1
                .request_move(
                    &position_string,
                    &go_string,
                    &mut e1_input,
                    &mut e1_output,
                    task.id,
                    &movelist,
                )
                .await;
            let engine_status;
            match reaction {
                EngineReaction::DisqualifyEngine => {
                    cleanup(e1, e2, e1_err, e2_err).await;
                    return TaskResult::disq(task, true, move_history, status);
                }
                EngineReaction::ContinueGame(temp) => {
                    game_move = temp.0;
                    engine_status = temp.1;
                }
            }
            if let EngineStatus::ProclaimsNothing = &engine_status {
                draw_adjudication = 0;
                win_adjudication = 0;
            } else if let EngineStatus::ProclaimsDraw = &engine_status {
                win_adjudication = 0;
                draw_adjudication += 1;
            } else if let EngineStatus::ProclaimsWin = &engine_status {
                draw_adjudication = 0;
                if !win_adjudication_for_p1 {
                    win_adjudication = 1;
                } else {
                    win_adjudication += 1;
                }
                win_adjudication_for_p1 = true;
            } else if let EngineStatus::ProclaimsLoss = &engine_status {
                draw_adjudication = 0;
                if win_adjudication_for_p1 {
                    win_adjudication = 1;
                } else {
                    win_adjudication += 1;
                }
                win_adjudication_for_p1 = false;
            }
        } else {
            let reaction = task
                .engine2
                .request_move(
                    &position_string,
                    &go_string,
                    &mut e2_input,
                    &mut e2_output,
                    task.id,
                    &movelist,
                )
                .await;
            let engine_status;
            match reaction {
                EngineReaction::DisqualifyEngine => {
                    cleanup(e1, e2, e1_err, e2_err).await;
                    return TaskResult::disq(task, false, move_history, status);
                }
                EngineReaction::ContinueGame(temp) => {
                    game_move = temp.0;
                    engine_status = temp.1;
                }
            }
            if let EngineStatus::ProclaimsNothing = &engine_status {
                draw_adjudication = 0;
                win_adjudication = 0;
            } else if let EngineStatus::ProclaimsDraw = &engine_status {
                win_adjudication = 0;
                draw_adjudication += 1;
            } else if let EngineStatus::ProclaimsWin = &engine_status {
                draw_adjudication = 0;
                if win_adjudication_for_p1 {
                    win_adjudication = 1;
                } else {
                    win_adjudication += 1;
                }
                win_adjudication_for_p1 = false;
            } else if let EngineStatus::ProclaimsLoss = &engine_status {
                draw_adjudication = 0;
                if !win_adjudication_for_p1 {
                    win_adjudication = 1;
                } else {
                    win_adjudication += 1;
                }
                win_adjudication_for_p1 = true;
            }
        }

        //Make new state with move
        move_history.push(game_move);
        let state = make_move(latest_state, game_move);
        if state.get_full_moves() < 35 {
            draw_adjudication = 0;
        }
        let agsi = movegen::generate_moves(&state, false, &mut movelist);
        let check = check_end_condition(
            &state,
            !movelist.move_list.is_empty(),
            agsi.stm_incheck,
            &history,
        );
        history.push(state);
        status = check.0;
        endcondition = check.1;
        //Check for adjudication
        if let GameResult::Ingame = status {
            //Check adjudication values
            if draw_adjudication >= 20 {
                status = GameResult::Draw;
                endcondition = Some(EndConditionInformation::DrawByadjudication);
            } else if win_adjudication >= 10 {
                endcondition = Some(EndConditionInformation::MateByadjudication);
                if win_adjudication_for_p1 {
                    if task.p1_is_white {
                        status = GameResult::WhiteWin;
                    } else {
                        status = GameResult::BlackWin;
                    }
                } else if task.p1_is_white {
                    status = GameResult::BlackWin;
                } else {
                    status = GameResult::WhiteWin;
                }
            }
        }
    }

    //-------------------------------------------------------------
    //Cleanup players' processes
    write_all(&mut e1_input, "quit\n").await;
    write_all(&mut e2_input, "quit\n").await;
    delay_for(Duration::from_millis(20)).await;
    cleanup(e1, e2, e1_err, e2_err).await;

    let draw = status == GameResult::Draw;
    let p1_win = status == GameResult::WhiteWin && task.p1_is_white
        || status == GameResult::BlackWin && !task.p1_is_white;
    if draw {
        task.engine1.draws += 1;
        task.engine2.draws += 1;
    } else if p1_win {
        task.engine1.wins += 1;
        task.engine2.losses += 1;
    } else {
        task.engine1.losses += 1;
        task.engine2.wins += 1;
    }

    task.engine1.stats.divide(); //Make the mean of nps and deepth
    task.engine2.stats.divide();
    task.engine1.stats.avg_timeleft = task.engine1.time_control.time_left() as f64; //Set the time left
    task.engine2.stats.avg_timeleft = task.engine2.time_control.time_left() as f64;
    TaskResult {
        task,
        endcondition,
        move_sequence: move_history,
        final_status: status,
    }
}

pub fn check_end_condition(
    game_state: &GameState,
    has_legal_moves: bool,
    in_check: bool,
    history: &[GameState],
) -> (GameResult, Option<EndConditionInformation>) {
    let enemy_win = if game_state.get_color_to_move() == 0 {
        GameResult::BlackWin
    } else {
        GameResult::WhiteWin
    };
    if in_check && !has_legal_moves {
        return (enemy_win, Some(EndConditionInformation::Mate));
    }
    if !in_check && !has_legal_moves {
        return (GameResult::Draw, Some(EndConditionInformation::StaleMate));
    }

    //Missing pieces
    if game_state.get_pieces_from_side_without_king(WHITE)
        | game_state.get_pieces_from_side_without_king(BLACK)
        == 0u64
    {
        return (
            GameResult::Draw,
            Some(EndConditionInformation::DrawByMissingPieces),
        );
    }
    if game_state.get_half_moves() >= 100 {
        return (
            GameResult::Draw,
            Some(EndConditionInformation::HundredMoveDraw),
        );
    }
    if get_occurences(history, game_state) >= 2 {
        return (
            GameResult::Draw,
            Some(EndConditionInformation::ThreeFoldRepetition),
        );
    }

    (GameResult::Ingame, None)
}

pub fn get_occurences(history: &[GameState], state: &GameState) -> usize {
    let mut occ = 0;
    for other in history {
        if other.get_hash() == state.get_hash() {
            occ += 1;
        }
    }
    occ
}
