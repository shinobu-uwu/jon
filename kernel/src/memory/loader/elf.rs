use goblin::elf::{self, program_header::ProgramHeader, reloc::R_X86_64_RELATIVE, Elf};
use log::{debug, error};

use crate::{
    arch::x86::memory::{PMM, VMM},
    memory::{
        address::VirtualAddress,
        loader::{Loader, LoadingError},
        paging::{align_down, align_up, PageFlags},
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
        let file_offset = segment.p_vaddr as usize % PAGE_SIZE;
        let total_size = segment.p_memsz as usize + file_offset;
        let mapped_size = align_up(total_size, PAGE_SIZE);

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
            let src = binary.as_ptr().offset(segment.p_offset as isize);
            let dest = (base_address.as_usize() + segment.p_vaddr as usize) as *mut u8;
            debug!("Copying {} bytes to {:p}", segment.p_filesz, dest);
            core::ptr::copy_nonoverlapping(src, dest, segment.p_filesz as usize);
        }

        if segment.p_memsz > segment.p_filesz {
            let bss_start =
                base_address.as_usize() + segment.p_vaddr as usize + segment.p_filesz as usize;
            let bss_size = (segment.p_memsz - segment.p_filesz) as usize;

            debug!("Zeroing BSS at {:#x?} ({} bytes)", bss_start, bss_size);
            unsafe {
                core::ptr::write_bytes(bss_start as *mut u8, 0, bss_size);
            }
        }

        Ok(())
    }

    fn apply_relocations(&self, elf: &Elf, base_address: usize) {
        for rela in &elf.dynrelas {
            if rela.r_type == R_X86_64_RELATIVE {
                debug!("Applying relocation: {:#x?}", rela);
                let reloc_addr = base_address + rela.r_offset as usize;
                let value = base_address + rela.r_addend.unwrap_or(0) as usize;
                unsafe {
                    core::ptr::write_unaligned(reloc_addr as *mut usize, value);
                }
            } else {
                error!("Unsupported relocation type: {}", rela.r_type);
            }
        }
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

        for ph in elf.program_headers.iter() {
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

        self.apply_relocations(&elf, base_address.as_usize());
        let entry = base_address.offset(elf.entry as usize);
        memory_descriptor.entrypoint = entry.as_u64();

        Ok((memory_descriptor, entry))
    }
}
