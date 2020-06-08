use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

const PAGE_SIZE: usize = 4096;

/// Initialize a new OffsetPageTable mapper.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn initialize_mapper(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let (level_4_table_frame, _) = Cr3::read();
    let phys_addr = level_4_table_frame.start_address();
    let virt_addr = physical_memory_offset + phys_addr.as_u64();
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();
    OffsetPageTable::new(&mut *page_table_ptr, physical_memory_offset)
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn new(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    // Map usable memory regions to physical frames by breaking the address ranges
    // into page-sized chunks.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_map
            .iter()
            .filter(|region| region.region_type == MemoryRegionType::Usable)
            .map(|usable_region| usable_region.range.start_addr()..usable_region.range.end_addr())
            // Divide regions into page-sized chunks (may leave extra bytes at end of some regions)
            .flat_map(|range| range.step_by(PAGE_SIZE))
            // Yield physical frames corresponding to start of each chunk
            .map(|address| PhysFrame::containing_address(PhysAddr::new(address)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
