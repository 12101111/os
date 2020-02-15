#![no_std]
#![no_main]
#![feature(decl_macro)]
#![feature(abi_efiapi)]

#[macro_use]
extern crate log;

use uefi::prelude::*;
extern crate alloc;

#[entry]
fn efi_main(image: uefi::Handle, st: SystemTable<Boot>) -> Status {
    os::init(image, st);
    info!("init done!");
    panic!("Done!")
}
