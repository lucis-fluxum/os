use alloc::alloc::Layout;

use bootloader::BootInfo;
use x86_64::VirtAddr;

use crate::sync::Mutex;

pub use self::{
    bump_allocator::BumpAllocator,
    fixed_size_block_allocator::FixedSizeBlockAllocator,
    frame_allocator::BootInfoFrameAllocator,
    heap::{HEAP_SIZE, HEAP_START},
    linked_list_allocator::LinkedListAllocator,
};

mod bump_allocator;
mod fixed_size_block_allocator;
mod frame_allocator;
mod heap;
mod linked_list_allocator;

#[global_allocator]
static HEAP_ALLOCATOR: Mutex<FixedSizeBlockAllocator> = Mutex::new(FixedSizeBlockAllocator::new());

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}

pub fn initialize_heap_allocator(boot_info: &'static BootInfo) {
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { frame_allocator::initialize_mapper(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(&boot_info.memory_map) };
    heap::initialize(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    unsafe {
        HEAP_ALLOCATOR.lock().initialize(HEAP_START, HEAP_SIZE);
    }
}

/// Align the given address `addr` upwards to nearest `alignment`.
///
/// Requires that `alignment` is a power of two.
fn align_up(addr: usize, alignment: usize) -> usize {
    assert!(alignment.count_ones() == 1);
    // Round addr + alignment - 1 down to the nearest multiple of alignment
    (addr + alignment - 1) & !(alignment - 1)
}
