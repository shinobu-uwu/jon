use super::PAGE_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct VirtualAddress(usize);

impl PhysicalAddress {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub const fn as_usize(&self) -> usize {
        self.0
    }

    pub fn page_align(&self) -> Self {
        Self(self.0 & !(PAGE_SIZE - 1))
    }

    pub fn is_page_aligned(&self) -> bool {
        (self.0 & (PAGE_SIZE - 1)) == 0
    }

    pub fn offset(&self, bytes: usize) -> Self {
        Self(self.0 + bytes)
    }
}

impl VirtualAddress {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub const fn as_usize(&self) -> usize {
        self.0
    }

    pub fn page_align(&self) -> Self {
        Self(self.0 & !(4096 - 1))
    }

    pub fn is_page_aligned(&self) -> bool {
        (self.0 & (PAGE_SIZE - 1)) == 0
    }

    pub fn offset(&self, bytes: usize) -> Self {
        Self(self.0 + bytes)
    }

    pub fn page_index(&self) -> usize {
        self.0 / PAGE_SIZE
    }
}
