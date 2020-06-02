#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod gdt;
pub mod interrupts;
pub mod io;
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
    loop {}
}

// Main OS initialization procedure: set up IDT, GDT, interrupt controller, etc.
pub fn initialize() {
    interrupts::initialize_interrupt_descriptor_table();
    gdt::initialize_global_descriptor_table();
    interrupts::initialize_interrupt_controller();
}
