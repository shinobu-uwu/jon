use alloc::vec::Vec;
use bitflags::bitflags;

#[derive(Debug)]
pub struct MemoryDescriptor {
    pub regions: Vec<MemoryRegion>,
    pub start_brk: u64,
    pub brk: u64,
    pub start_stack: u64,
    pub stack: u64,
}

#[derive(Debug)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
    pub flags: MemoryRegionFlags,
}

bitflags! {
    #[derive(Debug)]
    pub struct MemoryRegionFlags: u64 {
        const READABLE = 1 << 0;
        const WRITABLE = 1 << 1;
        const EXECUTABLE = 1 << 2;
        const USER = 1 << 3;
        const DEVICE = 1 << 4;
        const MMIO = 1 << 5;
    }
}
