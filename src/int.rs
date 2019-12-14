use x86_64::structures::idt;

static mut IDT: idt::InterruptDescriptorTable = idt::InterruptDescriptorTable::new();

pub unsafe fn init() {
    IDT.page_fault.set_handler_fn(page_fault_handler);
    IDT.load();
}

extern "x86-interrupt" fn page_fault_handler(
    frame: &mut idt::InterruptStackFrame,
    error_code: idt::PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    error!("Page fault: {:?}", error_code);
    error!("Address: {:?}", Cr2::read());
    panic!("{:#?}", frame);
}
