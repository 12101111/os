#![no_std]
#![feature(decl_macro)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

pub mod acpi;
pub mod console;
pub mod int;
pub mod mm;

use uefi::{
    prelude::*,
    table::{boot, SystemTable},
};

pub fn init(image: uefi::Handle, st: SystemTable<Boot>) {
    console::init(&st);
    let bt = st.boot_services();
    let mmap_size =
        st.boot_services().memory_map_size() + 8 * core::mem::size_of::<boot::MemoryDescriptor>();
    let mmap_buffer = bt
        .allocate_pool(boot::MemoryType::LOADER_DATA, mmap_size)
        .expect_success("alloc failed");
    let buffer = unsafe { core::slice::from_raw_parts_mut(mmap_buffer, mmap_size) };
    let (st, map) = st
        .exit_boot_services(image, &mut buffer[..])
        .expect_success("Failed to exit boot services");
    unsafe { int::init() }
    mm::init(map);
    let acpi = acpi::get_acpi(&st);
    info!("BSP: {:?}",acpi.boot_processor);
    for ap in acpi.application_processors{
        info!("AP: {:?}",ap)
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    hlt_loop()
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
