use alloc::alloc::Layout;

use bootloader::BootInfo;
use linked_list_allocator::LockedHeap;
use x86_64::VirtAddr;

pub mod frame_allocator;
pub mod heap;

use frame_allocator::BootInfoFrameAllocator;
use heap::{HEAP_SIZE, HEAP_START};

#[global_allocator]
pub(crate) static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    log::error!("allocation error: {:?}", layout);
    crate::halt();
}

pub fn initialize_heap_allocator(boot_info: &'static BootInfo) {
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { frame_allocator::initialize_mapper(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(&boot_info.memory_map) };
    heap::initialize(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
}

