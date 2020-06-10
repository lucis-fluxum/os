use alloc::alloc::Layout;

use bootloader::BootInfo;
use x86_64::VirtAddr;

mod bump_allocator;
pub mod frame_allocator;
pub mod heap;

use bump_allocator::BumpAllocator;
use frame_allocator::BootInfoFrameAllocator;
use heap::{HEAP_SIZE, HEAP_START};

#[global_allocator]
pub(crate) static HEAP_ALLOCATOR: Mutex<BumpAllocator> = Mutex::new(BumpAllocator::new());

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    // TODO: I don't like panicking here, maybe think of something else
    panic!("allocation error: {:?}", layout);
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

/// Align the given address `addr` upwards to nearest `alignment`.
///
/// Requires that `alignment` is a power of two.
fn align_up(addr: usize, alignment: usize) -> usize {
    debug_assert!(alignment.count_ones() == 1);
    // Round addr + alignment - 1 down to the nearest multiple of alignment
    (addr + alignment - 1) & !(alignment - 1)
}

/// A wrapper around spinning_top::Spinlock to permit trait implementations.
pub(crate) struct Mutex<A> {
    inner: spinning_top::Spinlock<A>,
}

impl<A> Mutex<A> {
    pub(crate) const fn new(inner: A) -> Self {
        Self {
            inner: spinning_top::Spinlock::new(inner),
        }
    }

    pub(crate) fn lock(&self) -> spinning_top::SpinlockGuard<A> {
        self.inner.lock()
    }
}
