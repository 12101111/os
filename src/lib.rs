#![no_std]
#![cfg_attr(test, no_main)]
#![feature(decl_macro)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
extern crate log;

pub mod kernel;
pub mod kmain;

pub fn test_runner(tests: &[&dyn Fn()]) {
    trace!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
use uefi::prelude::*;

#[cfg(test)]
#[no_mangle]
pub extern "C" fn efi_main(image: uefi::Handle, st: SystemTable<Boot>) -> ! {
    let (_st, _map) = drivers::uefi_init(image, st);
    info!("Start to run test");
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
}

#[test_case]
fn test_print() {
    info!("It works");
    assert_eq!(1 + 1, 2);
}
