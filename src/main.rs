#[macro_use]
extern crate lazy_static;
extern crate rand;

mod board_representation;
mod misc;
mod bitboards;
mod move_generation;
mod evaluation;
mod logging;
mod search;

use self::board_representation::game_state::GameState;
use self::move_generation::movegen;
use std::time::Instant;
use logging::log;
use search::alphabeta::principal_variation_search;
use search::statistics;

fn main() {
    let now = Instant::now();
    bitboards::init_bitboards();
    move_generation::magic::init_magics();
    board_representation::zobrist_hashing::init_at_program_start();
    search::init_constants();
    log("Should have initialized everything!");

    let new_now = Instant::now();
    log(&format!("Initialization Time: {}ms\n", new_now.duration_since(now).as_secs() * 1000 + new_now.duration_since(now).subsec_millis() as u64));

    /*let now = Instant::now();

    let nodes = perft(&GameState::standard(),6);
    println!("{}",nodes);

    let new_now = Instant::now();
    let time_passed = new_now.duration_since(now).as_secs() as f64 + new_now.duration_since(now).subsec_millis() as f64 / 1000.0;
    println!("Time: {}ms", time_passed * 1000.0);
    println!("NPS: {}", nodes as f64 / time_passed);*/
    let state = GameState::from_fen("r3k2r/pbpnqpb1/1p1pp2p/6pn/2NPP3/2PB2B1/PP1NQPPP/R3K2R b KQkq - 5 12");
    //let state = GameState::from_fen("r6r/2k4p/1pq3p1/8/1P1Q1R2/5P2/P5PP/R4NK1 w - - 2 29");
    let mut ca = search::cache::Cache::new();
    let mut search = search::search::Search::new(&mut ca, &state);
    let pv = search.search(16);
    let score = pv.score;
    println!("{}", score);
    println!("{}", search.search_statistics);
    println!("{}", pv);
}

pub fn perft_div(g: &GameState, depth: usize) -> u64 {
    let mut count = 0u64;
    let (valid_moves, _in_check) = movegen::generate_moves(&g);
    for mv in valid_moves {
        let next_g = movegen::make_move(&g, &mv);
        let res = perft(&next_g, depth - 1);
        println!("{:?}: {}", mv, res);
        count += res;
    }
    count
}

