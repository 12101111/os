use fbterm::*;
use spin::Mutex;

static CONSOLE: Mutex<Option<Fbterm<XRGB8888>>> = Mutex::new(None);

pub fn init_fb(fb: *mut u8, width: usize, height: usize) {
    let background = XRGB8888::new(0, 0, 0, 0xA8);
    let foreground = XRGB8888::new(255, 0xA8, 0xA8, 0xA8);
    let fb = unsafe { Framebuffer::new(fb, width, height, background, foreground) };
    let mut fbterm = Fbterm::new(fb, fbterm::Fonts::VGA8x14);
    fbterm.clear();
    let mut console = CONSOLE.lock();
    if console.is_none() {
        *console = Some(fbterm)
    }
}

pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    CONSOLE.lock().as_mut().unwrap().write_fmt(args).unwrap();
}

#[cfg(not(test))]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::fb::_print(format_args!($($arg)*)));
}

#[cfg(not(test))]
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
