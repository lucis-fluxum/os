#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[cfg(test)]
use bootloader::entry_point;
use bootloader::BootInfo;
use log::info;
use x86_64::VirtAddr;

use memory::{
    frame_allocator::{self, BootInfoFrameAllocator},
    heap_allocator,
};

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
    interrupts::initialize_interrupt_descriptor_table();

    info!("  - global descriptor table");
    gdt::initialize_global_descriptor_table();

    info!("  - interrupt controller");
    interrupts::initialize_interrupt_controller();

    info!("  - heap allocator");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { frame_allocator::initialize_mapper(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(&boot_info.memory_map) };
    heap_allocator::initialize_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    info!("Initialization complete.");
}

pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
