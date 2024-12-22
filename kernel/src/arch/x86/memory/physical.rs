use core::usize;

use bitmap_allocator::{BitAlloc, BitAlloc16M};
use limine::memory_map::EntryType;
use log::debug;

use crate::memory::{
    address::PhysicalAddress,
    paging::PAGE_SIZE,
    physical::{FrameAllocationError, PhysicalMemoryManager},
    MEMORY_MAP_REQUEST,
};

pub struct X86PhysicalMemoryManager {
    bitmap: BitAlloc16M,
    total_frames: usize,
}

impl X86PhysicalMemoryManager {
    pub fn new() -> Self {
        Self {
            bitmap: BitAlloc16M::default(),
            total_frames: 0,
        }
    }
}

impl PhysicalMemoryManager for X86PhysicalMemoryManager {
    fn init(&mut self) -> Result<(), ()> {
        let entries = MEMORY_MAP_REQUEST.get_response().ok_or(())?.entries();
        debug!("Bootloader reported {} entries", entries.len());
        let max_addr = entries.iter().map(|e| e.base + e.length).max().unwrap_or(0) as usize;
        self.total_frames = (max_addr + PAGE_SIZE - 1) / PAGE_SIZE;
        self.bitmap.insert(0..self.total_frames);

        for entry in entries.iter() {
            if entry.entry_type == EntryType::USABLE {
                let start_frame = (entry.base as usize) / PAGE_SIZE;
                let end_frame = ((entry.base + entry.length) as usize + PAGE_SIZE - 1) / PAGE_SIZE;
                self.bitmap.remove(start_frame..end_frame);
                debug!("Marked frames {}..{} as free", start_frame, end_frame);
            }
        }

        Ok(())
    }

    fn allocate_frame(&mut self) -> Result<PhysicalAddress, FrameAllocationError> {
        match self.bitmap.alloc() {
            Some(frame_number) => {
                debug!("Allocated frame {}", frame_number);
                Ok(PhysicalAddress::new(frame_number * PAGE_SIZE))
            }
            None => Err(FrameAllocationError::OutOfMemory),
        }
    }

    fn free_frame(&mut self, frame: PhysicalAddress) {
        todo!()
    }

    fn is_frame_free(&self, frame: PhysicalAddress) -> bool {
        todo!()
    }

    fn total_memory(&self) -> usize {
        todo!()
    }

    fn available_memory(&self) -> usize {
        todo!()
    }

    fn allocate_frames(&mut self, count: usize) -> Option<&'static [PhysicalAddress]> {
        todo!()
    }

    fn reserve_frame_range(&mut self, start: PhysicalAddress, end: PhysicalAddress) {
        todo!()
    }
}
