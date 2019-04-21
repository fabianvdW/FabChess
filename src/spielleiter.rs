extern crate core;

use core::board_representation::game_state::GameState;

fn main() {
    println!("{}", "Ich bin der neue Spielleiter");
    let g = GameState::standard();
    println!("{}", g);
}
