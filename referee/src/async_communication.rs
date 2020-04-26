use log::{info, warn};
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::time::timeout;

pub async fn write_all<T: AsyncWrite + Unpin>(stdin: &mut T, msg: &str) {
    stdin
        .write_all(msg.as_bytes())
        .await
        .unwrap_or_else(|msg| warn!("Could not write: {:?}", msg));
    stdin
        .flush()
        .await
        .unwrap_or_else(|msg| warn!("Could not flush: {:?}", msg));
}

pub async fn stderr_listener<T: AsyncBufRead + Unpin>(mut stderr: T) {
    let mut err = String::new();
    stderr.read_to_string(&mut err).await.unwrap_or_else(|msg| {
        warn!("Could not read from stderr of child: {}", msg);
        0
    });
    if err.len() > 0 {
        log_err(&err);
    }
}
pub fn log_err(msg: &str) {
    info!("StdERR of child:");
    info!("{}", msg);
}
pub async fn expect_output<T: AsyncBufRead + Unpin>(
    starts_with: &str,
    time_frame: u64,
    output: &mut T,
) -> (Option<String>, usize) {
    let res = expect_output_and_listen_for_info(starts_with, "", time_frame, output).await;
    (res.0, res.2)
}
pub async fn expect_output_and_listen_for_info<T: AsyncBufRead + Unpin>(
    starts_with: &str,
    info_starts_with: &str,
    time_frame: u64,
    output: &mut T,
) -> (Option<String>, String, usize) {
    let now = Instant::now();
    let mut info = String::new();
    let res = timeout(Duration::from_millis(time_frame), async {
        let mut reader = output.lines();
        while let Some(line) = reader.next_line().await.unwrap_or_else(|msg| {
            warn!("Could not read next line from reader: {:?}", msg);
            None
        }) {
            if line.starts_with(info_starts_with) {
                info.push_str(&format!("{}\n", line));
            }
            if line.starts_with(starts_with) {
                return Some(line);
            }
        }
        None
    })
    .await;
    let time_spent = Instant::now().duration_since(now).as_millis() as usize;
    if let Ok(s) = res {
        (s, info, time_spent)
    } else {
        (None, info, time_spent)
    }
}
