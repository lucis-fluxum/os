#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::testing::test_panic_handler(info)
}

fn main(boot_info: &'static BootInfo) -> ! {
    os::memory::initialize_heap_allocator(boot_info);
    test_main();
    os::halt();
}

entry_point!(main);

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn fill_entire_heap() {
    for i in 0..os::memory::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn long_lived_after_filled_heap() {
    let long_lived = Box::new(1);
    for i in 0..os::memory::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}
