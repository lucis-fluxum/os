#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use os::println;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic_test(info: &PanicInfo) -> ! {
    os::test::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)] test_main();
    loop {}
}
