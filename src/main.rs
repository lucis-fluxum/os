#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    os::halt();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::testing::test_panic_handler(info)
}

// Entry point for starting the OS, or running main tests
fn main(boot_info: &'static BootInfo) -> ! {
    os::initialize();

    let result = boot_info.memory_map.iter().map(|region| {
        (
            region.range.start_addr(),
            region.range.end_addr(),
            (region.range.end_addr() - region.range.start_addr()) / 4096,
            region.region_type,
        )
    });
    for item in result {
        log::debug!("{:?}", item);
    }

    #[cfg(test)]
    test_main();

    os::halt();
}

entry_point!(main);
