use bitmap_allocator::BitAlloc;
use limine::memory_map::EntryType;
use log::{debug, warn};
use x86_64::{
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
    PhysAddr,
};

use crate::memory::{
    address::PhysicalAddress,
    physical::{FrameAllocationError, PhysicalMemoryManager, FRAME_ALLOCATOR},
    MEMORY_MAP, PAGE_SIZE,
};

#[derive(Debug)]
pub struct X86PhysicalMemoryManager {
    total_frames: usize,
    usable_frames: usize,
}

impl X86PhysicalMemoryManager {
    pub fn new() -> Self {
        let entries = &MEMORY_MAP;
        let max_addr = entries.iter().map(|e| e.base + e.length).max().unwrap() as usize;
        debug!("Max physical address found: {:#x}", max_addr);

        let total_frames = (max_addr + PAGE_SIZE - 1) / PAGE_SIZE;
        let mut usable_frames = 0;

        let mut allocator = FRAME_ALLOCATOR.lock();

        // By default, all frames are unavailable
        // Mark usable regions as available in the bitmap
        for entry in entries.iter() {
            if entry.entry_type == EntryType::USABLE {
                let start = (entry.base as usize) / PAGE_SIZE;
                let end = ((entry.base + entry.length) as usize) / PAGE_SIZE;

                debug!(
                    "Marking usable region: {:#x} - {:#x}",
                    entry.base,
                    entry.base + entry.length
                );
                allocator.insert(start..end); // Mark as available
                usable_frames += end - start;
            }
        }

        debug!(
            "Total frames: {}, Usable frames: {}",
            total_frames, usable_frames
        );

        Self {
            total_frames,
            usable_frames,
        }
    }
}

impl PhysicalMemoryManager for X86PhysicalMemoryManager {
    fn allocate(&mut self) -> Result<PhysicalAddress, FrameAllocationError> {
        debug!("Allocating new frame");
        let mut allocator = FRAME_ALLOCATOR.lock();

        match allocator.alloc() {
            Some(frame) => {
                let addr = PhysicalAddress::new(frame * PAGE_SIZE);
                debug!("Allocated frame at physical address {:#x}", addr.as_u64());
                Ok(addr)
            }
            None => {
                warn!("Out of physical memory!");
                Err(FrameAllocationError::OutOfMemory)
            }
        }
    }

    fn allocate_contiguous(
        &mut self,
        size: usize,
    ) -> Result<PhysicalAddress, FrameAllocationError> {
        let frames_needed = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        let mut allocator = FRAME_ALLOCATOR.lock();

        match allocator.alloc_contiguous(None, frames_needed, 0) {
            Some(start_frame) => {
                let addr = PhysicalAddress::new(start_frame * PAGE_SIZE);
                debug!(
                    "Allocated {} contiguous frames starting at {:#x}",
                    frames_needed,
                    addr.as_u64()
                );
                Ok(addr)
            }
            None => {
                warn!("Failed to allocate {} contiguous frames", frames_needed);
                Err(FrameAllocationError::OutOfMemory)
            }
        }
    }

    fn free(&mut self, frame: PhysicalAddress) {
        let frame_number = frame.as_usize() / PAGE_SIZE;
        let mut allocator = FRAME_ALLOCATOR.lock();
        allocator.dealloc(frame_number);
        debug!("Freed frame at {:#x}", frame.as_u64());
    }

    fn available_memory(&self) -> usize {
        let allocator = FRAME_ALLOCATOR.lock();
        let mut free_frames = 0;
        for frame in 0..self.total_frames {
            if allocator.test(frame) {
                // test returns true if frame is available
                free_frames += 1;
            }
        }
        free_frames * PAGE_SIZE
    }

    fn total_memory(&self) -> usize {
        self.total_frames * PAGE_SIZE
    }

    fn is_frame_free(&self, frame: PhysicalAddress) -> bool {
        let frame_number = frame.as_usize() / PAGE_SIZE;
        FRAME_ALLOCATOR.lock().test(frame_number) // true means available
    }

    fn reserve_frame_range(&mut self, start: PhysicalAddress, end: PhysicalAddress) {
        todo!()
    }
}

unsafe impl FrameAllocator<Size4KiB> for X86PhysicalMemoryManager {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        match self.allocate() {
            Ok(f) => Some(PhysFrame::from_start_address(PhysAddr::new(f.as_u64())).unwrap()),
            Err(_) => None,
        }
    }
}
