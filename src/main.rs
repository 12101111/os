#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(decl_macro)]

pub mod drivers;
pub mod kmain;

#[macro_use]
extern crate log;

use drivers::console::{fbterm::init_fb, init_logger, uart::init_uart};
use kmain::kmain;

use uefi::{
    prelude::*,
    proto::console::gop::GraphicsOutput,
    table::boot::{MemoryDescriptor, MemoryType},
};

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
    let _rt = unsafe { st.runtime_services() };
    init_logger();
    init_fb(fb.as_mut_ptr(), width, height);
    init_uart();
    kmain()
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

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    loop {}
}
