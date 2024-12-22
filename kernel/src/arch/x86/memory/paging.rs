use x86_64::structures::paging::{FrameAllocator, Page, PageTable, Size4KiB};

pub struct X86VirtualMemoryManager {
    p4_table: &'static PageTable,
}

impl crate::memory::paging::Page for Page {
    fn physical_address(&self) -> Option<crate::memory::address::PhysicalAddress> {
        todo!()
    }

    fn set_address(&mut self, addr: crate::memory::address::PhysicalAddress) {
        todo!()
    }

    fn flags(&self) -> crate::memory::paging::PageFlags {
        todo!()
    }

    fn set_flags(&mut self, flags: crate::memory::paging::PageFlags) {
        todo!()
    }

    fn is_present(&self) -> bool {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }
}
