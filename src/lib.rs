#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use log::info;

pub mod gdt;
pub mod interrupts;
pub mod io;
pub mod keyboard;
pub mod logging;
pub mod qemu;
pub mod testing;

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    testing::test_panic_handler(info)
}

// Entry point for running unit tests
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    initialize();
    test_main();
    halt();
}

// Main OS initialization procedure: set up IDT, GDT, interrupt controller, etc.
pub fn initialize() {
    logging::initialize();
    info!("Initializing OS...");
    info!("  - interrupt descriptor table");
    interrupts::initialize_interrupt_descriptor_table();
    info!("  - global descriptor table");
    gdt::initialize_global_descriptor_table();
    info!("  - interrupt controller");
    interrupts::initialize_interrupt_controller();
    info!("Initialization complete.");
}

pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
