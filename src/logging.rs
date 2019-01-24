pub const LOG_FILE: &str = "log.txt";
pub const LOG_TO_STDOUT: bool = false;

use std::fs::File;
use std::error::Error;
use std::sync::Mutex;
use std::io::Write;
lazy_static! {
    static ref LOG:Mutex<File> = Mutex::new(init_log());
}
pub fn init_log() -> File {
    match File::create(LOG_FILE) {
        Err(why) => panic!("Couldn't create file, {}", why.description()),
        Ok(file) => file,
    }
}

pub fn log(s: &str) {
    if LOG_TO_STDOUT{
        print!("{}",s);
    }else {
        LOG.lock().unwrap().write(s.as_bytes());
    }
}