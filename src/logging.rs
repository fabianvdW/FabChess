pub const VERBOSE: bool = false;
pub const LOG_FILE: &str = "log.txt";
pub const LOG_TO_STDOUT: bool = false;

use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
lazy_static! {
    static ref LOG: Mutex<File> = Mutex::new(init_log());
}
pub fn init_log() -> File {
    match File::create(LOG_FILE) {
        Err(why) => panic!("Couldn't create file, {}", why.description()),
        Ok(file) => file,
    }
}

pub fn log(s: &str) {
    if LOG_TO_STDOUT {
        print!("{}", s);
    } else {
        let x = LOG.lock().unwrap().write(s.as_bytes());
        match x {
            Err(why) => panic!("Something went wrong, {}", why.description()),
            Ok(_) => {}
        };
    }
}
pub struct Logger {
    pub file: Mutex<File>,
}
impl Logger {
    pub fn new(path: &str, append: bool) -> Self {
        if !append {
            match std::fs::remove_file(path) {
                _ => {}
            }
        };
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .open(path)
            .unwrap();
        Logger {
            file: Mutex::new(file),
        }
    }

    pub fn log(&self, msg: &str, also_stdout: bool) {
        self.file
            .lock()
            .unwrap()
            .write(msg.as_bytes())
            .expect("Something went wrong when trying to write to this file");
        if also_stdout {
            print!("{}", msg);
        }
    }
}
