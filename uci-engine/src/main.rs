use core_sdk::logging::log;
use std::time::Instant;

pub mod uci_engine;
pub mod uci_parser;

fn main() {
    let now = Instant::now();
    core_sdk::bitboards::init_bitboards();
    core_sdk::move_generation::magic::init_magics();
    core_sdk::board_representation::zobrist_hashing::init_at_program_start();
    core_sdk::search::init_constants();
    log("Should have initialized everything!");

    let new_now = Instant::now();
    log(&format!(
        "Initialization Time: {}ms\n",
        new_now.duration_since(now).as_secs() * 1000
            + u64::from(new_now.duration_since(now).subsec_millis())
    ));
    let mut args = std::env::args();
    if args.nth(1) == Some("bench".to_owned()) {
        core_sdk::bench(
            args.nth(2)
                .and_then(|depth| depth.parse::<usize>().ok())
                .unwrap_or(13),
        );
    } else {
        uci_parser::parse_loop();
    }
}
