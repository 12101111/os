#![no_std]
#![no_main]
#![feature(decl_macro)]
#![feature(panic_info_message)]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
extern crate log;

use os::drivers::{console::*, uefi_init};
use os::{exit_qemu,QemuExitCode};
use uefi::prelude::*;
use core::fmt::{self,Write};
use core::panic::PanicInfo;

const MESSAGE: &str = "Example panic message from panic_handler test";
const PANIC_LINE: u32 = 31; // adjust this when moving the `panic!` call

#[no_mangle]
pub extern "C" fn efi_main(image: uefi::Handle, st: SystemTable<Boot>) -> ! {
    let (_st, _map) = uefi_init(image, st);
    info!("Test panic_handler");
    test_main();
    loop{}
}

#[test_case]
pub fn test_panic_handler(){
    panic!(MESSAGE);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    check_location(info);
    check_message(info);
    info!("PASS");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

fn fail(error: &str) -> ! {
    error!("{}", error);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn check_location(info: &PanicInfo) {
    let location = info.location().unwrap_or_else(|| fail("no location"));
    if location.file() != file!() {
        fail("file name wrong");
    }
    if location.line() != PANIC_LINE {
        fail("file line wrong");
    }
}

fn check_message(info: &PanicInfo) {
    let message = info.message().unwrap_or_else(|| fail("no message"));
    let mut compare_message = CompareMessage { expected: MESSAGE };
    write!(&mut compare_message, "{}", message).unwrap_or_else(|_| fail("write failed"));
    if compare_message.expected.len() != 0 {
        fail("message shorter than expected message");
    }
}

/// Compares a `fmt::Arguments` instance with the `MESSAGE` string
///
/// To use this type, write the `fmt::Arguments` instance to it using the
/// `write` macro. If the message component matches `MESSAGE`, the `expected`
/// field is the empty string.
struct CompareMessage {
    expected: &'static str,
}

impl fmt::Write for CompareMessage {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.expected.starts_with(s) {
            self.expected = &self.expected[s.len()..];
        } else {
            fail("message not equal to expected message");
        }
        Ok(())
    }
}
