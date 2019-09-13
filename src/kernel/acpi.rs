use uefi::table::{Runtime, SystemTable};

pub fn rsdp(st: &SystemTable<Runtime>) -> *const core::ffi::c_void {
    let cfg = st.config_table();
    cfg.into_iter()
        .find(|cfg| cfg.guid == uefi::table::cfg::ACPI2_GUID)
        .expect("Can't find ACPI Table")
        .address
}