//! Global descriptor table initialization.

use conquer_once::spin::Lazy;
use x86_64::{
    instructions::{segmentation::set_cs, tables::load_tss},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

/// The first stack in the interrupt stack table is meant for the double fault handler.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 20 * 1024;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
        stack_start + STACK_SIZE
    };
    tss
});

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
    (
        gdt,
        Selectors {
            code_selector,
            tss_selector,
        },
    )
});

/// Set up the global descriptor table with a kernel code segment and a task state segment
/// that contains known good stacks to use in case of an interrupt.
pub fn initialize_global_descriptor_table() {
    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
