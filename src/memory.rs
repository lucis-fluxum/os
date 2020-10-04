use alloc::alloc::Layout;

use bootloader::BootInfo;
use x86_64::VirtAddr;

mod bump_allocator;
mod fixed_size_block_allocator;
mod frame_allocator;
mod heap;
mod linked_list_allocator;

pub use self::linked_list_allocator::LinkedListAllocator;
pub use bump_allocator::BumpAllocator;
pub use fixed_size_block_allocator::FixedSizeBlockAllocator;
pub use frame_allocator::BootInfoFrameAllocator;
pub use heap::{HEAP_SIZE, HEAP_START};

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

/// A wrapper around `spinning_top::Spinlock` to permit trait implementations.
pub struct Mutex<T>(spinning_top::Spinlock<T>);

impl<T> Mutex<T> {
    pub const fn new(inner: T) -> Self {
        Self(spinning_top::Spinlock::new(inner))
    }

    pub fn lock(&self) -> spinning_top::SpinlockGuard<T> {
        self.0.lock()
    }
}
