#![no_std]
#![no_main]
#![feature(decl_macro)]

use os::kmain::kmain;
use os::drivers::init;
use uefi::prelude::*;

#[no_mangle]
pub extern "C" fn efi_main(image: uefi::Handle, st: SystemTable<Boot>) -> ! {
    init(image,st);
    kmain()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    x86_64::instructions::hlt();
    loop {}
}
