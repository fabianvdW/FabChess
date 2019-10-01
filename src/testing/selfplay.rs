use core::board_representation::game_state::{
    GameMove, GameMoveType, GameResult, GameState, PieceType, BISHOP, BLACK, KNIGHT, PAWN, QUEEN,
    ROOK, WHITE,
};
use core::board_representation::game_state_attack_container::GameStateAttackContainer;
use core::logging::Logger;
use core::move_generation::makemove::make_move;
use core::move_generation::movegen;
use core::search::timecontrol::TimeControl;
use core::testing::async_communication::{
    expect_output, expect_output_and_listen_for_info, print_command, write_stderr_to_log,
};
use core::testing::EndConditionInformation;
use core::testing::EngineReaction;
use core::testing::PlayTask;
use core::testing::TaskResult;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio_process::CommandExt;

pub fn play_game(mut task: PlayTask, mut error_log: Arc<Logger>) -> TaskResult {
    let mut movelist = movegen::MoveList::default();
    let mut attack_container = GameStateAttackContainer::default();
    //-------------------------------------------------------------
    //Set game up
    let opening_fen = task.opening.to_fen();
    attack_container.write_state(&task.opening);
    let agsi = movegen::generate_moves(&task.opening, false, &mut movelist, &attack_container);
    let mut history: Vec<GameState> = Vec::with_capacity(100);
    let mut status = check_end_condition(
        &task.opening,
        agsi.stm_haslegalmove,
        agsi.stm_incheck,
        &history,
    )
    .0;
    assert_eq!(status, GameResult::Ingame);
    history.push(task.opening.clone());
    let mut move_history: Vec<GameMove> = Vec::with_capacity(100);
    let mut endcondition = None;
    //-------------------------------------------------------------
    //Set tokio runtime up
    let mut runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime!");
    //-------------------------------------------------------------
    //Set players up
    let disq_closure = |p1| TaskResult::disq(task, p1, move_history, status);

    //Check uci and isready
    let (mut e1_process, mut e1_input, mut e1_output, mut e1_err) = task.engine1.get_handles();
    let reaction = task.engine1.valid_uci_isready_reaction(
        e1_input,
        e1_output,
        e1_err,
        &mut runtime,
        task.id,
        error_log,
    );
    match reaction {
        EngineReaction::DisqualifyEngine => return disq_closure(true),
        EngineReaction::ContinueGame(temp) => {
            e1_input = temp.0;
            e1_output = temp.1;
            e1_err = temp.2;
            error_log = temp.3;
        }
    }

    let (mut e2_process, mut e2_input, mut e2_output, mut e2_err) = task.engine2.get_handles();
    let reaction = task.engine2.valid_uci_isready_reaction(
        e2_input,
        e2_output,
        e2_err,
        &mut runtime,
        task.id,
        error_log,
    );
    match reaction {
        EngineReaction::DisqualifyEngine => return disq_closure(false),
        EngineReaction::ContinueGame(temp) => {
            e2_input = temp.0;
            e2_output = temp.1;
            e2_err = temp.2;
            error_log = temp.3;
        }
    }
    //-------------------------------------------------------------
    //Adjudications
    let mut draw_adjudication = 0;
    let mut win_adjudication = 0;
    let mut win_adjudication_for_p1 = true;

    while let GameResult::Ingame = status {
        //Request move
        let latest_state = &history[history.len() - 1];
        let player1_move = task.p1_is_white && latest_state.color_to_move == 0
            || !task.p1_is_white && latest_state.color_to_move == 1;
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
            "go wtime {} winc {} btime {} binc {} \n",
            if task.p1_is_white {
                player1_time
            } else {
                player2_time
            },
            if task.p1_is_white {
                player1_inc
            } else {
                player2_inc
            },
            if task.p1_is_white {
                player2_time
            } else {
                player1_time
            },
            if task.p1_is_white {
                player2_inc
            } else {
                player1_inc
            }
        ));
        let game_move: &GameMove;
        if player1_move {
        } else {
        }
        if player1_move {
            moves_p1 += 1;
            player1_input = print_command(&mut runtime, player1_input, position_string.clone());
            player1_input = print_command(&mut runtime, player1_input, "isready\n".to_owned());
            let output = expect_output("readyok".to_owned(), 200, player1_output, &mut runtime);
            if output.0.is_none() {
                error_log.log(
                    &format!(
                        "Player 1 didn't readyok after position description in game {}!\n",
                        task.id
                    ),
                    true,
                );
                error_log.log(
                    &format!("Position description is:\n{}\n", position_string),
                    true,
                );
                write_stderr_to_log(error_log, player1_stderr, &mut runtime);
                return TaskResult::disq(
                    true,
                    task.id,
                    task.opening_sequence,
                    move_history,
                    status,
                    task.p1_is_white,
                    engine1_name,
                    engine2_name,
                );
            }
            player1_output = output.1.unwrap();
            player1_input = print_command(&mut runtime, player1_input, go_string);
            let output = expect_output_and_listen_for_info(
                "bestmove".to_owned(),
                player1_time,
                player1_output,
                &mut runtime,
                "info".to_owned(),
            );
            if output.0.is_none() {
                error_log.log(
                    &format!(
                        "Player 1 didn't send bestmove in time in game {}! He had {}ms left!\nPosition:\n{}",
                        task.id, player1_time, position_string.clone()
                    ),
                    true,
                );
                write_stderr_to_log(error_log, player1_stderr, &mut runtime);
                return TaskResult::disq(
                    true,
                    task.id,
                    task.opening_sequence,
                    move_history,
                    status,
                    task.p1_is_white,
                    engine1_name,
                    engine2_name,
                );
            }
            player1_output = output.1.unwrap();
            if output.2 as u64 > player1_time {
                error_log.log(&format!("Mistake in Referee! Bestmove found but it took longer than time still left for player 1! Disqualifying player1 illegitimately in game {}\n", task.id), true);
                return TaskResult::disq(
                    true,
                    task.id,
                    task.opening_sequence,
                    move_history,
                    status,
                    task.p1_is_white,
                    engine1_name,
                    engine2_name,
                );
            }
            player1_time -= output.2 as u64;
            player1_time += player1_inc;

            //Parse the move
            let line = output.0.unwrap();
            let split_line: Vec<&str> = line.split_whitespace().collect();
            if split_line[0] == "bestmove" {
                let mv = GameMove::string_to_move(split_line[1]);
                let found_move = find_move(mv.0, mv.1, mv.2, &movelist);
                if found_move.is_none() {
                    error_log.log(
                        &format!("Player 1 sent illegal {} in game {}\n", line, task.id),
                        true,
                    );
                    write_stderr_to_log(error_log, player1_stderr, &mut runtime);
                    return TaskResult::disq(
                        true,
                        task.id,
                        task.opening_sequence,
                        move_history,
                        status,
                        task.p1_is_white,
                        engine1_name,
                        engine2_name,
                    );
                }
                game_move = found_move.unwrap();
            } else {
                error_log.log(&format!(
                    "Bestmove wasn't first argument after bestmove keyword! Disqualifiying player 1 in game {}\n",
                    task.id
                ), true);
                write_stderr_to_log(error_log, player1_stderr, &mut runtime);
                return TaskResult::disq(
                    true,
                    task.id,
                    task.opening_sequence,
                    move_history,
                    status,
                    task.p1_is_white,
                    engine1_name,
                    engine2_name,
                );
            }

            //Get additional info about player1 e.g. how deep he saw, nps, and his evaluation
            {
                let info = fetch_info(output.3.clone());
                let has_score = info.cp_score.is_some();
                if info.negative_mate_found | info.positive_mate_found {
                    draw_adjudication = 0;
                    if info.negative_mate_found {
                        if win_adjudication_for_p1 {
                            win_adjudication_for_p1 = false;
                            win_adjudication = 0;
                        }
                        win_adjudication += 1;
                    } else {
                        if !win_adjudication_for_p1 {
                            win_adjudication_for_p1 = true;
                            win_adjudication = 0;
                        }
                        win_adjudication += 1;
                    }
                } else {
                    if has_score {
                        let score = info.cp_score.unwrap();
                        if score.abs() <= 10 {
                            draw_adjudication += 1;
                        } else {
                            draw_adjudication = 0;
                        }
                        if score < -1000 {
                            if win_adjudication_for_p1 {
                                win_adjudication_for_p1 = false;
                                win_adjudication = 0;
                            }
                            win_adjudication += 1;
                        } else if score > 1000 {
                            if !win_adjudication_for_p1 {
                                win_adjudication_for_p1 = true;
                                win_adjudication = 0;
                            }
                            win_adjudication += 1;
                        } else {
                            win_adjudication = 0;
                        }
                    } else {
                        draw_adjudication = 0;
                        win_adjudication = 0;
                    }
                }
                if let Some(s) = info.depth {
                    average_depth_p1 += s as f64;
                }
                if let Some(nps) = info.nps {
                    average_nps_p1 += nps as f64;
                }
            }
        } else {
            moves_p2 += 1;
            player2_input = print_command(&mut runtime, player2_input, position_string.clone());
            player2_input = print_command(&mut runtime, player2_input, "isready\n".to_owned());
            let output = expect_output("readyok".to_owned(), 200, player2_output, &mut runtime);
            if output.0.is_none() {
                error_log.log(
                    &format!(
                        "Player 2 didn't readyok after position description in game {}!\n",
                        task.id
                    ),
                    true,
                );
                error_log.log(
                    &format!("Position description is:\n{}\n", position_string),
                    true,
                );
                write_stderr_to_log(error_log, player2_stderr, &mut runtime);
                return TaskResult::disq(
                    false,
                    task.id,
                    task.opening_sequence,
                    move_history,
                    status,
                    task.p1_is_white,
                    engine1_name,
                    engine2_name,
                );
            }
            player2_output = output.1.unwrap();
            player2_input = print_command(&mut runtime, player2_input, go_string);
            let output = expect_output_and_listen_for_info(
                "bestmove".to_owned(),
                player2_time,
                player2_output,
                &mut runtime,
                "info".to_owned(),
            );
            if output.0.is_none() {
                error_log.log(
                    &format!(
                        "Player 2 didn't send bestmove in time in game {}! He had {}ms left!\n",
                        task.id, player2_time
                    ),
                    true,
                );
                write_stderr_to_log(error_log, player2_stderr, &mut runtime);
                return TaskResult::disq(
                    false,
                    task.id,
                    task.opening_sequence,
                    move_history,
                    status,
                    task.p1_is_white,
                    engine1_name,
                    engine2_name,
                );
            }
            player2_output = output.1.unwrap();
            if output.2 as u64 > player2_time {
                error_log.log(&format!("Mistake in Referee! Bestmove found but it took longer than time still left for player 2! Disqualifying player1 illegitimately in game {}\n", task.id), true);
                return TaskResult::disq(
                    false,
                    task.id,
                    task.opening_sequence,
                    move_history,
                    status,
                    task.p1_is_white,
                    engine1_name,
                    engine2_name,
                );
            }
            player2_time -= output.2 as u64;
            player2_time += player2_inc;

            //Parse the move
            let line = output.0.unwrap();
            let split_line: Vec<&str> = line.split_whitespace().collect();
            if split_line[0] == "bestmove" {
                let mv = GameMove::string_to_move(split_line[1]);
                let found_move = find_move(mv.0, mv.1, mv.2, &movelist);
                if found_move.is_none() {
                    error_log.log(
                        &format!("Player 2 sent illegal {} in game {}\n", line, task.id),
                        true,
                    );
                    write_stderr_to_log(error_log, player2_stderr, &mut runtime);
                    return TaskResult::disq(
                        false,
                        task.id,
                        task.opening_sequence,
                        move_history,
                        status,
                        task.p1_is_white,
                        engine1_name,
                        engine2_name,
                    );
                }
                game_move = found_move.unwrap();
            } else {
                error_log.log(&format!(
                    "Bestmove wasn't first argument after bestmove keyword! Disqualifiying player 2 in game {}\n",
                    task.id
                ), true);
                write_stderr_to_log(error_log, player2_stderr, &mut runtime);
                return TaskResult::disq(
                    false,
                    task.id,
                    task.opening_sequence,
                    move_history,
                    status,
                    task.p1_is_white,
                    engine1_name,
                    engine2_name,
                );
            }

            //Get additional info about player2 e.g. how deep he saw, nps, and his evaluation
            {
                let info = fetch_info(output.3);
                let has_score = info.cp_score.is_some();
                if info.negative_mate_found | info.positive_mate_found {
                    draw_adjudication = 0;
                    if info.negative_mate_found {
                        if !win_adjudication_for_p1 {
                            win_adjudication_for_p1 = true;
                            win_adjudication = 0;
                        }
                        win_adjudication += 1;
                    } else {
                        if win_adjudication_for_p1 {
                            win_adjudication_for_p1 = false;
                            win_adjudication = 0;
                        }
                        win_adjudication += 1;
                    }
                } else {
                    if has_score {
                        let score = info.cp_score.unwrap();
                        if score.abs() <= 10 {
                            draw_adjudication += 1;
                        } else {
                            draw_adjudication = 0;
                        }
                        if score < -1000 {
                            if !win_adjudication_for_p1 {
                                win_adjudication_for_p1 = true;
                                win_adjudication = 0;
                            }
                            win_adjudication += 1;
                        } else if score > 1000 {
                            if win_adjudication_for_p1 {
                                win_adjudication_for_p1 = false;
                                win_adjudication = 0;
                            }
                            win_adjudication += 1;
                        } else {
                            win_adjudication = 0;
                        }
                    } else {
                        draw_adjudication = 0;
                        win_adjudication = 0;
                    }
                }
                if let Some(s) = info.depth {
                    average_depth_p2 += s as f64;
                }
                if let Some(nps) = info.nps {
                    average_nps_p2 += nps as f64;
                }
            }
        }
        //Make new state with move
        move_history.push(game_move.clone());
        let state = make_move(latest_state, game_move);
        if state.full_moves < 35 {
            draw_adjudication = 0;
        }
        attack_container.write_state(&state);
        let agsi = movegen::generate_moves(&state, false, &mut movelist, &attack_container);
        let check = check_end_condition(&state, agsi.stm_haslegalmove, agsi.stm_incheck, &history);
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
    print_command(&mut runtime, player1_input, "quit\n".to_owned());
    print_command(&mut runtime, player2_input, "quit\n".to_owned());
    thread::sleep(Duration::from_millis(20));
    let draw = status == GameResult::Draw;
    let p1_win = status == GameResult::WhiteWin && task.p1_is_white
        || status == GameResult::BlackWin && !task.p1_is_white;

    task.engine1.stats.divide(); //Make the mean of nps and deepth
    task.engine2.stats.divide();
    task.engine1.stats.avg_timeleft = task.engine1.time_control.time_left() as f64; //Set the time left
    task.engine2.stats.avg_timeleft = task.engine2.time_control.time_left() as f64;
    TaskResult {
        p1_name: engine1_name,
        p2_name: engine2_name,
        p1_white: task.p1_is_white,
        p1_won: p1_win,
        draw,
        p1_disq: false,
        p2_disq: false,
        endcondition,
        task_id: task.id,
        opening_sequence: task.opening_sequence,
        move_sequence: move_history,
        final_status: status,
        nps_p1: average_nps_p1 / f64::from(moves_p1),
        nps_p2: average_nps_p2 / f64::from(moves_p2),
        depth_p1: average_depth_p1 / f64::from(moves_p1),
        depth_p2: average_depth_p2 / f64::from(moves_p2),
        time_left_p1: player1_time as usize,
        time_left_p2: player2_time as usize,
    }
}

pub fn check_end_condition(
    game_state: &GameState,
    has_legal_moves: bool,
    in_check: bool,
    history: &[GameState],
) -> (GameResult, Option<EndConditionInformation>) {
    let enemy_win = if game_state.color_to_move == 0 {
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
    if game_state.pieces[PAWN][WHITE]
        | game_state.pieces[KNIGHT][WHITE]
        | game_state.pieces[BISHOP][WHITE]
        | game_state.pieces[ROOK][WHITE]
        | game_state.pieces[QUEEN][WHITE]
        | game_state.pieces[PAWN][BLACK]
        | game_state.pieces[KNIGHT][BLACK]
        | game_state.pieces[BISHOP][BLACK]
        | game_state.pieces[ROOK][BLACK]
        | game_state.pieces[QUEEN][BLACK]
        == 0u64
    {
        return (
            GameResult::Draw,
            Some(EndConditionInformation::DrawByMissingPieces),
        );
    }
    if game_state.half_moves >= 100 {
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
        if other.hash == state.hash {
            occ += 1;
        }
    }
    occ
}
