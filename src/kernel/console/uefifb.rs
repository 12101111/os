use super::CONSOLE;
use fbterm::*;
use uefi::{prelude::*, proto::console::gop::GraphicsOutput};

pub fn init(st: &SystemTable<Boot>) {
    let bt = st.boot_services();
    let gop = bt
        .locate_protocol::<GraphicsOutput>()
        .expect_success("UEFI GOP not support");
    let gop = unsafe { &mut *gop.get() };
    set_mode(gop, st);
    let fb = gop.frame_buffer().as_mut_ptr();
    trace!("uefifb address: {:?}",fb);
    let info = gop.current_mode_info();
    let background = RGBA8888::new(0, 0, 0, 0xA8);
    let foreground = RGBA8888::new(255, 0xA8, 0xA8, 0xA8);
    let (w, h) = info.resolution();
    let fb = unsafe { Framebuffer::new(fb, w, h, info.stride(), background, foreground) };
    let mut fbterm = Fbterm::new(fb, fbterm::Fonts::VGA8x14);
    fbterm.clear();
    CONSOLE.lock().fbterm = Some(fbterm);
}

#[cfg(not(debug_assertions))]
use core::fmt::Write;
#[cfg(not(debug_assertions))]
use uefi::proto::console::text::{Input, Key};

#[cfg(not(debug_assertions))]
fn set_mode(gop: &mut GraphicsOutput, st: &SystemTable<Boot>) {
    let stdin = st.stdin();
    stdin.reset(false).expect_success("Reset stdin failed");
    stdin.wait_for_key_event();
    let stdout = st.stdout();
    stdout.clear().expect_success("Clear stdout failed");
    write!(stdout, "Select resolution:").expect("output failed");
    let modes = gop.modes();
    let mut mode_changed = None;
    'select: for mode in modes {
        let mode = mode.expect("can't get mode");
        let info = mode.info();
        let resolution = info.resolution();
        write!(stdout, "\n{}x{} y/n?", resolution.0, resolution.1).expect("output failed");
        'wait: loop {
            if let Some(option) = get_option(stdin) {
                if option {
                    mode_changed = Some(mode);
                    break 'select;
                } else {
                    break 'wait;
                }
            }
        }
    }
    write!(stdout, "\n").expect("output failed");
    if let Some(mode) = mode_changed {
        gop.set_mode(&mode)
            .expect_success("Failed to set graphics mode");
    }
}

#[cfg(not(debug_assertions))]
fn get_option(input: &mut Input) -> Option<bool> {
    if let Some(Key::Printable(c)) = input.read_key().expect_success("can't read key") {
        let c: char = c.into();
        match c {
            'y' | 'Y' => Some(true),
            'n' | 'N' => Some(false),
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(debug_assertions)]
fn set_mode(gop: &mut GraphicsOutput, st: &SystemTable<Boot>) {
    let stdout = st.stdout();
    stdout.clear().expect_success("Clear stdout failed");
    let mode = gop
        .modes()
        .map(|mode| mode.expect("Warnings encountered while querying mode"))
        .find(|ref mode| mode.info().resolution() == (1024, 768))
        .unwrap();
    gop.set_mode(&mode)
        .expect_success("Failed to set graphics mode");
}
