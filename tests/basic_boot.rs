#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test::test_panic_handler(info)
}

// Entry point for this integration test
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}
