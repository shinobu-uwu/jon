use super::PMM;
use crate::memory::{
    address::{PhysicalAddress, VirtualAddress},
    paging::{MapError, PageFlags, UnmapError, VirtualMemoryManager},
    MEMORY_OFFSET, PAGE_SIZE,
};
use log::{debug, warn};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        mapper::{MapToError, TranslateResult, UnmapError as X86UnmapError},
        Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB, Translate,
    },
    PhysAddr, VirtAddr,
};

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

        debug!(
            "Created VMM with page table at physical address {:#x}",
            phys.as_u64()
        );
        Self { page_table }
    }

    pub fn page_flags(&self, virtual_addr: VirtualAddress) -> Option<PageTableFlags> {
        let virt_addr = VirtAddr::new(virtual_addr.as_u64());
        match self.page_table.translate(virt_addr) {
            TranslateResult::Mapped { flags, .. } => Some(flags),
            TranslateResult::NotMapped => None,
            TranslateResult::InvalidFrameAddress(_) => None,
        }
    }

    /// Maps a range of virtual addresses to physical addresses
    pub fn map_range(
        &mut self,
        virt_start: VirtualAddress,
        phys_start: PhysicalAddress,
        size: usize,
        flags: PageFlags,
    ) -> Result<(), MapError> {
        let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;

        for i in 0..pages {
            let virt_addr = VirtualAddress::new(virt_start.as_usize() + i * PAGE_SIZE);
            let phys_addr = PhysicalAddress::new(phys_start.as_usize() + i * PAGE_SIZE);

            self.map(virt_addr, phys_addr, flags)?;
        }

        Ok(())
    }

    /// Checks if a virtual address is mapped
    pub fn is_mapped(&self, virtual_addr: VirtualAddress) -> bool {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virtual_addr.as_u64()));
        self.page_table.translate_page(page).is_ok()
    }

    pub fn get_physical_address(&self, virtual_addr: VirtualAddress) -> Option<PhysicalAddress> {
        let virt_addr = VirtAddr::new(virtual_addr.as_u64());
        match self.page_table.translate(virt_addr) {
            TranslateResult::Mapped {
                frame,
                offset,
                flags,
            } => {
                let phys_addr = frame.start_address().as_u64() + offset;
                debug!(
                    "Translated {:#x} to {:#x} (flags: {:?})",
                    virtual_addr.as_u64(),
                    phys_addr,
                    flags
                );
                Some(PhysicalAddress::new(phys_addr as usize))
            }
            TranslateResult::NotMapped => {
                debug!("Address {:#x} is not mapped", virtual_addr.as_u64());
                None
            }
            TranslateResult::InvalidFrameAddress(addr) => {
                debug!(
                    "Address {:#x} translates to invalid frame address {:#x}",
                    virtual_addr.as_u64(),
                    addr
                );
                None
            }
        }
    }
}

impl VirtualMemoryManager for X86VirtualMemoryManager {
    fn map(
        &mut self,
        virtual_addr: VirtualAddress,
        physical_addr: PhysicalAddress,
        flags: PageFlags,
    ) -> Result<(), MapError> {
        debug!("Mapping {:#x?} -> {:#x?}", physical_addr, virtual_addr);

        if !virtual_addr.is_page_aligned() {
            debug!("Virtual address is not page aligned");
            return Err(MapError::InvalidAddress);
        }

        if !physical_addr.is_page_aligned() {
            debug!("Physical address is not page aligned");
            return Err(MapError::InvalidAddress);
        }

        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virtual_addr.as_u64()));
        let frame = PhysFrame::containing_address(PhysAddr::new(physical_addr.as_u64()));
        let mut pmm = PMM.lock();

        let result = unsafe { self.page_table.map_to(page, frame, flags.into(), &mut *pmm) };

        match result {
            Ok(flush) => {
                flush.flush();
                debug!("Successfully mapped page");
                Ok(())
            }
            Err(MapToError::FrameAllocationFailed) => {
                warn!("Failed to allocate page table frame");
                Err(MapError::NoPhysicalMemory)
            }
            Err(MapToError::ParentEntryHugePage) => {
                warn!("Cannot map: parent entry is a huge page");
                Err(MapError::InvalidAddress)
            }
            Err(MapToError::PageAlreadyMapped(_)) => {
                warn!(
                    "Virtual address {:#x} is already mapped",
                    virtual_addr.as_u64()
                );
                Err(MapError::AlreadyMapped)
            }
        }
    }

    fn unmap(&mut self, virtual_addr: VirtualAddress) -> Result<(), UnmapError> {
        if !virtual_addr.is_page_aligned() {
            return Err(UnmapError::InvalidAddress);
        }

        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virtual_addr.as_u64()));

        debug!("Unmapping page at {:#x}", virtual_addr.as_u64());

        match self.page_table.unmap(page) {
            Ok((_, flush)) => {
                flush.flush();
                debug!("Successfully unmapped page");
                Ok(())
            }
            Err(X86UnmapError::ParentEntryHugePage) => {
                warn!("Cannot unmap: parent entry is a huge page");
                Err(UnmapError::InvalidAddress)
            }
            Err(X86UnmapError::PageNotMapped) => {
                warn!("Page not mapped");
                Err(UnmapError::NotMapped)
            }
            Err(X86UnmapError::InvalidFrameAddress(_)) => {
                warn!("Invalid frame address");
                Err(UnmapError::InvalidAddress)
            }
        }
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
        if flags.contains(PageFlags::USER_ACCESSIBLE) {
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
