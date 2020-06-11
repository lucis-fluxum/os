#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(const_in_array_repeat_expressions)]
#![feature(unsafe_block_in_unsafe_fn)]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(wake_trait)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[cfg(test)]
use bootloader::entry_point;
use bootloader::BootInfo;
use log::info;

pub mod gdt;
pub mod interrupt;
#[doc(hidden)]
pub mod io;
pub(crate) mod keyboard;
pub(crate) mod logging;
pub mod memory;
pub mod qemu;
pub mod task;
pub mod testing;

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    testing::test_panic_handler(info)
}

#[cfg(test)]
fn run_unit_tests(boot_info: &'static BootInfo) -> ! {
    initialize(boot_info);
    test_main();
    halt();
}

#[cfg(test)]
entry_point!(run_unit_tests);

// Main OS initialization procedure: set up IDT, GDT, interrupt controller, etc.
pub fn initialize(boot_info: &'static BootInfo) {
    // Set up logging using VGA text buffer
    logging::initialize();

    info!("Initializing OS...");
    info!("  - interrupt descriptor table");
    interrupt::initialize_interrupt_descriptor_table();
    info!("  - global descriptor table");
    gdt::initialize_global_descriptor_table();
    info!("  - interrupt controller");
    interrupt::initialize_interrupt_controller();
    info!("  - heap allocator");
    memory::initialize_heap_allocator(boot_info);
    info!("Initialization complete.");
}

pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
