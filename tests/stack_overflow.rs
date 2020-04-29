#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use os::{qemu, serial_print, serial_println};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::testing::test_panic_handler(info)
}

// Set up a custom interrupt descriptor table with a handler function that
// prints [ok] and exits qemu.
lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut table = InterruptDescriptorTable::new();
        unsafe {
            table
                .double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(os::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        table
    };
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    qemu::exit(qemu::ExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow... ");

    os::gdt::initialize_global_descriptor_table();
    TEST_IDT.load();

    #[allow(unconditional_recursion)]
    fn stack_overflow() {
        stack_overflow();
    }
    stack_overflow();

    panic!("Continued running after stack overflow!");
}

