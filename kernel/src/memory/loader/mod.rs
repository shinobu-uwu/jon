use crate::sched::memory::MemoryDescriptor;

use super::address::VirtualAddress;

pub mod elf;

pub trait Loader {
    fn load(
        &self,
        base_address: VirtualAddress,
        binary: &[u8],
    ) -> Result<(MemoryDescriptor, VirtualAddress), &'static str>;
}
