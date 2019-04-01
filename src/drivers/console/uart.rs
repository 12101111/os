use super::CONSOLE;
use uart_16550::*;
pub fn init_uart() {
    if CONSOLE.lock().uart.is_some() {
        return;
    }
    for i in 1..=4 {
        let mut uart = unsafe { UART::new(i) };
        if uart.is_some() {
            info!("Found UART: COM {}", i);
            uart.as_mut().unwrap().init_with_default();
            CONSOLE.lock().uart = uart;
        }
    }
}
