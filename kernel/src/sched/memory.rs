use alloc::vec::Vec;

use crate::memory::{address::VirtualAddress, paging::PageFlags};

#[derive(Debug)]
pub struct MemoryDescriptor {
    pub regions: Vec<VirtualMemoryArea>,
    pub start_brk: u64,
    pub brk: u64,
    pub start_stack: u64,
    pub stack: u64,
}

#[derive(Debug)]
struct VirtualMemoryArea {
    pub start: u64,
    pub end: u64,
    pub flags: PageFlags,
    pub area_type: MemoryAreaType,
}

#[derive(Debug)]
pub enum MemoryAreaType {
    Text,
    Data,
    Heap,
    Stack,
}

impl MemoryDescriptor {
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            start_brk: 0,
            brk: 0,
            start_stack: 0,
            stack: 0,
        }
    }

    pub fn add_region(
        &mut self,
        start: u64,
        end: u64,
        flags: PageFlags,
        area_type: MemoryAreaType,
    ) {
        self.regions.push(VirtualMemoryArea {
            start,
            end,
            flags,
            area_type,
        });
    }

    pub fn find_region(&self, address: VirtualAddress) -> Option<&VirtualMemoryArea> {
        let addr = address.as_u64();

        self.regions
            .iter()
            .find(|region| region.start <= addr && addr < region.end)
    }
}
