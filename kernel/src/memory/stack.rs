use log::debug;

use crate::{
    arch::x86::memory::{PMM, VMM},
    memory::{address::VirtualAddress, paging::PageFlags, physical::PhysicalMemoryManager},
};

#[derive(Debug)]
pub struct Stack {
    bottom: VirtualAddress,
    size: usize,
}

impl Stack {
    pub fn new(bottom: VirtualAddress, size: usize) -> Self {
        debug!(
            "Creating stack starting at {:#x?} with size {:#x}",
            bottom, size
        );
        let bottom_phys = PMM.lock().allocate_contiguous(size).unwrap();
        VMM.lock()
            .map_range(
                bottom,
                bottom_phys,
                size,
                PageFlags::PRESENT | PageFlags::WRITABLE | PageFlags::USER_ACCESSIBLE,
            )
            .unwrap();

        Self { bottom, size }
    }

    pub fn top(&self) -> VirtualAddress {
        VirtualAddress::new(self.bottom.as_usize() + self.size)
    }
}
