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
    info!("啥也不能干的操作系统, powered by Rust with ♥");
    info!("Rust 非常适合编写新一代操作系统, 可以方便的利用no_std生态,例如TrueType™字体支持");
    info!("再见,我要panic了");
    panic!("Done!")
}
