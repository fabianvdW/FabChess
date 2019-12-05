extern crate core;

use core::logging::log;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
    let mut args = std::env::args();
    if args.nth(1) == Some("bench".to_owned()) {
        bench(
            args.nth(2)
                .and_then(|depth| depth.parse::<usize>().ok())
                .unwrap_or(13),
        );
    } else {
        core::uci::uci_parser::parse_loop();
    }
}
//TAKEN FROM ETHEREAL
const BENCHMARKING_POSITIONS: [&str; 50] = [
    "r3k2r/2pb1ppp/2pp1q2/p7/1nP1B3/1P2P3/P2N1PPP/R2QK2R w KQkq a6 0 14",
    "4rrk1/2p1b1p1/p1p3q1/4p3/2P2n1p/1P1NR2P/PB3PP1/3R1QK1 b - - 2 24",
    "r3qbrk/6p1/2b2pPp/p3pP1Q/PpPpP2P/3P1B2/2PB3K/R5R1 w - - 16 42",
    "6k1/1R3p2/6p1/2Bp3p/3P2q1/P7/1P2rQ1K/5R2 b - - 4 44",
    "8/8/1p2k1p1/3p3p/1p1P1P1P/1P2PK2/8/8 w - - 3 54",
    "7r/2p3k1/1p1p1qp1/1P1Bp3/p1P2r1P/P7/4R3/Q4RK1 w - - 0 36",
    "r1bq1rk1/pp2b1pp/n1pp1n2/3P1p2/2P1p3/2N1P2N/PP2BPPP/R1BQ1RK1 b - - 2 10",
    "3r3k/2r4p/1p1b3q/p4P2/P2Pp3/1B2P3/3BQ1RP/6K1 w - - 3 87",
    "2r4r/1p4k1/1Pnp4/3Qb1pq/8/4BpPp/5P2/2RR1BK1 w - - 0 42",
    "4q1bk/6b1/7p/p1p4p/PNPpP2P/KN4P1/3Q4/4R3 b - - 0 37",
    "2q3r1/1r2pk2/pp3pp1/2pP3p/P1Pb1BbP/1P4Q1/R3NPP1/4R1K1 w - - 2 34",
    "1r2r2k/1b4q1/pp5p/2pPp1p1/P3Pn2/1P1B1Q1P/2R3P1/4BR1K b - - 1 37",
    "r3kbbr/pp1n1p1P/3ppnp1/q5N1/1P1pP3/P1N1B3/2P1QP2/R3KB1R b KQkq b3 0 17",
    "8/6pk/2b1Rp2/3r4/1R1B2PP/P5K1/8/2r5 b - - 16 42",
    "1r4k1/4ppb1/2n1b1qp/pB4p1/1n1BP1P1/7P/2PNQPK1/3RN3 w - - 8 29",
    "8/p2B4/PkP5/4p1pK/4Pb1p/5P2/8/8 w - - 29 68",
    "3r4/ppq1ppkp/4bnp1/2pN4/2P1P3/1P4P1/PQ3PBP/R4K2 b - - 2 20",
    "5rr1/4n2k/4q2P/P1P2n2/3B1p2/4pP2/2N1P3/1RR1K2Q w - - 1 49",
    "1r5k/2pq2p1/3p3p/p1pP4/4QP2/PP1R3P/6PK/8 w - - 1 51",
    "q5k1/5ppp/1r3bn1/1B6/P1N2P2/BQ2P1P1/5K1P/8 b - - 2 34",
    "r1b2k1r/5n2/p4q2/1ppn1Pp1/3pp1p1/NP2P3/P1PPBK2/1RQN2R1 w - - 0 22",
    "r1bqk2r/pppp1ppp/5n2/4b3/4P3/P1N5/1PP2PPP/R1BQKB1R w KQkq - 0 5",
    "r1bqr1k1/pp1p1ppp/2p5/8/3N1Q2/P2BB3/1PP2PPP/R3K2n b Q - 1 12",
    "r1bq2k1/p4r1p/1pp2pp1/3p4/1P1B3Q/P2B1N2/2P3PP/4R1K1 b - - 2 19",
    "r4qk1/6r1/1p4p1/2ppBbN1/1p5Q/P7/2P3PP/5RK1 w - - 2 25",
    "r7/6k1/1p6/2pp1p2/7Q/8/p1P2K1P/8 w - - 0 32",
    "r3k2r/ppp1pp1p/2nqb1pn/3p4/4P3/2PP4/PP1NBPPP/R2QK1NR w KQkq - 1 5",
    "3r1rk1/1pp1pn1p/p1n1q1p1/3p4/Q3P3/2P5/PP1NBPPP/4RRK1 w - - 0 12",
    "5rk1/1pp1pn1p/p3Brp1/8/1n6/5N2/PP3PPP/2R2RK1 w - - 2 20",
    "8/1p2pk1p/p1p1r1p1/3n4/8/5R2/PP3PPP/4R1K1 b - - 3 27",
    "8/4pk2/1p1r2p1/p1p4p/Pn5P/3R4/1P3PP1/4RK2 w - - 1 33",
    "8/5k2/1pnrp1p1/p1p4p/P6P/4R1PK/1P3P2/4R3 b - - 1 38",
    "8/8/1p1kp1p1/p1pr1n1p/P6P/1R4P1/1P3PK1/1R6 b - - 15 45",
    "8/8/1p1k2p1/p1prp2p/P2n3P/6P1/1P1R1PK1/4R3 b - - 5 49",
    "8/8/1p4p1/p1p2k1p/P2npP1P/4K1P1/1P6/3R4 w - - 6 54",
    "8/8/1p4p1/p1p2k1p/P2n1P1P/4K1P1/1P6/6R1 b - - 6 59",
    "8/5k2/1p4p1/p1pK3p/P2n1P1P/6P1/1P6/4R3 b - - 14 63",
    "8/1R6/1p1K1kp1/p6p/P1p2P1P/6P1/1Pn5/8 w - - 0 67",
    "1rb1rn1k/p3q1bp/2p3p1/2p1p3/2P1P2N/PP1RQNP1/1B3P2/4R1K1 b - - 4 23",
    "4rrk1/pp1n1pp1/q5p1/P1pP4/2n3P1/7P/1P3PB1/R1BQ1RK1 w - - 3 22",
    "r2qr1k1/pb1nbppp/1pn1p3/2ppP3/3P4/2PB1NN1/PP3PPP/R1BQR1K1 w - - 4 12",
    "2r2k2/8/4P1R1/1p6/8/P4K1N/7b/2B5 b - - 0 55",
    "6k1/5pp1/8/2bKP2P/2P5/p4PNb/B7/8 b - - 1 44",
    "2rqr1k1/1p3p1p/p2p2p1/P1nPb3/2B1P3/5P2/1PQ2NPP/R1R4K w - - 3 25",
    "r1b2rk1/p1q1ppbp/6p1/2Q5/8/4BP2/PPP3PP/2KR1B1R b - - 2 14",
    "6r1/5k2/p1b1r2p/1pB1p1p1/1Pp3PP/2P1R1K1/2P2P2/3R4 w - - 1 36",
    "rnbqkb1r/pppppppp/5n2/8/2PP4/8/PP2PPPP/RNBQKBNR b KQkq c3 0 2",
    "2rr2k1/1p4bp/p1q1p1p1/4Pp1n/2PB4/1PN3P1/P3Q2P/2RR2K1 w - f6 0 20",
    "3br1k1/p1pn3p/1p3n2/5pNq/2P1p3/1PN3PP/P2Q1PB1/4R1K1 w - - 0 23",
    "2r2b2/5p2/5k2/p1r1pP2/P2pB3/1P3P2/K1P3R1/7R w - - 23 93",
];
fn bench(depth: usize) {
    let cache = Arc::new(core::search::cache::Cache::with_size(8));
    let before_time = Instant::now();
    let mut nodes = 0;
    for position in BENCHMARKING_POSITIONS.iter() {
        let state = core::board_representation::game_state::GameState::from_fen(position);
        nodes += core::search::searcher::search_move(
            depth as i16,
            state,
            Vec::new(),
            Arc::new(AtomicBool::new(false)),
            Arc::clone(&cache),
            Arc::new(AtomicU64::new(0)),
            0,
            core::uci::uci_engine::UCIOptions {
                hash_size: 8,
                threads: 1,
                move_overhead: 0,
                debug_print: false,
                skip_ratio: core::search::searcher::DEFAULT_SKIP_RATIO,
            },
            core::search::timecontrol::TimeControl::Infinite,
        )
        .1
        .expect("Invalid benchmark!")
        .nodes_searched_sum
        .load(Ordering::Relaxed) as i32;
        cache.clear();
    }
    let dur = Instant::now().duration_since(before_time).as_millis();
    println!("Time: {}ms", dur);
    println!("Nodes: {}", nodes);
    println!("NPS: {:.0}", 1000. * nodes as f64 / dur as f64)
}
#[cfg(test)]
mod tests {
    use core::board_representation::game_state::GameState;
    use core::board_representation::game_state_attack_container::GameStateAttackContainer;
    use core::evaluation::phase::Phase;
    use core::evaluation::psqt_evaluation::psqt;
    use core::misc::KING_BASE_PATH;
    use core::move_generation::makemove::make_move;
    use core::move_generation::movegen;
    use core::perft;
    use core::pgn::pgn_reader::{GameParser, PGNParser};
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
            (20, 1, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            (400, 2, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            (8902, 3, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            (197_281, 4, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            (4_865_609, 5, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            (119_060_324, 6, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            //https://gist.github.com/peterellisjones/8c46c28141c162d1d8a0f0badbc9cff9
            (8, 1, "r6r/1b2k1bq/8/8/7B/8/8/R3K2R b QK - 3 2"),
            (8, 1, "8/8/8/2k5/2pP4/8/B7/4K3 b - d3 5 3"),
            (19, 1, "r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w QqKk - 2 2"),
            (5, 1, "r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b QqKk - 3 2"),
            (44, 1, "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b QK - 3 2"),
            (39, 1, "rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w QK - 3 9"),
            (9, 1, "2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4"),
            (62379, 3, "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
            (89890, 3, "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"),
            (1_134_888, 6, "3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1"),
            (1_015_133, 6, "8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1"),
            (1_440_467, 6, "8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1"),
            (661_072, 6, "5k2/8/8/8/8/8/8/4K2R w K - 0 1"),
            (803_711, 6, "3k4/8/8/8/8/8/8/R3K3 w Q - 0 1"),
            (1_274_206, 4, "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1"),
            (1_720_476, 4, "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1"),
            (3_821_001, 6, "2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1"),
            (1_004_658, 5, "8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1"),
            (217_342, 6, "4k3/1P6/8/8/8/8/K7/8 w - - 0 1"),
            (92683, 6, "8/P1k5/K7/8/8/8/8/8 w - - 0 1"),
            (2217, 6, "K1k5/8/P7/8/8/8/8/8 w - - 0 1"),
            (567_584, 7, "8/k1P5/8/1K6/8/8/8/8 w - - 0 1"),
            (23527, 4, "8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1"),
            (48, 1, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"),
            (2039, 2, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"),
            (97862, 3, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"),
            (4_085_603, 4, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"),
            (14, 1, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"),
            (191, 2, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"),
            (2812, 3, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"),
            (43238, 4, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"),
            (674_624, 5, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"),
            (6, 1, "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
            (264, 2, "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
            (9467, 3, "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
            (422_333, 4, "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
            (6, 1, "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"),
            (264, 2, "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"),
            (9467, 3, "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"),
            (422_333, 4, "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"),
            (44, 1, "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
            (1486, 2, "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
            (62379, 3, "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
            (2_103_487, 4, "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
            (46, 1, "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"),
            (2079, 2, "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"),
            (89890, 3, "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"),
            (3_894_594, 4, "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"),
            //Hall of Fame Bugs!
            //63%9==63%7
            (4, 1, "4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"),
            (198, 2, "4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"),
            (7605, 3, "4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"),
            (346_440, 4, "4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"),
            (14_660_480, 5, "4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"),
            //Pawn promotion capture when pinned
            (26, 1, "6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"),
            (613, 2, "6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"),
            (14277, 3, "6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"),
            (345_436, 4, "6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"),
            (7_804_316, 5, "6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"),
            //Pawn en passant capture when pinned
            //Capture is possible when 1) on capture mask and 2) on ray or capturing the pinning piece
            (48, 1, "3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"),
            (1221, 2, "3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"),
            (54983, 3, "3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"),
            (1_520_218, 4, "3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"),
            (67_336_445, 5, "3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"),
            //A case that passed all others before + the pgn test, because of its absurdity.
            (30, 1, "NQbk2nr/1p1pp1bp/6p1/q3Pp2/3K4/8/PB4PP/R4B1R w - f6 0 24"),
            (885, 2, "NQbk2nr/1p1pp1bp/6p1/q3Pp2/3K4/8/PB4PP/R4B1R w - f6 0 24"),
            (21360, 3, "NQbk2nr/1p1pp1bp/6p1/q3Pp2/3K4/8/PB4PP/R4B1R w - f6 0 24"),
            (601_693, 4, "NQbk2nr/1p1pp1bp/6p1/q3Pp2/3K4/8/PB4PP/R4B1R w - f6 0 24"),
            (16_183_274, 5, "NQbk2nr/1p1pp1bp/6p1/q3Pp2/3K4/8/PB4PP/R4B1R w - f6 0 24"),
            //Missed that special en passant case for queens(got it for rooks earlier)
            (29, 1, "8/4q3/6R1/4b3/4QpPk/5P2/8/6K1 b - g3 0 79"),
            (865, 2, "8/4q3/6R1/4b3/4QpPk/5P2/8/6K1 b - g3 0 79"),
            (22609, 3, "8/4q3/6R1/4b3/4QpPk/5P2/8/6K1 b - g3 0 79"),
            (685_012, 4, "8/4q3/6R1/4b3/4QpPk/5P2/8/6K1 b - g3 0 79"),
            (17_252_119, 5, "8/4q3/6R1/4b3/4QpPk/5P2/8/6K1 b - g3 0 79"),
        ];

        for case in cases.iter() {
            println!("{}", case.2);
            assert_eq!(
                case.0,
                perft(
                    &GameState::from_fen(case.2),
                    case.1,
                    &mut movelist,
                    &mut attack_container,
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
                        g.en_passant,
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
    fn phase_incremental() {
        let mut rng = rand::thread_rng();
        let mut movelist = movegen::MoveList::default();
        let mut attack_container = GameStateAttackContainer::default();
        for _i in 0..10_000 {
            let mut g = GameState::standard();
            assert!(
                (g.phase.phase - Phase::from_pieces(&g.pieces).phase).abs() < std::f64::EPSILON
            );
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
                assert!(
                    (g.phase.phase - Phase::from_pieces(&g.pieces).phase).abs() < std::f64::EPSILON
                );
            }
        }
    }
    #[test]
    fn psqt_incremental_test() {
        let mut rng = rand::thread_rng();
        let mut movelist = movegen::MoveList::default();
        let mut attack_container = GameStateAttackContainer::default();
        let mut _eval = core::evaluation::EvaluationResult {
            final_eval: 0,
            #[cfg(feature = "texel-tuning")]
            trace: core::tuning::trace::Trace::default(),
        };
        for _i in 0..100_000 {
            let mut g = GameState::standard();
            let w_psqt = psqt(true, &g.pieces, &mut _eval);
            let b_psqt = psqt(false, &g.pieces, &mut _eval);
            assert_eq!(g.psqt, w_psqt - b_psqt);
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
                let w_psqt = psqt(true, &g.pieces, &mut _eval);
                let b_psqt = psqt(false, &g.pieces, &mut _eval);
                assert_eq!(g.psqt, w_psqt - b_psqt);
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
                attack_container: GameStateAttackContainer::default(),
            };
            for _game in parser.into_iter() {
                //println!("{}", game.1);
            }
        }
    }
}
