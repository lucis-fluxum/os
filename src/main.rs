#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};
use x86_64::VirtAddr;

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

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { os::memory::translate_addr(virt, phys_mem_offset) };
        log::debug!("{:?} -> {:?}", virt, phys);
    }

    #[cfg(test)]
    test_main();

    os::halt();
}

entry_point!(main);
