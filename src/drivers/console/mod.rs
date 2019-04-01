pub mod fbterm;
pub mod uart;
use ::fbterm::*;
use core::fmt::{self, Write};
use log::{LevelFilter, Log, Metadata, Record};
use spin::Mutex;
use uart_16550::UART;

static CONSOLE: Mutex<Console> = Mutex::new(Console {
    fbterm: None,
    uart: None,
});

struct ConsoleLogger;

static LOGGER: ConsoleLogger = ConsoleLogger;

pub struct Console {
    fbterm: Option<Fbterm<'static, XRGB8888>>,
    uart: Option<UART>,
}

pub fn _print(args: fmt::Arguments) {
    let mut console = CONSOLE.lock();
    if console.fbterm.is_some() {
        console.fbterm.as_mut().unwrap().write_fmt(args).unwrap();
    }
    if console.uart.is_some() {
        console.uart.as_mut().unwrap().write_fmt(args).unwrap();
    }
}

pub fn init_logger() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Trace);
}

pub macro print($($arg:tt)*) {
    _print(format_args!($($arg)*))
}

pub macro println($($arg:tt)*) {
    print!("{}\n", format_args!($($arg)*))
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
