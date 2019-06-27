#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(decl_macro)]

pub mod drivers;
pub mod kmain;
#[macro_use]
extern crate log;

use drivers::acpi::rsdp;
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
    init_logger();
    init_fbterm(fb);
    init_uart();
    let rsdp = rsdp(&st);
    let buffer = alloc_mmap(&st);
    let (st, map) = st
        .exit_boot_services(image, &mut buffer[..])
        .expect_success("Failed to exit boot services");
    let _rt = unsafe { st.runtime_services() };
    for mem in map {
        trace!(
            "mem: 0x{:016X} Size(Page): {:8} Type: {:?}",
            mem.phys_start,
            mem.page_count,
            mem.ty
        );
    }
    trace!("RDSP: {:?}", rsdp);
    kmain()
}

fn alloc_mmap(st: &SystemTable<Boot>) -> &'static mut [u8] {
    let bt = st.boot_services();
    let mmap_size =
        st.boot_services().memory_map_size() + 8 * core::mem::size_of::<MemoryDescriptor>();
    let mmap_buffer = bt
        .allocate_pool(MemoryType::LOADER_DATA, mmap_size)
        .expect_success("alloc failed");
    unsafe { core::slice::from_raw_parts_mut(mmap_buffer, mmap_size) }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    x86_64::instructions::hlt();
    loop {}
}
