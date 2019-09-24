#![no_std]
#![no_main]
#![feature(decl_macro)]

#[macro_use]
extern crate log;

use os::kernel;
use uefi::prelude::*;

#[entry]
fn efi_main(image: uefi::Handle, st: SystemTable<Boot>) -> Status {
    let (st, map) = kernel::init(image, st);
    let _rt = unsafe { st.runtime_services() };
    mem_map(map);
    os::kmain::kmain()
}

fn mem_map(map: uefi::table::boot::MemoryMapIter) {
    for mem in map {
        trace!(
            "mem: 0x{:016X} Size(Page): {:8} Type:{:?}",
            mem.phys_start,
            mem.page_count,
            mem.ty
        );
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    os::hlt_loop();
}
