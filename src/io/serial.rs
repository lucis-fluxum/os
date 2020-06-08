use core::fmt;

use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use uart_16550::SerialPort;

static SERIAL1: Lazy<Spinlock<SerialPort>> = Lazy::new(|| {
    let mut serial_port = unsafe { SerialPort::new(0x3f8) };
    serial_port.init();
    Spinlock::new(serial_port)
});

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Couldn't print to serial port.");
}
