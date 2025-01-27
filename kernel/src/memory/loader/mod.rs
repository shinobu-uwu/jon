use core::{error::Error, fmt::Display};

use crate::sched::memory::MemoryDescriptor;

use super::address::VirtualAddress;

pub mod elf;

pub trait Loader {
    fn load(
        &self,
        base_address: VirtualAddress,
        binary: &[u8],
    ) -> Result<(MemoryDescriptor, VirtualAddress), LoadingError>;
}

#[derive(Debug)]
pub enum LoadingError {
    ParseError,
    MemoryAllocationError,
    MappingError,
    InvalidInput,
}

impl Display for LoadingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LoadingError::ParseError => write!(f, "Failed to parse ELF binary"),
            LoadingError::MemoryAllocationError => write!(f, "Failed to allocate memory"),
            LoadingError::MappingError => write!(f, "Failed to map memory"),
            LoadingError::InvalidInput => write!(f, "Invalid input"),
        }
    }
}

impl Error for LoadingError {}
