//! Handler functions for CPU interrupts.

use log::error;
use x86_64::{
    registers::control::Cr2,
    structures::idt::{InterruptStackFrame, PageFaultErrorCode},
};

use crate::{
    interrupt::{InterruptIndex, PICS},
    keyboard,
    task::scancode_queue::ScancodeQueue,
};

/// Breakpoint exception handler. Currently, this just logs the exception and continues.
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    error!("EXCEPTION: breakpoint\n{:#?}", stack_frame);
}

/// Page fault handler. I haven't implemented advanced page management yet (e.g. swapping),
/// so currently this causes the OS to halt.
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    error!(
        "EXCEPTION: page fault\naccessed address: {:?}\nerror code: {:?}\n{:#?}",
        Cr2::read(),
        error_code,
        stack_frame
    );
    crate::halt();
}

/// Double fault handler. Currently, this just logs the exception and halts.
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    error!("EXCEPTION: double fault\n{:#?}", stack_frame);
    crate::halt();
}

/// Timer handler. This doesn't do anything except signal the end of the interrupt.
pub extern "x86-interrupt" fn timer_handler(_stack_frame: &mut InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

/// Keyboard handler. This adds the scancode of any key pressed onto the global scancode
/// queue, which is later read by an asynchronous task.
pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: &mut InterruptStackFrame) {
    ScancodeQueue::add_scancode(keyboard::read_scancode());

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

pub extern "x86-interrupt" fn mouse_handler(_stack_frame: &mut InterruptStackFrame) {
    let mut controller = unsafe { ps2::Controller::new() };
    // TODO: Two interrupts seem to be triggered on each event, but the second one doesn't have
    //       a data packet available to read. What's going on?
    match controller.mouse().read_data_packet() {
        Ok(packet) => log::debug!("Mouse event: {:?}", packet),
        // Ignore events missing packets for now
        Err(_) => {}
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Mouse as u8);
    }
}
