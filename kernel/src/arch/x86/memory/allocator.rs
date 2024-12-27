use buddy_system_allocator::LockedHeap;
use log::debug;
use x86_64::{
    structures::paging::{Page, Size4KiB},
    VirtAddr,
};

use crate::memory::{
    address::VirtualAddress, paging::VirtualMemoryManager, physical::PhysicalMemoryManager,
};

use super::{PMM, VMM};

const HEAP_START: usize = 0x_4444_4444_0000;
const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
pub static GLOBAL_ALLOC: LockedHeap<32> = LockedHeap::empty();

pub fn init() {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        debug!("heap_start: {:#?}", heap_start);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        debug!("heap_end: {:#?}", heap_end);
        let heap_start_page = Page::<Size4KiB>::containing_address(heap_start);
        debug!("heap_start_page: {:#?}", heap_start_page);
        let heap_end_page = Page::containing_address(heap_end);
        debug!("heap_end_page: {:#?}", heap_end_page);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = PMM.lock().allocate().unwrap();
        VMM.lock()
            .map(
                VirtualAddress::new(page.start_address().as_u64() as usize),
                frame,
            )
            .unwrap();
    }

    unsafe { GLOBAL_ALLOC.lock().init(HEAP_START, HEAP_SIZE) };
}
