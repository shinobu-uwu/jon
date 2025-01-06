use core::{error::Error, fmt::Display};

use bitmap_allocator::{BitAlloc, BitAlloc256M};
use spinning_top::Spinlock;

use super::address::PhysicalAddress;

pub static FRAME_ALLOCATOR: Spinlock<BitAlloc256M> = Spinlock::new(BitAlloc256M::DEFAULT);

pub trait PhysicalMemoryManager {
    /// Allocate a single physical frame
    fn allocate(&mut self) -> Result<PhysicalAddress, FrameAllocationError>;

    /// Alocate a block of contiguous frames returning the address of the start of the block
    fn allocate_contiguous(&mut self, size: usize)
        -> Result<PhysicalAddress, FrameAllocationError>;

    /// Free a previously allocated physical frame
    fn free(&mut self, frame: PhysicalAddress);

    /// Check if a specific frame is available
    fn is_frame_free(&self, frame: PhysicalAddress) -> bool;

    /// Get total physical memory
    fn total_memory(&self) -> usize;

    /// Get available physical memory
    fn available_memory(&self) -> usize;

    /// Reserve a specific frame range (for kernel, hardware, etc.)
    fn reserve_frame_range(&mut self, start: PhysicalAddress, end: PhysicalAddress);
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FrameAllocationError {
    OutOfMemory,
    Reserved,
}

impl Display for FrameAllocationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use FrameAllocationError::*;

        let message = match self {
            OutOfMemory => "out of memory",
            Reserved => "reserved",
        };

        write!(f, "{message}")
    }
}

impl Error for FrameAllocationError {}
