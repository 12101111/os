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
    let fb = console::uefifb::get_fb(&st);
    let font = console::uefifb::get_font(&st);
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
    let map = mm::init(map);
    console::uefifb::init(fb, font);
    for mem in map {
        info!(
            "Page: 0x{:06X} -- 0x{:06X} Type:{:?}",
            mem.phys_start >> 12,
            (mem.phys_start >> 12) + mem.page_count,
            mem.ty
        );
    }
    use ::acpi::PlatformInfo;
    let acpi = acpi::get_acpi(&st);
    let plat = PlatformInfo::new(&acpi).unwrap();
    let proc_info = plat.processor_info.unwrap();
    info!("BSP: {:?}", proc_info.boot_processor);
    for ap in proc_info.application_processors {
        info!("AP: {:?}", ap)
    }
    info!("Available memory: {} Bytes", mm::size());
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
