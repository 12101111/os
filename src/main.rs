#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(decl_macro)]

pub mod drivers;
pub mod kmain;
#[macro_use]
extern crate log;

use drivers::console::{fbterm::init_fbterm, init_logger, uart::init_uart};
use drivers::uefifb::init_fb;
use kmain::kmain;
use uefi::{
    prelude::*,
    table::boot::{MemoryDescriptor, MemoryType},
};

#[no_mangle]
pub extern "C" fn efi_main(image: uefi::Handle, st: SystemTable<Boot>) -> ! {
    let fb = init_fb(&st);
    exit_boot_services(st, image);
    init_logger();
    init_fbterm(fb);
    init_uart();
    kmain()
}

fn exit_boot_services(st: SystemTable<Boot>, image: uefi::Handle) {
    let bt = st.boot_services();
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
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    x86_64::instructions::hlt();
    loop {}
}
