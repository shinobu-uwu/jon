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
/// Represents a kernel stack that grows downwards
pub struct KernelStack {
    /// The base address of the stack, since it grows downwards, this is the lowest address
    pub base: VirtualAddress,
    /// The size of the stack in bytes, does not need to be page-aligned as it will aligned by the
    /// new function
    pub size: usize,
    /// The top of the stack, since it grows downwards, this is the highest address when the stack
    /// is empty.
    pub top: VirtualAddress,
}

impl KernelStack {
    pub fn new(base: VirtualAddress, size: usize) -> Self {
        let frame = PMM.lock().allocate_contiguous(size).unwrap();
        VMM.lock()
            .map_range(base, frame, size, PageFlags::PRESENT | PageFlags::WRITABLE)
            .unwrap();

        Self {
            base,
            size,
            top: VirtualAddress::new(base.as_usize() + size),
        }
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
