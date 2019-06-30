pub mod acpi;
pub mod console;
pub mod uefifb;

use acpi::rsdp;
use console::{fbterm::init_fbterm, init_logger, uart::init_uart};
use uefifb::init_fb;
use uefi::{
    prelude::*,
    table::boot::{MemoryDescriptor, MemoryType},
};

pub fn init(image: uefi::Handle, st: SystemTable<Boot>){
    let fb = init_fb(&st);
    init_logger();
    init_fbterm(fb);
    init_uart();
    let rsdp = rsdp(&st);
    let buffer = alloc_mmap(&st);
    let (st, map) = st
        .exit_boot_services(image, &mut buffer[..])
        .expect_success("Failed to exit boot services");
    let _rt = unsafe { st.runtime_services() };
    for mem in map {
        trace!(
            "mem: 0x{:016X} Size(Page): {:8} Type: {:?}",
            mem.phys_start,
            mem.page_count,
            mem.ty
        );
    }
    trace!("RDSP: {:?}", rsdp);
}

fn alloc_mmap(st: &SystemTable<Boot>) -> &'static mut [u8] {
    let bt = st.boot_services();
    let mmap_size =
        st.boot_services().memory_map_size() + 8 * core::mem::size_of::<MemoryDescriptor>();
    let mmap_buffer = bt
        .allocate_pool(MemoryType::LOADER_DATA, mmap_size)
        .expect_success("alloc failed");
    unsafe { core::slice::from_raw_parts_mut(mmap_buffer, mmap_size) }
}