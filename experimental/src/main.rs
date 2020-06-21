use core_sdk::evaluation::eval_game_state;
use core_sdk::evaluation::nn_trace::NNTrace;
use core_sdk::evaluation::trace::Trace;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use tuning::{load_positions, FileFormatSupported, LabelledGameState, Statistics};

fn main() {
    //let position_file = "D:/FenCollection/Lichess/lichess-quiet.txt";
    let position_file = "D:/FabChess/experimental/test.txt";
    let mut stats = Statistics::default();
    let mut positions = Vec::new();
    load_positions(
        position_file,
        FileFormatSupported::OwnEncoding,
        &mut positions,
        &mut stats,
    );
    println!("Loaded {} positions", positions.len());
    //dump_nn_traces(&positions, "training_big.dat").unwrap();
    dump_nn_traces(&positions, "test.dat").unwrap();
}
pub fn dump_nn_traces(positions: &Vec<LabelledGameState>, path: &str) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(false)
        .open(path)
        .unwrap();
    let mut buf_writer = BufWriter::new(file);
    buf_writer.write_all(format!("{},Label\n", &Trace::csv_header()).as_bytes())?;
    for pos in positions {
        let eval = eval_game_state(&pos.game_state, &mut NNTrace::new());
        let trace = eval.trace;
        buf_writer
            .write_all(format!("{},{}\n", trace.dump_as_csv(), pos.label.to_string()).as_bytes())?;
    }
    Ok(())
}
