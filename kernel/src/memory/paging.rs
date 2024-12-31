use bitflags::bitflags;
use core::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::memory::address::VirtualAddress;

use super::{address::PhysicalAddress, MEMORY_OFFSET};

pub trait VirtualMemoryManager {
    /// Maps a virtual address to a physical address with specified flags
    fn map(
        &mut self,
        virtual_addr: VirtualAddress,
        physical_addr: PhysicalAddress,
        flags: PageFlags,
    ) -> Result<(), MapError>;

    /// Unmaps a virtual address
    fn unmap(&mut self, virtual_addr: VirtualAddress) -> Result<(), UnmapError>;
}
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PageFlags: u64 {
        const PRESENT       = 1 << 0;
        const WRITABLE     = 1 << 1;
        const USER         = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE     = 1 << 4;
        const ACCESSED     = 1 << 5;
        const DIRTY        = 1 << 6;
        const HUGE_PAGE    = 1 << 7;
        const GLOBAL       = 1 << 8;
        const NO_EXECUTE   = 1 << 63;
    }
}

#[derive(Debug)]
pub enum MapError {
    AlreadyMapped,
    NoPhysicalMemory,
    InvalidAddress,
}

#[derive(Debug)]
pub enum UnmapError {
    NotMapped,
    InvalidAddress,
}

#[inline]
pub fn phys_to_virt(addr: usize) -> usize {
    *MEMORY_OFFSET as usize + addr
}

impl Display for MapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use MapError::*;
        let message = match self {
            AlreadyMapped => "already mapped",
            NoPhysicalMemory => "no physical memory",
            InvalidAddress => "invalid address",
        };

        write!(f, "{}", message)
    }
}

impl Display for UnmapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use UnmapError::*;
        let message = match self {
            NotMapped => "not mapped",
            InvalidAddress => "invalid address",
        };

        write!(f, "{}", message)
    }
}

impl Error for MapError {}
impl Error for UnmapError {}
