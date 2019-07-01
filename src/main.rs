#![no_std]
#![no_main]
#![feature(decl_macro)]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
extern crate log;

use os::drivers::{acpi::rsdp, uefi_init};
use os::kmain::kmain;
use uefi::prelude::*;

#[no_mangle]
pub extern "C" fn efi_main(image: uefi::Handle, st: SystemTable<Boot>) -> ! {
    let (st, _map) = uefi_init(image, st);
    let _rsdp = rsdp(&st); //0x7bfa014
    let _rt = unsafe { st.runtime_services() };
    #[cfg(test)]
    test_main();
    #[cfg(not(test))]
    kmain();
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    x86_64::instructions::hlt();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    os::test_panic_handler(info)
}

#[test_case]
fn test_print() {
    info!("It works");
    assert_eq!(1+1,2);
}