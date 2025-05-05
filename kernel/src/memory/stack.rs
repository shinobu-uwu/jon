use log::{debug, info};

use crate::{
    arch::x86::{
        cpu::current_pcr,
        memory::{PMM, VMM},
    },
    memory::{address::VirtualAddress, paging::PageFlags, physical::PhysicalMemoryManager},
};

#[derive(Debug)]
pub struct Stack {
    bottom: VirtualAddress,
    size: usize,
    len: usize,
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

        Self {
            bottom,
            size,
            len: 0,
        }
    }

    pub const fn empty() -> Self {
        Self {
            bottom: VirtualAddress::new(0),
            size: 0,
            len: 0,
        }
    }

    pub fn top(&self) -> VirtualAddress {
        // stacks grow downwards, so the top is the bottom address + size - len
        VirtualAddress::new(self.bottom.as_usize() + self.size - self.len)
    }

    pub fn set_top(&mut self, top: VirtualAddress) {
        // stacks grow downwards, so the len is the bottom address + size - top
        self.len = self.bottom.as_usize() + self.size - top.as_usize();
        debug!("Stack top set to {:#x?}", top);
    }

    /// Resets the stack to its original state
    pub fn restart(&mut self) {
        debug!("Restarting stack");
        self.len = 0;
    }
}
