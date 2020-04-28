#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os::{qemu, serial_print, serial_println};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    qemu::exit(qemu::ExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
        serial_println!("[test did not panic]");
        qemu::exit(qemu::ExitCode::Failed);
    }
    qemu::exit(qemu::ExitCode::Success);
}

#[test_case]
fn should_fail() {
    serial_print!("should_fail... ");
    assert_eq!(0, 1);
}
