use alloc::alloc::{GlobalAlloc, Layout};
use core::{
    mem,
    ptr::{self, NonNull},
};

use super::Mutex;

/// The block sizes to use.
///
/// The sizes must each be power of 2 because they are also used as
/// the block alignment (alignments must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

/// Choose an appropriate block size for the given layout.
///
/// Returns an index into the `BLOCK_SIZES` array.
fn block_size_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES
        .iter()
        .position(|&size| size >= required_block_size)
}

struct Block {
    next: Option<&'static mut Block>,
}

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut Block>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl FixedSizeBlockAllocator {
    /// Creates an empty FixedSizeBlockAllocator.
    pub const fn new() -> Self {
        Self {
            list_heads: [None; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// # Safety
    /// This method is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the heap is unused. This method must be
    /// called only once.
    pub unsafe fn initialize(&mut self, heap_start: usize, heap_size: usize) {
        unsafe {
            self.fallback_allocator.init(heap_start, heap_size);
        }
    }

    /// Allocates using the fallback allocator.
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

unsafe impl GlobalAlloc for Mutex<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match block_size_index(&layout) {
            Some(index) => {
                // Grab the head of the block list for the given size
                match allocator.list_heads[index].take() {
                    Some(node) => {
                        // Pop first block off the front of the list
                        allocator.list_heads[index] = node.next.take();
                        node as *mut Block as *mut u8
                    }
                    None => {
                        // No block exists in list, so manually allocate a new block
                        let block_size = BLOCK_SIZES[index];
                        // This only works if all block sizes are a power of 2
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match block_size_index(&layout) {
            Some(index) => {
                // Whether or not this region was allocated with the fallback allocator, we
                // are now going to add the region to the list of free blocks, since it matches
                // one of our block sizes.

                // Create a new block to replace the head of the block list for the given size
                let new_block = Block {
                    next: allocator.list_heads[index].take(),
                };

                // Verify that block size and alignment is good for storing a pointer
                assert!(mem::size_of::<Block>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<Block>() <= BLOCK_SIZES[index]);

                #[allow(clippy::cast_ptr_alignment)]
                let new_block_ptr = ptr as *mut Block;
                unsafe {
                    // Write the new block info to the freed memory region
                    new_block_ptr.write(new_block);

                    // Set the new block to be the head of the list
                    allocator.list_heads[index] = Some(&mut *new_block_ptr);
                }
            }
            None => {
                // Region is too large, so it must have been allocated by the fallback
                // allocator. We'll use the fallback allocator to deallocate it.
                let ptr = NonNull::new(ptr).unwrap();
                unsafe {
                    allocator.fallback_allocator.deallocate(ptr, layout);
                }
            }
        }
    }
}
