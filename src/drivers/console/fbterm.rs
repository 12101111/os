use super::CONSOLE;
use fbterm::*;
pub fn init_fb(fb: *mut u8, width: usize, height: usize) {
    if CONSOLE.lock().fbterm.is_some() {
        return;
    }
    let background = XRGB8888::new(0, 0, 0, 0xA8);
    let foreground = XRGB8888::new(255, 0xA8, 0xA8, 0xA8);
    let fb = unsafe { Framebuffer::new(fb, width, height, background, foreground) };
    let mut fbterm = Fbterm::new(fb, fbterm::Fonts::VGA8x14);
    fbterm.clear();
    CONSOLE.lock().fbterm = Some(fbterm);
}
