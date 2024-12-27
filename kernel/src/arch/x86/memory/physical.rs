use bitmap_allocator::BitAlloc;
use limine::memory_map::EntryType;
use log::debug;
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
}

impl X86PhysicalMemoryManager {
    pub fn new() -> Self {
        let entries = &MEMORY_MAP;
        let max_addr = entries.iter().map(|e| e.base + e.length).max().unwrap() as usize;
        debug!("Max address found: {:#x}", max_addr);
        let total_frames = (max_addr + PAGE_SIZE - 1) / PAGE_SIZE;
        debug!("Total frames: {}", total_frames);
        let mut alloc = FRAME_ALLOCATOR.lock();

        for entry in entries.iter() {
            if entry.entry_type != EntryType::USABLE {
                let start = (entry.base as usize) / PAGE_SIZE;
                let end = ((entry.base + entry.length) as usize + PAGE_SIZE - 1) / PAGE_SIZE;
                alloc.insert(start..end);
                debug!("Marked frames from {:#x} to {:#x} as used", start, end);
            }
        }

        Self { total_frames }
    }
}

impl PhysicalMemoryManager for X86PhysicalMemoryManager {
    fn allocate(&mut self) -> Result<PhysicalAddress, FrameAllocationError> {
        debug!("Allocating new frame");
        let mut allocator = FRAME_ALLOCATOR.lock();

        match allocator.alloc() {
            Some(f) => {
                debug!("Allocated frame: {:#x}", f * PAGE_SIZE);
                Ok(PhysicalAddress::new(f * PAGE_SIZE))
            }
            None => {
                debug!("Failed to allocate new frame");
                Err(FrameAllocationError::OutOfMemory)
            }
        }
    }

    fn free(&mut self, frame: PhysicalAddress) {
        let mut allocator = FRAME_ALLOCATOR.lock();
        allocator.dealloc(frame.as_usize() / PAGE_SIZE);
    }

    fn is_frame_free(&self, _frame: PhysicalAddress) -> bool {
        todo!()
    }

    fn total_memory(&self) -> usize {
        todo!()
    }

    fn available_memory(&self) -> usize {
        self.total_frames * PAGE_SIZE
    }

    fn allocate_frames(&mut self, _count: usize) -> Option<&[PhysicalAddress]> {
        todo!()
    }

    fn reserve_frame_range(&mut self, _start: PhysicalAddress, _end: PhysicalAddress) {
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
