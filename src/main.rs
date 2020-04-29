#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::testing::test_panic_handler(info)
}

// Entry point for starting the OS, or running main tests
#[no_mangle]
pub extern "C" fn _start() -> ! {
    os::initialize();

    #[cfg(test)]
    test_main();

    loop {}
}
