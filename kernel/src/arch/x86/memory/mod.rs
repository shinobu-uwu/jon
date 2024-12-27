use lazy_static::lazy_static;
use log::debug;
use paging::X86VirtualMemoryManager;
use physical::X86PhysicalMemoryManager;
use spinning_top::Spinlock;

pub mod allocator;
pub mod paging;
pub mod physical;

lazy_static! {
    pub static ref PMM: Spinlock<X86PhysicalMemoryManager> = {
        debug!("Creating PMM");
        let pmm = X86PhysicalMemoryManager::new();
        debug!("Created PMM successfully");

        Spinlock::new(pmm)
    };
    pub static ref VMM: Spinlock<X86VirtualMemoryManager> = {
        debug!("Creating VMM");
        let vmm = X86VirtualMemoryManager::new();
        debug!("Created VMM successfully");

        Spinlock::new(vmm)
    };
}
