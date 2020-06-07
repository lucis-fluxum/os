#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
use bootloader::{entry_point, BootInfo};
use log::info;

pub mod gdt;
pub mod interrupts;
#[doc(hidden)]
pub mod io;
pub(crate) mod keyboard;
pub(crate) mod logging;
pub mod memory;
pub mod qemu;
pub mod testing;

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    testing::test_panic_handler(info)
}

#[cfg(test)]
fn run_unit_tests(_boot_info: &'static BootInfo) -> ! {
    initialize();
    test_main();
    halt();
}

#[cfg(test)]
entry_point!(run_unit_tests);

// Main OS initialization procedure: set up IDT, GDT, interrupt controller, etc.
pub fn initialize() {
    // Set up logging using VGA text buffer
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
