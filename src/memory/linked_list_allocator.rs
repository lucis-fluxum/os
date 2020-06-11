use super::{align_up, Mutex};
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};

struct FreeListNode {
    size: usize,
    next: Option<&'static mut FreeListNode>,
}

impl FreeListNode {
    const fn new(size: usize) -> Self {
        Self { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

/// A heap allocator that uses a linked list to keep track of freed memory regions.
///
/// This allocator eventually creates many small free regions without merging adjacent
/// free regions, which causes fragmentation. Using the `LockedHeap` type from the
/// `linked_list_allocator` crate is recommended instead.
pub struct LinkedListAllocator {
    head: FreeListNode,
}

impl LinkedListAllocator {
    /// Creates an empty LinkedListAllocator.
    pub const fn new() -> Self {
        Self {
            head: FreeListNode::new(0),
        }
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// This function is unsafe because the caller must guarantee that the given heap bounds
    /// are valid and that the heap is unused. This method must be called only once.
    pub unsafe fn initialize(&mut self, heap_start: usize, heap_size: usize) {
        self.free_region(heap_start, heap_size);
    }

    /// Adds the given memory region to the front of the list.
    unsafe fn free_region(&mut self, addr: usize, size: usize) {
        // ensure that the freed region is capable of holding ListNode
        assert_eq!(align_up(addr, mem::align_of::<FreeListNode>()), addr);
        assert!(size >= mem::size_of::<FreeListNode>());

        let mut node = FreeListNode::new(size);
        // break off list at item 1, attach it to new node
        node.next = self.head.next.take();
        // write new node to start of freed region
        let node_ptr = addr as *mut FreeListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr)
    }

    /// Looks for a free region with the given size and alignment and removes it from the list.
    ///
    /// Returns a `Some` with a tuple of the list node and the aligned start address of the
    /// allocation, or `None` if no suitable region was found.
    fn allocate_region(
        &mut self,
        size: usize,
        align: usize,
    ) -> Option<(&'static mut FreeListNode, usize)> {
        let mut current = &mut self.head;
        // look for a large enough memory region in linked list
        while let Some(ref mut region) = current.next {
            if Self::is_suitable_region(&region, size, align) {
                // get pointer to the FreeListNode after the suitable region
                let next_free_node = region.next.take();
                // get aligned address to start of new region
                let aligned_start_addr = align_up(region.start_addr(), align);
                // get pointer to suitable region's FreeListNode
                let suitable_region_details =
                    Some((current.next.take().unwrap(), aligned_start_addr));
                // skip over the now-removed node
                current.next = next_free_node;
                return suitable_region_details;
            } else {
                // try next region
                current = current.next.as_mut().unwrap();
            }
        }

        // no suitable region found
        None
    }

    /// Determines whether to use the given region for an allocation with a given size and
    /// alignment.
    fn is_suitable_region(region: &FreeListNode, size: usize, align: usize) -> bool {
        let start_addr = align_up(region.start_addr(), align);
        let end_addr = {
            match start_addr.checked_add(size) {
                Some(result) => result,
                None => return false,
            }
        };

        // Is the region big enough?
        if end_addr > region.end_addr() {
            return false;
        }

        // If there is leftover space, can we fit a FreeListNode in it, assuming we choose this
        // region?
        let excess_size = region.end_addr() - end_addr;
        if excess_size > 0 && excess_size < mem::size_of::<FreeListNode>() {
            return false;
        }

        true
    }

    /// Adjust the given layout so that the resulting allocated memory
    /// region is capable of storing a `FreeListNode`.
    ///
    /// Returns the adjusted size and alignment as a (size, align) tuple.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<FreeListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        // Minimum size must be size of a FreeListNode
        let size = layout.size().max(mem::size_of::<FreeListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Mutex<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Make sure allocated space can hold a FreeListNode when it's freed later
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.allocate_region(size, align) {
            // We already checked for overflow in is_suitable_region
            let alloc_end = alloc_start + size;
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                // We know leftover space is big enough to hold a FreeListNode
                allocator.free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Make sure the region we're about to free can hold a FreeListNode
        let (size, _) = LinkedListAllocator::size_align(layout);
        self.lock().free_region(ptr as usize, size)
    }
}
