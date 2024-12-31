use log::debug;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        mapper::{MapToError, UnmapError as X86UnmapError},
        Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

use crate::memory::{
    address::{PhysicalAddress, VirtualAddress},
    paging::{MapError, PageFlags, UnmapError, VirtualMemoryManager},
    MEMORY_OFFSET,
};

use super::PMM;

#[derive(Debug)]
pub struct X86VirtualMemoryManager {
    page_table: OffsetPageTable<'static>,
}

impl X86VirtualMemoryManager {
    pub fn new() -> Self {
        let (l4_table_frame, _) = Cr3::read();
        let phys = l4_table_frame.start_address();
        let memory_offset = *MEMORY_OFFSET;
        let virt = VirtAddr::new(memory_offset) + phys.as_u64();
        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
        let page_table =
            unsafe { OffsetPageTable::new(&mut *page_table_ptr, VirtAddr::new(memory_offset)) };

        Self { page_table }
    }
}

impl VirtualMemoryManager for X86VirtualMemoryManager {
    fn map(
        &mut self,
        virtual_addr: VirtualAddress,
        physical_addr: PhysicalAddress,
        flags: PageFlags,
    ) -> Result<(), MapError> {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virtual_addr.as_u64()));
        let frame = PhysFrame::containing_address(PhysAddr::new(physical_addr.as_u64()));
        let mut pmm = PMM.lock();

        let result = unsafe {
            self.page_table
                .map_to(page, frame, PageTableFlags::from(flags), &mut *pmm)
        };
        let mapper_flush = result.map_err(|e| match e {
            MapToError::FrameAllocationFailed => MapError::NoPhysicalMemory,
            MapToError::ParentEntryHugePage => MapError::InvalidAddress,
            MapToError::PageAlreadyMapped(_) => MapError::AlreadyMapped,
        })?;
        mapper_flush.flush();
        debug!("Mapped {:?} -> {:?}", virtual_addr, physical_addr);

        Ok(())
    }

    fn unmap(&mut self, virtual_addr: VirtualAddress) -> Result<(), UnmapError> {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virtual_addr.as_u64()));
        let result = self.page_table.unmap(page);
        let (_, flush) = result.map_err(|e| match e {
            X86UnmapError::ParentEntryHugePage => UnmapError::InvalidAddress,
            X86UnmapError::PageNotMapped => UnmapError::NotMapped,
            X86UnmapError::InvalidFrameAddress(_) => UnmapError::InvalidAddress,
        })?;

        flush.flush();

        Ok(())
    }
}

#[cfg(target_arch = "x86_64")]
impl From<PageFlags> for PageTableFlags {
    fn from(flags: PageFlags) -> Self {
        use x86_64::structures::paging::PageTableFlags;
        let mut x86_flags = PageTableFlags::empty();

        if flags.contains(PageFlags::PRESENT) {
            x86_flags |= PageTableFlags::PRESENT;
        }
        if flags.contains(PageFlags::WRITABLE) {
            x86_flags |= PageTableFlags::WRITABLE;
        }
        if flags.contains(PageFlags::USER) {
            x86_flags |= PageTableFlags::USER_ACCESSIBLE;
        }
        if flags.contains(PageFlags::WRITE_THROUGH) {
            x86_flags |= PageTableFlags::WRITE_THROUGH;
        }
        if flags.contains(PageFlags::NO_CACHE) {
            x86_flags |= PageTableFlags::NO_CACHE;
        }
        if flags.contains(PageFlags::ACCESSED) {
            x86_flags |= PageTableFlags::ACCESSED;
        }
        if flags.contains(PageFlags::DIRTY) {
            x86_flags |= PageTableFlags::DIRTY;
        }
        if flags.contains(PageFlags::HUGE_PAGE) {
            x86_flags |= PageTableFlags::HUGE_PAGE;
        }
        if flags.contains(PageFlags::GLOBAL) {
            x86_flags |= PageTableFlags::GLOBAL;
        }
        if flags.contains(PageFlags::NO_EXECUTE) {
            x86_flags |= PageTableFlags::NO_EXECUTE;
        }

        x86_flags
    }
}
