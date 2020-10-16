use super::CONSOLE;
use core::ptr::NonNull;
use fbterm::*;
use uefi::{
    prelude::*,
    proto::console::gop::GraphicsOutput,
    proto::media::{
        file::{File, FileAttribute, FileInfo, FileMode, FileType},
        fs::SimpleFileSystem,
    },
    table::boot,
};

pub fn get_fb(st: &SystemTable<Boot>) -> Framebuffer<'static, RGBA8888> {
    let bt = st.boot_services();
    let gop = bt
        .locate_protocol::<GraphicsOutput>()
        .expect_success("UEFI GOP not support");
    let gop = unsafe { &mut *gop.get() };
    set_mode(gop, st);
    let fb = gop.frame_buffer().as_mut_ptr();
    trace!("uefifb address: {:?}", fb);
    let info = gop.current_mode_info();
    let background = RGBA8888::new(0, 0, 0, 0xA8);
    let foreground = RGBA8888::new(255, 0xA8, 0xA8, 0xA8);
    let (w, h) = info.resolution();
    let fb = NonNull::new(fb).unwrap();
    unsafe { Framebuffer::new(fb, w, h, info.stride(), background, foreground) }
}

pub fn get_font(st: &SystemTable<Boot>) -> &'static [u8] {
    let bt = st.boot_services();
    let fs = bt
        .locate_protocol::<SimpleFileSystem>()
        .expect_success("UEFI SimpleFileSystem not support");
    let fs = unsafe { &mut *fs.get() };
    let mut root = fs.open_volume().expect_success("Open volume failed");
    let font_file = root
        .open("font.ttf", FileMode::Read, FileAttribute::VALID_ATTR)
        .expect_success("Failed to load font.ttf")
        .into_type()
        .expect_success("file.into_type");
    let mut font_file = match font_file {
        FileType::Dir(_) => {
            panic!("Expect font.ttf is a regular file, not a diectory!");
        }
        FileType::Regular(file) => file,
    };
    info!("open font.ttf");
    let file_size = {
        let mut len = 128;
        let mut buffer_ptr = bt
            .allocate_pool(boot::MemoryType::LOADER_DATA, len)
            .expect_success("alloc failed");
        let mut buffer = unsafe { core::slice::from_raw_parts_mut(buffer_ptr, len) };
        let file_info = loop {
            match font_file.get_info::<FileInfo>(&mut buffer) {
                Ok(r) => break r.unwrap(),
                Err(e) => {
                    len = e.data().unwrap();
                    bt.free_pool(buffer_ptr)
                        .expect_success("Failed to free memory");
                    buffer_ptr = bt
                        .allocate_pool(boot::MemoryType::LOADER_DATA, len)
                        .expect_success("alloc failed");
                    buffer = unsafe { core::slice::from_raw_parts_mut(buffer_ptr, len) };
                    continue;
                }
            }
        };
        let file_size = file_info.file_size();
        drop(file_info);
        drop(buffer);
        bt.free_pool(buffer_ptr)
            .expect_success("Failed to free memory");
        file_size as usize
    };
    info!("font.ttf size: {} bytes", file_size);
    let font_ptr = bt
        .allocate_pool(boot::MemoryType::LOADER_DATA, file_size)
        .expect_success("alloc failed");
    let font = unsafe { core::slice::from_raw_parts_mut(font_ptr, file_size) };
    font_file
        .read(font)
        .expect_success("Failed to read font.ttf");
    info!("read font.ttf to memory");
    font as &[u8]
}

pub fn init(fb: Framebuffer<'static, RGBA8888>, font: &[u8]) {
    let mut console = CONSOLE.lock();
    let font = TrueTypeFont::new(font, 20.0);
    let mut newterm = Fbterm::new(fb, font);
    let double_buffer = alloc::vec![0u8; newterm.framebuffer.buffer_size()];
    // FIXME: don't use leak
    let addr = NonNull::new(double_buffer.leak().as_mut_ptr()).unwrap();
    unsafe { newterm.framebuffer.set_double_buffer(addr) };
    console.fbterm = Some(newterm);
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
    writeln!(stdout, "").expect("output failed");
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