pub fn perft(g: &GameState, depth: usize) -> u64 {
    if depth == 1 {
        let (vm, _ic) = movegen::generate_moves(&g);
        return vm.len() as u64;
    } else {
        if depth == 0 {
            return 1;
        }
        let mut res = 0;
        let (valid_moves, _incheck) = movegen::generate_moves(&g);
        for mv in valid_moves {
            res += perft(&movegen::make_move(&g, &mv), depth - 1);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::perft;
    use super::GameState;
    use crate::misc::{KING_BASE_PATH, GameParser, PGNParser};
    use std::io::BufReader;
    use std::fs::File;
    use std::error::Error;
    use rand::Rng;
    use super::movegen;

    #[test]
    fn fen_test() {
        let g = GameState::standard();
        assert_eq!(&g.to_fen(), "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
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
        assert_eq!(20, perft(&GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), 1));
        assert_eq!(400, perft(&GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), 2));
        assert_eq!(8902, perft(&GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), 3));
        assert_eq!(197281, perft(&GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), 4));
        assert_eq!(4865609, perft(&GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), 5));
        assert_eq!(119060324, perft(&GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), 6));
        //https://gist.github.com/peterellisjones/8c46c28141c162d1d8a0f0badbc9cff9
        assert_eq!(8, perft(&GameState::from_fen("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b QK - 3 2"), 1));
        assert_eq!(8, perft(&GameState::from_fen("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 5 3"), 1));
        assert_eq!(19, perft(&GameState::from_fen("r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w QqKk - 2 2"), 1));
        assert_eq!(5, perft(&GameState::from_fen("r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b QqKk - 3 2"), 1));
        assert_eq!(44, perft(&GameState::from_fen("2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b QK - 3 2"), 1));
        assert_eq!(39, perft(&GameState::from_fen("rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w QK - 3 9"), 1));
        assert_eq!(9, perft(&GameState::from_fen("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4"), 1));
        assert_eq!(62379, perft(&GameState::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"), 3));
        assert_eq!(89890, perft(&GameState::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"), 3));

        assert_eq!(1134888, perft(&GameState::from_fen("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1"), 6));
        assert_eq!(1015133, perft(&GameState::from_fen("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1"), 6));

        assert_eq!(1440467, perft(&GameState::from_fen("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1"), 6));
        assert_eq!(661072, perft(&GameState::from_fen("5k2/8/8/8/8/8/8/4K2R w K - 0 1"), 6));
        assert_eq!(803711, perft(&GameState::from_fen("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1"), 6));
        assert_eq!(1274206, perft(&GameState::from_fen("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1"), 4));
        assert_eq!(1720476, perft(&GameState::from_fen("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1"), 4));
        assert_eq!(3821001, perft(&GameState::from_fen("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1"), 6));
        assert_eq!(1004658, perft(&GameState::from_fen("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1"), 5));
        assert_eq!(217342, perft(&GameState::from_fen("4k3/1P6/8/8/8/8/K7/8 w - - 0 1"), 6));
        assert_eq!(92683, perft(&GameState::from_fen("8/P1k5/K7/8/8/8/8/8 w - - 0 1"), 6));
        assert_eq!(2217, perft(&GameState::from_fen("K1k5/8/P7/8/8/8/8/8 w - - 0 1"), 6));
        assert_eq!(567584, perft(&GameState::from_fen("8/k1P5/8/1K6/8/8/8/8 w - - 0 1"), 7));
        assert_eq!(23527, perft(&GameState::from_fen("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1"), 4));
        assert_eq!(48, perft(&GameState::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"), 1));
        assert_eq!(2039, perft(&GameState::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"), 2));
        assert_eq!(97862, perft(&GameState::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"), 3));
        assert_eq!(4085603, perft(&GameState::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"), 4));
        assert_eq!(14, perft(&GameState::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"), 1));
        assert_eq!(191, perft(&GameState::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"), 2));
        assert_eq!(2812, perft(&GameState::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"), 3));
        assert_eq!(43238, perft(&GameState::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"), 4));
        assert_eq!(674624, perft(&GameState::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"), 5));
        assert_eq!(6, perft(&GameState::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"), 1));
        assert_eq!(264, perft(&GameState::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"), 2));
        assert_eq!(9467, perft(&GameState::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"), 3));
        assert_eq!(422333, perft(&GameState::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"), 4));
        assert_eq!(6, perft(&GameState::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"), 1));
        assert_eq!(264, perft(&GameState::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"), 2));
        assert_eq!(9467, perft(&GameState::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"), 3));
        assert_eq!(422333, perft(&GameState::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"), 4));
        assert_eq!(44, perft(&GameState::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"), 1));
        assert_eq!(1486, perft(&GameState::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"), 2));
        assert_eq!(62379, perft(&GameState::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"), 3));
        assert_eq!(2103487, perft(&GameState::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"), 4));
        assert_eq!(46, perft(&GameState::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"), 1));
        assert_eq!(2079, perft(&GameState::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"), 2));
        assert_eq!(89890, perft(&GameState::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"), 3));
        assert_eq!(3894594, perft(&GameState::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"), 4));
        //Hall of Fame Bugs!
        //63%9==63%7
        assert_eq!(4, perft(&GameState::from_fen("4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"), 1));
        assert_eq!(198, perft(&GameState::from_fen("4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"), 2));
        assert_eq!(7605, perft(&GameState::from_fen("4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"), 3));
        assert_eq!(346440, perft(&GameState::from_fen("4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"), 4));
        assert_eq!(14660480, perft(&GameState::from_fen("4rb1k/1p2qb2/1pp4p/8/2P1BR2/5N2/5r1P/Q5RK b - - 3 34"), 5));
        //Pawn promotion capture when pinned
        assert_eq!(26, perft(&GameState::from_fen("6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"), 1));
        assert_eq!(613, perft(&GameState::from_fen("6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"), 2));
        assert_eq!(14277, perft(&GameState::from_fen("6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"), 3));
        assert_eq!(345436, perft(&GameState::from_fen("6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"), 4));
        assert_eq!(7804316, perft(&GameState::from_fen("6R1/2p2r2/2PP4/2b5/2B3p1/6k1/5p2/4BK2 b - - 0 1"), 5));
        //Pawn enpassant capture when pinned
        //Capture is possible when 1) on capture mask and 2) on ray or capturing the pinning piece
        assert_eq!(48, perft(&GameState::from_fen("3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"), 1));
        assert_eq!(1221, perft(&GameState::from_fen("3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"), 2));
        assert_eq!(54983, perft(&GameState::from_fen("3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"), 3));
        assert_eq!(1520218, perft(&GameState::from_fen("3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"), 4));
        assert_eq!(67336445, perft(&GameState::from_fen("3r4/6k1/pN1q2p1/Pp6/1PPpp3/4brPP/1Q2R1RK/8 b - c3 0 1"), 5));
    }

    #[test]
    fn zobrist_hash_test() {
        //Tests incremental update of hash
        let mut rng = rand::thread_rng();
        for _i in 0..10000 {
            let mut g = GameState::standard();
            for _j in 0..200 {
                assert_eq!(g.hash, GameState::calculate_zobrist_hash(g.color_to_move, g.pieces, g.castle_white_kingside, g.castle_white_queenside, g.castle_black_kingside, g.castle_black_queenside, g.en_passant));
                let legal_moves = movegen::generate_moves(&g).0;
                if legal_moves.len() == 0 {
                    break;
                }
                g = movegen::make_move(&g, &legal_moves[rng.gen_range(0, legal_moves.len())])
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
                Ok(file) => file
            };
            let reader = BufReader::new(file);
            let parser = GameParser { pgn_parser: PGNParser { reader } };
            for _game in parser.into_iter() {
                //println!("{}", game.1);
            }
        }
    }
}
