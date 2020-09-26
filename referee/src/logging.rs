use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

pub struct FileLogger(Mutex<File>);
impl FileLogger {
    pub fn new(path: &str, append: bool) -> Self {
        if !append {
            let _ = std::fs::remove_file(path);
        };
        let file = OpenOptions::new().create(true).write(true).append(append).open(path).unwrap();
        FileLogger(Mutex::new(file))
    }
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_boxed_logger(Box::new(self)).map(|()| log::set_max_level(LevelFilter::Info))
    }
    pub fn dump_msg(&self, msg: &str) {
        self.0.lock().unwrap().write_all(msg.as_bytes()).expect("Could not log to file!");
    }
}
impl log::Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("{}::{} - {}\n", chrono::Utc::now(), record.level(), record.args());
            self.dump_msg(&msg);
            print!("{}", msg);
        }
    }

    fn flush(&self) {
        self.0.lock().unwrap().flush().expect("Could not flush in FileLogger");
    }
}
