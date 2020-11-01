use std::thread;
use std::time::Instant;
use tuning::*;

pub fn main() {
    let mut args = std::env::args();
    let threads = if args.nth(1) == Some("t".to_owned()) {
        args.nth(0).unwrap().parse::<usize>().unwrap()
    } else {
        1
    };
    let t = thread::Builder::new()
        .stack_size(12 * 1024 * 1024)
        .spawn(move || {
            actual_main(threads);
        })
        .expect("Couldn't start thread");
    t.join().expect("Could not join thread");
}

pub fn actual_main(threads: usize) {
    //Step 1. Load all positions from a file. Those positions should already be the q-searched positions.
    let position_files = [
        "D:/FenCollection/Andrews/E12.33-1M-D12-Resolved.epd",
        "D:/FenCollection/Andrews/E12.41-1M-D12-Resolved.epd",
        "D:/FenCollection/Andrews/E12.46FRC-1250k-D12-1s-Resolved.epd",
        "D:/FenCollection/Andrews/E12.52-1M-D12-Resolved.epd",
    ];
    let position_files = ["D:/FenCollection/Andrews/a.epd"];
    let now = Instant::now();
    let mut positions = Vec::with_capacity(1);
    let mut thread_handles = Vec::with_capacity(threads);
    for i in 0..threads {
        let pos_per_thread = (position_files.len() as f64 / threads as f64).ceil() as usize;
        let my_pos = ((i * pos_per_thread).min(position_files.len()), ((i + 1) * pos_per_thread).min(position_files.len()));
        let my_pos = position_files[my_pos.0..my_pos.1].to_vec();
        println!("i: {}, {:?}", i, my_pos);
        thread_handles.push(
            thread::Builder::new()
                .stack_size(12 * 1024 * 1024)
                .spawn(move || {
                    let mut positions: Vec<TexelState> = Vec::with_capacity(1);
                    for &file in my_pos.iter() {
                        tuning::loading::PositionLoader::new(file, FileFormatSupported::EPD).load_texel_positions(&mut positions);
                    }
                    positions
                })
                .expect("Couldn't start thread"),
        );
    }
    for handle in thread_handles.into_iter() {
        positions.extend(handle.join().unwrap());
    }
    println!("Loaded {} positions!", positions.len());
    println!("Took {}ms", Instant::now().duration_since(now).as_millis());
    let mut tuner = Tuner {
        k: 1.1155,
        positions,
        params: Parameters::default(),
    };
    println!("Start tuning for k");
    if OPTIMIZE_K {
        minimize_evaluation_error_fork(&mut tuner);
    }
    println!("Optimal K: {}", tuner.k);
    unsafe { texel_tuning(tuner, threads) };
}
