//! Handler functions for CPU interrupts.

use log::error;
use x86_64::{
    registers::control::Cr2,
    structures::idt::{InterruptStackFrame, PageFaultErrorCode},
};

use crate::{
    interrupt::{InterruptIndex, PICS},
    task::scancode_queue::ScancodeQueue,
};

/// Breakpoint exception handler. Currently, this just logs the exception and continues.
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: breakpoint\n{:#?}", stack_frame);
}

/// Page fault handler. I haven't implemented advanced page management yet (e.g. swapping),
/// so currently this causes the OS to halt.
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
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
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    error!("EXCEPTION: double fault\n{:#?}", stack_frame);
    crate::halt();
}

/// Timer handler. This doesn't do anything except signal the end of the interrupt.
pub extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

/// Keyboard handler. This adds the scancode of any key pressed onto the global scancode
/// queue, which is later read by an asynchronous task.
pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    // TODO: Make sure timeout doesn't cause spurious panics
    ScancodeQueue::add_scancode(unsafe { ps2::Controller::new().read_data().unwrap() });

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

pub extern "x86-interrupt" fn mouse_handler(_stack_frame: InterruptStackFrame) {
    let mut controller = unsafe { ps2::Controller::new() };
    // TODO: Two interrupts seem to be triggered on each event, but the second one doesn't have
    //       a data packet available to read. What's going on?
    if let Ok(packet) = controller.mouse().read_data_packet() {
        log::debug!("Mouse event: {:?}", packet);
    } else {
        // TODO: Handle events missing packets
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Mouse as u8);
    }
}
