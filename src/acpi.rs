use acpi::AcpiHandler;
use acpi::AcpiTables;
use core::ptr::NonNull;
use uefi::table::{Runtime, SystemTable};

pub fn get_acpi(st: &SystemTable<Runtime>) -> acpi::AcpiTables<Ident> {
    let cfg = st.config_table();
    let rsdp = cfg
        .into_iter()
        .find(|cfg| cfg.guid == uefi::table::cfg::ACPI2_GUID)
        .expect("Can't find ACPI Table")
        .address as usize;
    unsafe { AcpiTables::from_rsdp(Ident, rsdp).unwrap() }
}

#[derive(Copy, Clone)]
pub struct Ident;

impl AcpiHandler for Ident {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Ident, T> {
        acpi::PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new(physical_address as *mut T).unwrap(),
            region_length: size,
            mapped_length: size,
            handler: Ident,
        }
    }
    fn unmap_physical_region<T>(&self, _region: &acpi::PhysicalMapping<Ident, T>) {}
}
