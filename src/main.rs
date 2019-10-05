#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod io;

#[panic_handler]
fn panic(_inf: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;
    writeln!(io::WRITER.lock(), "Here is some text.").unwrap();
    loop {}
}
