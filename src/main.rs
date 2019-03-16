#![no_std]
#![no_main]
use fbterm::*;
use spin::Mutex;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::table::boot::{MemoryDescriptor, MemoryType};

static CONSOLE: Mutex<Option<Fbterm<XRGB8888>>> = Mutex::new(None);

const TEXT: &str = "Hello,World !\nBoot from UEFI !!\nWritten in Rust !!!";

#[no_mangle]
pub extern "C" fn efi_main(image: uefi::Handle, st: SystemTable<Boot>) -> ! {
    let bt = st.boot_services();
    let gop = bt
        .locate_protocol::<GraphicsOutput>()
        .expect_success("UEFI GOP not support");
    let gop = unsafe { &mut *gop.get() };
    let (width, height) = set_graphics_mode(gop);
    let mut fb = gop.frame_buffer();
    st.stdout().reset(false).unwrap_success();
    let max_mmap_size =
        st.boot_services().memory_map_size() + 8 * core::mem::size_of::<MemoryDescriptor>();
    let mmap_buffer = bt
        .allocate_pool(MemoryType::LOADER_DATA, max_mmap_size)
        .expect_success("alloc failed");
    let mmap_storage = unsafe { core::slice::from_raw_parts_mut(mmap_buffer, max_mmap_size) };
    let (st, _iter) = st
        .exit_boot_services(image, &mut mmap_storage[..])
        .expect_success("Failed to exit boot services");
    let rt = unsafe { st.runtime_services() };
    init_fb(fb.as_mut_ptr(), width, height);
    println!("{}", TEXT);
    let time = rt.get_time().expect_success("Failed to get time");
    println!("Time: {}-{}-{} {}:{}:{}",time.year(),time.month(),time.day(),time.hour(),time.minute(),time.second());
    panic!("Good bye!");
}

fn init_fb(fb: *mut u8, width: usize, height: usize) {
    let background = XRGB8888::new(0, 0, 0, 0xA8);
    let foreground = XRGB8888::new(255, 0xA8, 0xA8, 0xA8);
    let fb = unsafe { Framebuffer::new(fb, width, height, background, foreground) };
    let mut fbterm = Fbterm::new(fb,fbterm::Fonts::VGA8x14);
    fbterm.clear();
    let mut console = CONSOLE.lock();
    if console.is_none() {
        *console = Some(fbterm)
    }
}

#[cfg(not(debug_assertions))]
fn set_graphics_mode(gop: &mut GraphicsOutput) -> (usize, usize) {
    let mode = gop
        .modes()
        .map(|mode| mode.expect("Warnings encountered while querying mode"))
        .max_by(|m1, m2| {
            let r1 = m1.info().resolution();
            let r2 = m2.info().resolution();
            (r1.0 * r1.1).cmp(&(r2.0 * r2.1))
        })
        .unwrap();
    gop.set_mode(&mode)
        .expect_success("Failed to set graphics mode");
    mode.info().resolution()
}

#[cfg(debug_assertions)]
fn set_graphics_mode(gop: &mut GraphicsOutput) -> (usize, usize) {
    let mode = gop
        .modes()
        .map(|mode| mode.expect("Warnings encountered while querying mode"))
        .find(|ref mode| mode.info().resolution() == (640, 480))
        .unwrap();
    gop.set_mode(&mode)
        .expect_success("Failed to set graphics mode");
    mode.info().resolution()
}

pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    CONSOLE.lock().as_mut().unwrap().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
