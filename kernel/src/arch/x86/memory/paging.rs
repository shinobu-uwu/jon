use log::debug;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        page_table::PageTableEntry, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags,
        PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

use crate::memory::{
    address::{PhysicalAddress, VirtualAddress},
    paging::{Entry, VirtualMemoryManager},
};

use super::physical::X86PhysicalMemoryManager;

pub struct X86VirtualMemoryManager {
    pub pmm: X86PhysicalMemoryManager,
    offset_page_table: OffsetPageTable<'static>,
}

impl X86VirtualMemoryManager {
    pub fn new(memory_offset: u64, pmm: X86PhysicalMemoryManager) -> Self {
        let (l4_frame, _) = Cr3::read();
        let phys = l4_frame.start_address();
        debug!("L4 physical address: {:?}", phys);

        let virt = VirtAddr::new(memory_offset) + phys.as_u64();
        debug!("Virtual address calculated: {:?}", virt);

        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
        let offset_page_table =
            unsafe { OffsetPageTable::new(&mut *page_table_ptr, VirtAddr::new(memory_offset)) };

        Self {
            pmm,
            offset_page_table,
        }
    }
}

impl VirtualMemoryManager for X86VirtualMemoryManager {
    fn map(&mut self, addr: VirtualAddress, target: PhysicalAddress) {
        let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(addr.as_u64()));
        let phys_addr = PhysAddr::new(target.as_u64());
        let frame = PhysFrame::containing_address(phys_addr);
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            self.offset_page_table
                .map_to(page, frame, flags, &mut self.pmm)
                .unwrap()
                .flush();
        };
    }

    fn unmap(&mut self, _addr: VirtualAddress) {
        todo!()
    }

    fn get_entry(&mut self, _addr: VirtualAddress) -> Option<&mut PageTableEntry> {
        todo!()
    }

    fn get_page_slice_mut<'a>(&mut self, _addr: VirtualAddress) -> &'a mut [u8] {
        todo!()
    }

    fn flush_cache_copy_user(
        &mut self,
        _start: VirtualAddress,
        _end: VirtualAddress,
        _execute: bool,
    ) {
        todo!()
    }
}

impl Entry for PageTableEntry {
    fn accessed(&self) -> bool {
        self.flags().contains(PageTableFlags::ACCESSED)
    }

    fn dirty(&self) -> bool {
        self.flags().contains(PageTableFlags::DIRTY)
    }

    fn writable(&self) -> bool {
        self.flags().contains(PageTableFlags::WRITABLE)
    }

    fn present(&self) -> bool {
        self.flags().contains(PageTableFlags::PRESENT)
    }

    fn clear_accessed(&mut self) {
        let mut flags = self.flags();
        flags.remove(PageTableFlags::ACCESSED);
        let addr = self.addr();
        self.set_addr(addr, flags);
    }

    fn clear_dirty(&mut self) {
        let mut flags = self.flags();
        flags.remove(PageTableFlags::DIRTY);
        let addr = self.addr();
        self.set_addr(addr, flags);
    }

    fn set_writable(&mut self, value: bool) {
        let mut flags = self.flags();
        flags.set(PageTableFlags::WRITABLE, value);
        let addr = self.addr();
        self.set_addr(addr, flags);
    }

    fn set_present(&mut self, value: bool) {
        let mut flags = self.flags();
        flags.set(PageTableFlags::PRESENT, value);
        let addr = self.addr();
        self.set_addr(addr, flags);
    }

    fn target(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.addr().as_u64() as usize)
    }

    fn set_target(&mut self, target: PhysicalAddress) {
        let flags = self.flags();
        self.set_addr(x86_64::PhysAddr::new(target.as_u64()), flags);
    }
}
