use core_sdk::evaluation::eval_game_state;
use core_sdk::evaluation::nn_trace::NNTrace;
use core_sdk::evaluation::trace::Trace;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tuning::{load_positions, FileFormatSupported, LabelledGameState, Statistics};

fn main() {
    //let position_file = "D:/FenCollection/Lichess/lichess-quiet.txt";
    let position_file = "D:/FenCollection/Andrews/andrew_data_quiet.txt";
    let mut stats = Statistics::default();
    let mut positions = Vec::new();
    load_positions(
        position_file,
        FileFormatSupported::OwnEncoding,
        &mut positions,
        &mut stats,
    );
    positions.shuffle(&mut thread_rng());
    println!("Loaded {} positions", positions.len());
    //dump_nn_traces(&positions, "training_big.dat").unwrap();
    dump_nn_traces(&positions, "andrews_dataset.dat").unwrap();
    dump_nn_traces(&positions[..200_000_0], "andrews_dataset_subset.dat").unwrap();
}
pub const THREADS: usize = 4;
pub fn dump_nn_traces(positions: &[LabelledGameState], path: &str) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(false)
        .open(path)
        .unwrap();
    let now = Instant::now();
    let buf_writer = Arc::new(Mutex::new(BufWriter::new(file)));
    buf_writer
        .lock()
        .unwrap()
        .write_all(format!("{},Label\n", &Trace::csv_header()).as_bytes())?;
    let mut threads = Vec::new();
    let chunk_size = positions.len() / THREADS + 1;
    for chunk in positions.chunks(chunk_size) {
        let writer = Arc::clone(&buf_writer);
        let data = chunk.to_vec();
        threads.push(std::thread::spawn(move || {
            let mut nn_trace = NNTrace::new();
            let mut current_workload = String::new();
            for (i, pos) in data.iter().enumerate() {
                let eval = eval_game_state(&pos.game_state, &mut nn_trace);
                let trace = eval.trace;
                trace.dump_as_csv(&mut current_workload);
                current_workload.push_str(&format!(",{}\n", pos.label.to_string()));
                if i % 100000 == 0 || i == data.len() - 1 {
                    current_workload = current_workload.replace("[", "");
                    current_workload = current_workload.replace("]", "");
                    writer
                        .lock()
                        .unwrap()
                        .write_all(current_workload.as_bytes())
                        .unwrap();
                    current_workload = String::new();
                }
            }
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
    let duration = Instant::now().duration_since(now);
    println!("Took {}ms.", duration.as_millis());
    Ok(())
}
