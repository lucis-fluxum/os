#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
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
pub mod keyboard;
pub mod logging;
pub mod memory;
pub mod qemu;
pub mod sync;
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
    logging::initialize_logging();

    info!("Initializing OS...");
    info!("  - interrupt descriptor table");
    interrupt::initialize_interrupt_descriptor_table();
    info!("  - global descriptor table");
    gdt::initialize_global_descriptor_table();
    info!("  - interrupt controller");
    interrupt::initialize_interrupt_controller();
    info!("  - heap allocator");
    memory::initialize_heap_allocator(boot_info);
    info!("  - PS/2 controller");
    keyboard::initialize_ps2_controller();
    info!("Initialization complete.");

    for device_info in tinypci::brute_force_scan() {
        info!(
            "Device {}: {:?} ({})",
            device_info.device,
            device_info.full_class,
            tinypci::name_for_vendor_id(device_info.vendor_id)
        );
    }
}

pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
