use core::fmt::Display;

use super::address::PhysicalAddress;

pub trait PhysicalMemoryManager {
    fn init(&mut self) -> Result<(), ()>;

    /// Allocate a single physical frame
    fn allocate_frame(&mut self) -> Result<PhysicalAddress, FrameAllocationError>;

    /// Free a previously allocated physical frame
    fn free_frame(&mut self, frame: PhysicalAddress);

    /// Check if a specific frame is available
    fn is_frame_free(&self, frame: PhysicalAddress) -> bool;

    /// Get total physical memory
    fn total_memory(&self) -> usize;

    /// Get available physical memory
    fn available_memory(&self) -> usize;

    /// Allocate multiple contiguous frames
    fn allocate_frames(&mut self, count: usize) -> Option<&'static [PhysicalAddress]>;

    /// Reserve a specific frame range (for kernel, hardware, etc.)
    fn reserve_frame_range(&mut self, start: PhysicalAddress, end: PhysicalAddress);
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FrameAllocationError {
    OutOfMemory,
}

impl Display for FrameAllocationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use FrameAllocationError::*;

        let message = match self {
            OutOfMemory => "out of memory",
        };

        write!(f, "{message}")
    }
}

impl core::error::Error for FrameAllocationError {}
