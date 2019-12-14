use super::println;
use ::log::{LevelFilter, Log, Metadata, Record};

struct ConsoleLogger;

static LOGGER: ConsoleLogger = ConsoleLogger;

pub fn init() {
    log::set_logger(&LOGGER).expect("Set logger failed");
    log::set_max_level(LevelFilter::Debug);
}

impl Log for ConsoleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        println!("[{:<5}] {}", record.level(), record.args());
    }
    fn flush(&self) {}
}
