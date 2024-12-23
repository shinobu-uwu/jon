use super::address::{PhysicalAddress, VirtualAddress};
use bitflags::bitflags;

pub const PAGE_SIZE: usize = 4096;

pub trait VirtualMemoryManager {
    /// Map a page of virual address `addr` to the frame of physics address `target`
    /// Return the page table entry of the mapped virual address
    fn map(&mut self, addr: VirtualAddress, target: PhysicalAddress);

    /// Unmap a page of virual address `addr`
    fn unmap(&mut self, addr: VirtualAddress);

    /// Get the page table entry of a page of virual address `addr`
    /// If its page do not exist, return `None`
    fn get_entry(&mut self, addr: VirtualAddress) -> Option<&mut impl Entry>;

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

pub trait Entry {
    /// A bit set by hardware when the page is accessed
    fn accessed(&self) -> bool;
    /// A bit set by hardware when the page is written
    fn dirty(&self) -> bool;
    /// Will PageFault when try to write page where writable=0
    fn writable(&self) -> bool;
    /// Will PageFault when try to access page where present=0
    fn present(&self) -> bool;

    fn clear_accessed(&mut self);
    fn clear_dirty(&mut self);
    fn set_writable(&mut self, value: bool);
    fn set_present(&mut self, value: bool);

    /// The target physics address in the entry
    /// Can be used for other purpose if present=0
    fn target(&self) -> PhysicalAddress;
    fn set_target(&mut self, target: PhysicalAddress);
}
