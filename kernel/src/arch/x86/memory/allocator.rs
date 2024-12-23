use buddy_system_allocator::LockedHeap;
use log::debug;

use crate::{
    arch::x86::memory::{paging::X86VirtualMemoryManager, physical::X86PhysicalMemoryManager},
    memory::{
        address::VirtualAddress,
        paging::{VirtualMemoryManager, PAGE_SIZE},
        physical::PhysicalMemoryManager,
    },
    HHDM_REQUEST,
};

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap<32> = LockedHeap::new();

pub(super) fn init() {
    debug!("Creating PMM");
    let pmm = X86PhysicalMemoryManager::new();
    let frames_needed = (HEAP_SIZE + PAGE_SIZE - 1usize) / PAGE_SIZE;
    debug!("Frames needed: {}", frames_needed);
    let memory_offset = HHDM_REQUEST.get_response().unwrap().offset();
    debug!("Memory offset: {memory_offset}");
    debug!("Creating VMM");
    let mut vmm = X86VirtualMemoryManager::new(memory_offset, pmm);

    let heap_start = VirtualAddress::new(HEAP_START);

    for i in 0..frames_needed {
        let phys_addr = vmm.pmm.allocate_frame().expect("Failed to allocate frame");
        let virt_addr = VirtualAddress::new(heap_start.as_usize() + (i * PAGE_SIZE));
        debug!("Allocated frame: {:#?} -> {:#?}", virt_addr, phys_addr);
        vmm.map(virt_addr, phys_addr);
        debug!("Mapped frame");
    }

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START as usize, HEAP_SIZE as usize);
    }
}
