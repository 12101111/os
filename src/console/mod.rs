pub mod log;
pub mod uart;
pub mod uefifb;
use ::fbterm::*;
use core::fmt::{self, Write};
use spin::Mutex;
use uart_16550::UART;
use uefi::prelude::*;

static CONSOLE: Mutex<Console> = Mutex::new(Console {
    fbterm: None,
    uart: None,
});

pub struct Console {
    fbterm: Option<Fbterm<'static, RGBA8888>>,
    uart: Option<UART>,
}

pub fn _print(args: fmt::Arguments) {
    let mut console = CONSOLE.lock();
    if let Some(uart) = console.uart.as_mut() {
        uart.write_fmt(args).unwrap();
    }
    if let Some(fbterm) = console.fbterm.as_mut() {
        fbterm.write_fmt(args).unwrap();
    }
}

pub macro print($($arg:tt)*) {
    _print(format_args!($($arg)*))
}

pub macro println($($arg:tt)*) {
    print!("{}\r\n", format_args!($($arg)*))
}

pub fn init(st: &SystemTable<Boot>) {
    log::init();
    uart::init();
    uefifb::init(st);
}
