#![no_std]
#![no_main]
#![feature(decl_macro)]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
extern crate log;

use os::kernel;
use uefi::prelude::*;

#[no_mangle]
pub extern "C" fn efi_main(image: uefi::Handle, st: SystemTable<Boot>) -> ! {
    let (st, map) = kernel::init(image, st);
    let _rt = unsafe { st.runtime_services() };
    #[cfg(test)]
    {
        test_main();
        loop {}
    }
    #[cfg(not(test))]
    {
        mem_map(map);
        os::kmain::kmain()
    }
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

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    os::test_panic_handler(info)
}

#[test_case]
fn test_print() {
    info!("It works");
    assert_eq!(1 + 1, 2);
}
