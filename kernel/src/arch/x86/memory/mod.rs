use buddy_system_allocator::LockedHeap;
use lazy_static::lazy_static;
use log::debug;
use physical::X86PhysicalMemoryManager;
use spinning_top::Spinlock;

use crate::memory::{paging::PAGE_SIZE, physical::PhysicalMemoryManager};

mod paging;
mod physical;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap<32> = LockedHeap::new();

lazy_static! {
    pub static ref PMM: Spinlock<X86PhysicalMemoryManager> =
        Spinlock::new(X86PhysicalMemoryManager::new());
}

pub(super) fn init() {
    let mut pmm = PMM.lock();
    pmm.init().unwrap();
    let frames_needed = (HEAP_SIZE + PAGE_SIZE - 1usize) / PAGE_SIZE;
    debug!("Frames needed: {}", frames_needed);
    for _ in 0..frames_needed {
        pmm.allocate_frame().unwrap();
    }

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START as usize, HEAP_SIZE as usize);
    }
}
