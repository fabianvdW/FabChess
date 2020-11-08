pub mod uci_engine;
pub mod uci_parser;

fn main() {
    let mut args = std::env::args();
    if args.nth(1) == Some("bench".to_owned()) {
        core_sdk::bench(args.nth(2).and_then(|depth| depth.parse::<usize>().ok()).unwrap_or(13));
    } else {
        uci_parser::parse_loop();
    }
}
