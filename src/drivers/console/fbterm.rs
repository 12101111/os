use super::CONSOLE;
use fbterm::*;
pub fn init_fbterm(fb: Framebuffer<'static,XRGB8888>) {
    if CONSOLE.lock().fbterm.is_some() {
        return;
    }
    let mut fbterm = Fbterm::new(fb, fbterm::Fonts::VGA8x14);
    fbterm.clear();
    CONSOLE.lock().fbterm = Some(fbterm);
}
