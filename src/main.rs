#[macro_use]
extern crate lazy_static;
extern crate rand;

mod game_state;
mod misc;
mod bitboards;
mod movegen;
mod static_board_evaluation;

use self::game_state::GameState;
use std::time::Instant;

fn main() {
    let now = Instant::now();
    bitboards::init_all();
    println!("Should have initialized everything!");

    let new_now = Instant::now();
    println!("Initialization Time: {}ms", new_now.duration_since(now).as_secs() * 1000 + new_now.duration_since(now).subsec_millis() as u64);
    let now = Instant::now();

    //let g = GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let g= GameState::from_fen("r4rk1/2p1bppp/p1n5/1p1qPb2/3B4/1PPp1N2/1P3PPP/RB1Q1RK1 w - - 5 17");
    //let g= GameState::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    //let nodes = perft_div(&g, 2);
    //println!("{}", nodes);
    //misc::STD_FEN);
    //println!("{:}",GameState::from_fen(misc::STD_FEN));
    //println!("{:}",GameState::standard());
    let new_now = Instant::now();
    let time_passed = new_now.duration_since(now).as_secs() as f64 + new_now.duration_since(now).subsec_millis() as f64 / 1000.0;
    println!("Time: {}ms", new_now.duration_since(now).as_secs() * 1000 + new_now.duration_since(now).subsec_millis() as u64);
    //println!("NPS: {}", nodes as f64 / time_passed);

    println!("{}",static_board_evaluation::eval_game_state(&g));
}

pub fn perft_div(g: &GameState, depth: usize) -> u64 {
    let mut count = 0u64;
    let (valid_moves, in_check) = movegen::generate_moves(&g);
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
        let mut res = 0;
        let (valid_moves, incheck) = movegen::generate_moves(&g);
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
    }
}
