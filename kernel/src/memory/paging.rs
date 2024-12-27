use core::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::memory::address::VirtualAddress;

use super::address::PhysicalAddress;

pub trait VirtualMemoryManager {
    /// Maps a virtual address to a physical address with specified flags
    fn map(
        &mut self,
        virtual_addr: VirtualAddress,
        physical_addr: PhysicalAddress,
    ) -> Result<(), MapError>;

    /// Unmaps a virtual address
    fn unmap(&mut self, virtual_addr: VirtualAddress) -> Result<(), UnmapError>;
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
