use crate::{
    arch::x86::memory::{PMM, VMM},
    memory::{
        address::{PhysicalAddress, VirtualAddress},
        paging::{PageFlags, VirtualMemoryManager},
        physical::PhysicalMemoryManager,
        PAGE_SIZE,
    },
};

#[derive(Debug)]
pub struct KernelStack {
    pub base: VirtualAddress,
    pub size: usize,
}

impl KernelStack {
    pub fn new(base: VirtualAddress, size: usize) -> Self {
        let frame = PMM.lock().allocate_contiguous(size).unwrap();
        VMM.lock()
            .map_range(base, frame, size, PageFlags::PRESENT | PageFlags::WRITABLE)
            .unwrap();

        Self { base, size }
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let mut vmm = VMM.lock();
        let mut pmm = PMM.lock();
        let phys = vmm.get_physical_address(self.base).unwrap().as_usize();

        for i in 0..self.size / PAGE_SIZE {
            pmm.free(PhysicalAddress::new(phys + i * PAGE_SIZE));
            vmm.unmap(VirtualAddress::new(self.base.as_usize() + i * PAGE_SIZE))
                .unwrap();
        }
    }
}
