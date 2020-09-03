use tuning::*;

pub fn main() {
    //Step 1. Load all positions from a file. Those positions should already be the q-searched positions.
    let mut stats = Statistics::default();
    let mut positions: Vec<LabelledGameState> = Vec::with_capacity(1);
    load_positions(
        POSITION_FILE,
        if POSITION_FILE.ends_with(".txt") {
            FileFormatSupported::OwnEncoding
        } else if POSITION_FILE.ends_with("epd") {
            FileFormatSupported::EPD
        } else {
            panic!("Invalid position file encoding!")
        },
        &mut positions,
        &mut stats,
    );
    println!(
        "Loaded file {} with {} positions!",
        POSITION_FILE,
        positions.len()
    );
    let mut tuner = Tuner {
        k: 1.1155,
        positions: init_texel_states(&mut positions),
        params: Parameters::default(),
    };
    println!("Start tuning for k");
    if OPTIMIZE_K {
        minimize_evaluation_error_fork(&mut tuner);
    }
    println!("Optimal K: {}", tuner.k);
    texel_tuning(&mut tuner);
}
