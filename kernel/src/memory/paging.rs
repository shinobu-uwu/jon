use super::address::{PhysicalAddress, VirtualAddress};
use bitflags::bitflags;

pub const PAGE_SIZE: usize = 4096;

pub trait VirtualMemoryManager {
    /// Map a page of virual address `addr` to the frame of physics address `target`
    /// Return the page table entry of the mapped virual address
    fn map(&mut self, addr: VirtualAddress, target: PhysicalAddress) -> &mut impl Page;

    /// Unmap a page of virual address `addr`
    fn unmap(&mut self, addr: VirtualAddress);

    /// Get the page table entry of a page of virual address `addr`
    /// If its page do not exist, return `None`
    fn get_entry(&mut self, addr: VirtualAddress) -> Option<&mut impl Page>;

    /// Get a mutable reference of the content of a page of virtual address `addr`
    fn get_page_slice_mut<'a>(&mut self, addr: VirtualAddress) -> &'a mut [u8];

    /// When copied user data (in page fault handler)，maybe need to flush I/D cache.
    fn flush_cache_copy_user(&mut self, start: VirtualAddress, end: VirtualAddress, execute: bool);
}

bitflags! {
    pub struct PageFlags: u64 {
        const PRESENT     = 1 << 0;
        const WRITABLE   = 1 << 1;
        const USER       = 1 << 2;
        const NO_EXECUTE = 1 << 63;

        // These might be architecture-specific but are common enough to include
        const ACCESSED   = 1 << 5;
        const DIRTY     = 1 << 6;
        const GLOBAL    = 1 << 8;
    }
}

pub trait Page {
    /// Get the physical address this page maps to (if present)
    fn physical_address(&self) -> Option<PhysicalAddress>;

    /// Set the physical address this page maps to
    /// This is architecture-agnostic as it works with our PhysicalAddress type
    fn set_address(&mut self, addr: PhysicalAddress);

    /// Get the page flags
    /// Returns our architecture-agnostic PageFlags
    fn flags(&self) -> PageFlags;

    /// Set the page flags
    /// Takes our architecture-agnostic PageFlags and translates them
    /// to architecture-specific bits internally
    fn set_flags(&mut self, flags: PageFlags);

    /// Is this entry pointing to a valid mapping?
    fn is_present(&self) -> bool;

    /// Clear this entry
    fn clear(&mut self);
}
