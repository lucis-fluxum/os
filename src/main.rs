#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod io;
mod qemu;

#[cfg(test)]
mod test;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic_test(info: &PanicInfo) -> ! {
    serial_println!("[FAIL]\n");
    serial_println!("Error: {}\n", info);
    qemu::exit(qemu::ExitCode::Failed);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)] test_main();
    loop {}
}
