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
    paging::{MapError, UnmapError, VirtualMemoryManager},
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
    ) -> Result<(), MapError> {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virtual_addr.as_u64()));
        let frame = PhysFrame::containing_address(PhysAddr::new(physical_addr.as_u64()));
        let mut pmm = PMM.lock();

        let result = unsafe {
            self.page_table.map_to(
                page,
                frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                &mut *pmm,
            )
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
