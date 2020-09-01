use acpi::handler;
use core::ptr::NonNull;
use uefi::table::{Runtime, SystemTable};

pub fn get_acpi(st: &SystemTable<Runtime>) -> acpi::Acpi {
    let cfg = st.config_table();
    let rsdp = cfg
        .into_iter()
        .find(|cfg| cfg.guid == uefi::table::cfg::ACPI2_GUID)
        .expect("Can't find ACPI Table")
        .address as usize;
    unsafe { acpi::parse_rsdp(&mut Ident, rsdp).unwrap() }
}

struct Ident;

impl handler::AcpiHandler for Ident {
    unsafe fn map_physical_region<T>(
        &mut self,
        physical_address: usize,
        size: usize,
    ) -> handler::PhysicalMapping<T> {
        handler::PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new(physical_address as *mut T).unwrap(),
            region_length: size,
            mapped_length: size,
        }
    }
    fn unmap_physical_region<T>(&mut self, _region: handler::PhysicalMapping<T>) {}
}
