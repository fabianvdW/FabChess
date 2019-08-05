use core::logging::Logger;
use core::testing::queue::ThreadSafeString;
use std::io::BufReader;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::prelude::*;

pub fn print_command(
    runtime: &mut tokio::runtime::Runtime,
    input: tokio_process::ChildStdin,
    command: String,
) -> tokio_process::ChildStdin {
    let buf = command.as_bytes().to_owned();
    let fut = tokio_io::io::write_all(input, buf);
    runtime.block_on(fut).expect("Could not write!").0
}

pub fn expect_output(
    starts_with: String,
    time_frame: u64,
    output: tokio_process::ChildStdout,
    runtime: &mut tokio::runtime::Runtime,
) -> (Option<String>, Option<tokio_process::ChildStdout>, usize) {
    let lines_codec = tokio::codec::LinesCodec::new();
    let line_fut = tokio::codec::FramedRead::new(output, lines_codec)
        .filter(move |lines| lines.starts_with(&starts_with[..]))
        .into_future()
        .timeout(Duration::from_millis(time_frame));
    let before = Instant::now();
    let result = runtime.block_on(line_fut);
    let after = Instant::now();
    let dur = after.duration_since(before).as_millis() as usize;
    match result {
        Ok(s) => match s.0 {
            Some(str) => (Some(str), Some(s.1.into_inner().into_inner()), dur),
            _ => (None, None, dur),
        },
        Err(_) => (None, None, dur),
    }
}

pub fn expect_output_and_listen_for_info(
    starts_with: String,
    time_frame: u64,
    output: tokio_process::ChildStdout,
    runtime: &mut tokio::runtime::Runtime,
) -> (
    Option<String>,
    Option<tokio_process::ChildStdout>,
    usize,
    String,
) {
    let info_listener = Arc::new(ThreadSafeString::new());
    let info_listener_moved = info_listener.clone();
    let lines_codec = tokio::codec::LinesCodec::new();
    let line_fut = tokio::codec::FramedRead::new(output, lines_codec)
        .inspect(move |line| {
            if line.starts_with("info") {
                //println!("{}", line);
                info_listener_moved.push(&format!("{} ", line));
            }
        })
        .filter(move |lines| lines.starts_with(&starts_with[..]))
        .into_future()
        .timeout(Duration::from_millis(time_frame));
    let before = Instant::now();
    let result = runtime.block_on(line_fut);
    let after = Instant::now();
    let dur = after.duration_since(before).as_millis() as usize;
    match result {
        Ok(s) => match s.0 {
            Some(str) => (
                Some(str),
                Some(s.1.into_inner().into_inner().into_inner()),
                dur,
                info_listener.get_inner(),
            ),
            _ => (None, None, dur, info_listener.get_inner()),
        },
        Err(_) => (None, None, dur, info_listener.get_inner()),
    }
}

pub fn write_stderr_to_log(
    error_log: Arc<Logger>,
    stderr: tokio_process::ChildStderr,
    runtime: &mut tokio::runtime::Runtime,
) {
    error_log.log("StdERR of child: \n", true);
    let line_fut = tokio::io::lines(BufReader::new(stderr))
        .inspect(move |s| error_log.log(&format!("{}\n", s), true))
        .collect()
        .timeout(Duration::from_millis(100));
    let result = runtime.block_on(line_fut);
    match result {
        _ => {}
    };
}
