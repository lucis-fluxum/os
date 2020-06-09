use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

struct Dummy;

#[global_allocator]
static ALLOCATOR: Dummy = Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    log::error!("allocation error: {:?}", layout);
    crate::halt();
}
