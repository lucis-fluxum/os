use crate::gdt;
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::InterruptDescriptorTable;

mod handlers;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut table = InterruptDescriptorTable::new();
        table
            .breakpoint
            .set_handler_fn(handlers::breakpoint_handler);
        unsafe {
            table
                .double_fault
                .set_handler_fn(handlers::double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        table[InterruptIndex::Timer as usize].set_handler_fn(handlers::timer_handler);
        table
    };
}

lazy_static! {
    static ref PICS: Mutex<ChainedPics> =
        Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
}

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
    use crate::{serial_print, serial_println};

    #[test_case]
    fn test_breakpoint_exception() {
        serial_print!("test_breakpoint_exception... ");
        x86_64::instructions::interrupts::int3();
        serial_println!("[ok]");
    }
}
