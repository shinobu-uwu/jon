use goblin::elf::{self, program_header::ProgramHeader, Elf};
use log::debug;

use crate::{
    arch::x86::memory::{PMM, VMM},
    memory::{
        address::VirtualAddress,
        loader::{Loader, LoadingError},
        paging::{align_down, PageFlags},
        physical::PhysicalMemoryManager,
        PAGE_SIZE,
    },
    sched::memory::{MemoryAreaType, MemoryDescriptor},
};

pub struct ElfLoader;

impl ElfLoader {
    pub const fn new() -> Self {
        Self
    }

    fn load_segment(
        &self,
        base_address: VirtualAddress,
        binary: &[u8],
        segment: &ProgramHeader,
    ) -> Result<(), LoadingError> {
        debug!("Loading segment at {:#x?}", segment);
        debug!("Segment p_vaddr: {:#x?}", segment.p_vaddr);
        if segment.p_memsz == 0 {
            return Ok(());
        }

        if segment.p_offset + segment.p_filesz > binary.len() as u64 {
            return Err(LoadingError::InvalidInput);
        }

        let vaddr = align_down(segment.p_vaddr as usize, PAGE_SIZE);
        let mapped_size = align_down(
            segment.p_memsz as usize + (segment.p_vaddr as usize % PAGE_SIZE),
            PAGE_SIZE,
        ) + PAGE_SIZE;

        let phys = PMM
            .lock()
            .allocate_contiguous(mapped_size)
            .map_err(|_| LoadingError::MemoryAllocationError)?;
        let virt = VirtualAddress::new(base_address.as_usize() + vaddr);
        let flags = PageFlags::USER_ACCESSIBLE | PageFlags::PRESENT | PageFlags::WRITABLE;

        VMM.lock()
            .map_range(virt, phys, mapped_size, flags)
            .map_err(|_| LoadingError::MappingError)?;
        unsafe {
            debug!("Copying segment to {:#x?}", virt);
            core::ptr::copy_nonoverlapping(
                binary.as_ptr().offset(segment.p_offset as isize),
                (base_address.as_usize() + segment.p_vaddr as usize) as *mut u8,
                segment.p_filesz as usize,
            );
        }

        if segment.p_memsz > segment.p_filesz {
            let bss_start = virt.offset(segment.p_filesz as usize).as_u64();
            let bss_size = segment.p_memsz - segment.p_filesz;
            debug!(
                "Zeroing BSS at {:#x?} with size {:#x?}",
                bss_start, bss_size
            );

            unsafe {
                core::ptr::write_bytes(bss_start as *mut u8, 0, bss_size as usize);
            }
        }

        Ok(())
    }
}

impl Loader for ElfLoader {
    fn load(
        &self,
        base_address: VirtualAddress,
        binary: &[u8],
    ) -> Result<(MemoryDescriptor, VirtualAddress), LoadingError> {
        let elf = Elf::parse(binary).map_err(|_| LoadingError::ParseError)?;
        let mut memory_descriptor = MemoryDescriptor::new();

        for ph in elf.program_headers {
            if ph.p_type != elf::program_header::PT_LOAD {
                continue;
            }

            let start = ph.p_vaddr;
            let end = start + ph.p_memsz;
            let flags = PageFlags::USER_ACCESSIBLE | PageFlags::PRESENT | PageFlags::WRITABLE;

            self.load_segment(base_address, binary, &ph)?;

            let area_type = if ph.p_flags & elf::program_header::PF_X != 0 {
                MemoryAreaType::Text
            } else if ph.p_flags & elf::program_header::PF_W != 0 {
                MemoryAreaType::Data
            } else {
                MemoryAreaType::Heap
            };

            memory_descriptor.add_region(start, end, flags, area_type);
        }

        Ok((memory_descriptor, base_address.offset(elf.entry as usize)))
    }
}
