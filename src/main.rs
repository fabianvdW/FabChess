extern crate core;

use core::logging::log;
use std::time::Instant;
fn main() {
    let now = Instant::now();
    core::bitboards::init_bitboards();
    core::move_generation::magic::init_magics();
    core::board_representation::zobrist_hashing::init_at_program_start();
    core::search::init_constants();
    log("Should have initialized everything!");

    let new_now = Instant::now();
    log(&format!(
        "Initialization Time: {}ms\n",
        new_now.duration_since(now).as_secs() * 1000
            + u64::from(new_now.duration_since(now).subsec_millis())
    ));

    core::uci::uci_parser::parse_loop();
}

#[cfg(test)]
mod tests {
    use core::board_representation::game_state::GameState;
    use core::board_representation::game_state_attack_container::GameStateAttackContainer;
    use core::evaluation::psqt_evaluation::psqt;
    use core::misc::{GameParser, PGNParser, KING_BASE_PATH};
    use core::move_generation::makemove::make_move;
    use core::move_generation::movegen;
    use core::perft;
    use core::search::reserved_memory::ReservedAttackContainer;
    use core::search::reserved_memory::ReservedMoveList;
    use rand::Rng;
    use std::error::Error;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn fen_test() {
        let g = GameState::standard();
        assert_eq!(
            &g.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
        let fen = "4BR1N/1PPPQPp1/p1p2nPP/p1Pr1bp1/p1k3qB/1n1p2N1/1bP2pK1/5R2 w - - 0 1";
        let g = GameState::from_fen(fen);
        assert_eq!(&g.to_fen(), fen);
        let fen = "1nb1B3/bk1P2p1/p3PBp1/p3r1PP/1p3n1N/1pRNqP1P/p2p1RPK/3Q2r1 w - - 0 1";
        let g = GameState::from_fen(fen);
        assert_eq!(&g.to_fen(), fen);
        let fen = "8/1R2NP1N/pb1rPPK1/p1q1PpPQ/1Ppp3B/kpn2r2/nRBPP1p1/7b w - - 0 1";
        let g = GameState::from_fen(fen);
        assert_eq!(&g.to_fen(), fen);
        let fen = "3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1";
        let g = GameState::from_fen(fen);
        assert_eq!(&g.to_fen(), fen);
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        let g = GameState::from_fen(fen);
        assert_eq!(&g.to_fen(), fen);
        let fen = "8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1";
        let g = GameState::from_fen(fen);
        assert_eq!(&g.to_fen(), fen);
        let fen = "4BK2/2rPnppR/pPkp2Rn/2p4P/p3pBqP/4PPPN/2rPp3/b2b1N2 w - - 0 1";
        let g = GameState::from_fen(fen);
        assert_eq!(&g.to_fen(), fen);
        let fen = "6r1/B3P1p1/K1pP1kp1/Pp6/8/6N1/2P1p3/8 w - - 0 1";
        let g = GameState::from_fen(fen);
        assert_eq!(&g.to_fen(), fen);
        let fen = "3Rr3/1R1PP3/2P2k2/5n2/p2p1N2/1P6/4K3/1r6 w - - 0 1";
        let g = GameState::from_fen(fen);
        assert_eq!(&g.to_fen(), fen);
    }

    #[test]
    fn perft_test() {
        let mut movelist = ReservedMoveList::default();
        let mut attack_container = ReservedAttackContainer::default();
        #[rustfmt::skip]
        let cases = [
        (20,1,"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        (400,2,"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        (8902,3,"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        (197_281,4,"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        (4_865_609,5,"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        (119_060_324,6,"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        //https://gist.github.com/peterellisjones/8c46c28141c162d1d8a0f0badbc9cff9
        (8,1,"r6r/1b2k1bq/8/8/7B/8/8/R3K2R b QK - 3 2"),
        (8,1,"8/8/8/2k5/2pP4/8/B7/4K3 b - d3 5 3"),
        (19,1,"r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w QqKk - 2 2"),
        (5,1,"r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b QqKk - 3 2"),
        (44,1,"2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b QK - 3 2"),
        (39,1,"rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w QK - 3 9"),
        (9,1,"2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4"),
        (62379,3,"rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
        (89890,3,"r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"),
        (1_134_888,6,"3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1"),
        (1_015_133,6,"8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1"),
        (1_440_467,6,"8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1"),
        (661_072,6,"5k2/8/8/8/8/8/8/4K2R w K - 0 1"),
        (803_711,6,"3k4/8/8/8/8/8/8/R3K3 w Q - 0 1"),
        (1_274_206,4,"r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1"),
        (1_720_476,4,"r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1"),
        (3_821_001,6,"2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1"),
        (1_004_658,5,"8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1"),
        (217_342,6,"4k3/1P6/8/8/8/8/K7/8 w - - 0 1"),
        (92683,6,"8/P1k5/K7/8/8/8/8/8 w - - 0 1"),
        (2217,6,"K1k5/8/P7/8/8/8/8/8 w - - 0 1"),
        (567_584,7,"8/k1P5/8/1K6/8/8/8/8 w - - 0 1"),
        (23527,4,"8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1"),
        (48,1,"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"),
        (2039,2,"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"),
        (97862,3,"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"),
        (4_085_603,4,"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"),
        (14,1,"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"),
        (191,2,"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"),
        (2812,3,"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"),
        (43238,4,"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"),
        (674_624,5,"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"),
        (6,1,"r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
        (264,2,"r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
        (9467,3,"r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
        (422_333,4,"r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
        (6,1,"r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"),
        (264,2,"r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"),
        (9467,3,"r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"),
        (422_333,4,"r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"),
        (44,1,"rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
        (1486,2,"rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
        (62379,3,"rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
        (2_103_487,4,"rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
        (46,1,"r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"),
        (2079,2,"r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"),
        (89890,3,"r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"),
        (3_894_594,4,"r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"),
        //Hall of Fame Bugs!
        //63%9==63%7
        (4,1,"4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"),
        (198,2,"4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"),
        (7605,3,"4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"),
        (346_440,4,"4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"),
        (14_660_480,5,"4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"),
        //Pawn promotion capture when pinned
        (26,1,"6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"),
        (613,2,"6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"),
        (14277,3,"6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"),
        (345_436,4,"6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"),
        (7_804_316,5,"6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"),
        //Pawn en passant capture when pinned
        //Capture is possible when 1) on capture mask and 2) on ray or capturing the pinning piece
        (48,1,"3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"),
        (1221,2,"3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"),
        (54983,3,"3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"),
        (1_520_218,4,"3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"),
        (67_336_445,5,"3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"),
        //A case that passed all others before + the pgn test, because of its absurdity.
        (30,1,"NQbk2nr/1p1pp1bp/6p1/q3Pp2/3K4/8/PB4PP/R4B1R w - f6 0 24"),
        (885,2,"NQbk2nr/1p1pp1bp/6p1/q3Pp2/3K4/8/PB4PP/R4B1R w - f6 0 24"),
        (21360,3,"NQbk2nr/1p1pp1bp/6p1/q3Pp2/3K4/8/PB4PP/R4B1R w - f6 0 24"),
        (601_693,4,"NQbk2nr/1p1pp1bp/6p1/q3Pp2/3K4/8/PB4PP/R4B1R w - f6 0 24"),
        (16_183_274,5,"NQbk2nr/1p1pp1bp/6p1/q3Pp2/3K4/8/PB4PP/R4B1R w - f6 0 24"),
        //Missed that special en passant case for queens(got it for rooks earlier)
        (29,1,"8/4q3/6R1/4b3/4QpPk/5P2/8/6K1 b - g3 0 79"),
        (865,2,"8/4q3/6R1/4b3/4QpPk/5P2/8/6K1 b - g3 0 79"),
        (22609,3,"8/4q3/6R1/4b3/4QpPk/5P2/8/6K1 b - g3 0 79"),
        (685_012,4,"8/4q3/6R1/4b3/4QpPk/5P2/8/6K1 b - g3 0 79"),
        (17_252_119,5,"8/4q3/6R1/4b3/4QpPk/5P2/8/6K1 b - g3 0 79"),
        ];

        for case in cases.iter() {
            println!("{}", case.2);
            assert_eq!(
                case.0,
                perft(
                    &GameState::from_fen(case.2),
                    case.1,
                    &mut movelist,
                    &mut attack_container
                )
            );
        }
    }

    #[test]
    fn zobrist_hash_test() {
        //Tests incremental update of hash
        let mut movelist = movegen::MoveList::default();
        let mut attack_container = GameStateAttackContainer::default();
        let mut rng = rand::thread_rng();
        for _i in 0..10000 {
            let mut g = GameState::standard();
            for _j in 0..200 {
                assert_eq!(
                    g.hash,
                    GameState::calculate_zobrist_hash(
                        g.color_to_move,
                        g.pieces,
                        g.castle_white_kingside,
                        g.castle_white_queenside,
                        g.castle_black_kingside,
                        g.castle_black_queenside,
                        g.en_passant
                    )
                );
                attack_container.write_state(&g);
                let agsi = movegen::generate_moves(&g, false, &mut movelist, &attack_container);
                if !agsi.stm_haslegalmove {
                    break;
                }
                g = make_move(
                    &g,
                    movelist.move_list[rng.gen_range(0, movelist.counter)]
                        .as_ref()
                        .unwrap(),
                )
            }
        }
    }

    #[test]
    fn psqt_incremental_test() {
        let mut rng = rand::thread_rng();
        let mut movelist = movegen::MoveList::default();
        let mut attack_container = GameStateAttackContainer::default();
        let mut _eval = core::evaluation::EvaluationResult {
            phase: 0.,
            final_eval: 0,
            #[cfg(feature = "texel-tuning")]
            trace: core::tuning::trace::Trace::default(),
        };
        for _i in 0..100_000 {
            let mut g = GameState::standard();
            let (white_psqt_eval_mg, white_psqt_eval_eg) = psqt(true, &g.pieces, &mut _eval);
            let (black_psqt_eval_mg, black_psqt_eval_eg) = psqt(false, &g.pieces, &mut _eval);
            assert_eq!(g.psqt_mg, white_psqt_eval_mg - black_psqt_eval_mg);
            assert_eq!(g.psqt_eg, white_psqt_eval_eg - black_psqt_eval_eg);
            for _j in 0..200 {
                attack_container.write_state(&g);
                let agsi = movegen::generate_moves(&g, false, &mut movelist, &attack_container);
                if !agsi.stm_haslegalmove {
                    break;
                }
                g = make_move(
                    &g,
                    movelist.move_list[rng.gen_range(0, movelist.counter)]
                        .as_ref()
                        .unwrap(),
                );
                let (white_psqt_eval_mg, white_psqt_eval_eg) = psqt(true, &g.pieces, &mut _eval);
                let (black_psqt_eval_mg, black_psqt_eval_eg) = psqt(false, &g.pieces, &mut _eval);
                assert_eq!(g.psqt_mg, white_psqt_eval_mg - black_psqt_eval_mg);
                assert_eq!(g.psqt_eg, white_psqt_eval_eg - black_psqt_eval_eg);
            }
        }
    }

    #[test]
    #[ignore]
    fn pgn_test() {
        for path in &KING_BASE_PATH {
            let res = File::open(path);
            let file = match res {
                Err(why) => panic!("{}", why.description()),
                Ok(file) => file,
            };
            let reader = BufReader::new(file);
            let parser = GameParser {
                pgn_parser: PGNParser { reader },
                is_opening: false,
                opening_load_untilply: 0usize,
                move_list: movegen::MoveList::default(),
            };
            for _game in parser.into_iter() {
                //println!("{}", game.1);
            }
        }
    }
}
