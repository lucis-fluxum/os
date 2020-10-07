//! Interrupts, the programmable interrupt controller, and the interrupt descriptor table.

use conquer_once::spin::Lazy;
use pic8259_simple::ChainedPics;
use x86_64::structures::idt::InterruptDescriptorTable;

use crate::{gdt, sync::Mutex};

pub mod handlers;

/// User-defined interrupts start at index 32.
pub const PIC_1_OFFSET: u8 = 32;
/// The second PIC starts 8 positions away from the first.
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub(crate) enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Secondary,
    Serial2Or4,
    Serial1Or3,
    SoundOrParallel2And3,
    FloppyDisk,
    Parallel1,
    RealTimeClock,
    ACPI,
    Open1,
    Open2,
    Mouse,
    Coprocessor,
    PrimaryATA,
    SecondaryATA,
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut table = InterruptDescriptorTable::new();
    table
        .breakpoint
        .set_handler_fn(handlers::breakpoint_handler);
    table
        .page_fault
        .set_handler_fn(handlers::page_fault_handler);
    unsafe {
        table
            .double_fault
            .set_handler_fn(handlers::double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    table[InterruptIndex::Timer as usize].set_handler_fn(handlers::timer_handler);
    table[InterruptIndex::Keyboard as usize].set_handler_fn(handlers::keyboard_handler);
    table
});

static PICS: Lazy<Mutex<ChainedPics>> =
    Lazy::new(|| Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }));

pub fn initialize_interrupt_descriptor_table() {
    IDT.load();
}

pub fn initialize_interrupt_controller() {
    unsafe {
        PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn test_breakpoint_exception() {
        x86_64::instructions::interrupts::int3();
    }
}
