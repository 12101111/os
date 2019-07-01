pub mod acpi;
pub mod console;
pub mod uefifb;

use console::{fbterm::init_fbterm, init_logger, uart::init_uart};
use uefi::{
    prelude::*,
    table::boot::{MemoryDescriptor, MemoryMapIter, MemoryType},
    table::{Runtime, SystemTable},
};
use uefifb::init_fb;

pub fn uefi_init(
    image: uefi::Handle,
    st: SystemTable<Boot>,
) -> (SystemTable<Runtime>, MemoryMapIter<'static>) {
    let fb = init_fb(&st);
    init_logger();
    init_fbterm(fb);
    init_uart();
    let buffer = alloc_mmap(&st);
    st.exit_boot_services(image, &mut buffer[..])
        .expect_success("Failed to exit boot services")
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
