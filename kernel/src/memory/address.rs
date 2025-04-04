use super::PAGE_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress(usize);

impl PhysicalAddress {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub const fn as_usize(&self) -> usize {
        self.0
    }

    pub const fn as_u64(&self) -> u64 {
        self.0 as u64
    }

    pub fn align_down(&self, align: usize) -> Self {
        Self(self.0 & !(align - 1))
    }

    pub fn align_up(&self, align: usize) -> Self {
        Self((self.0 + align - 1) & !(align - 1))
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

    pub const fn as_u64(&self) -> u64 {
        self.0 as u64
    }

    pub fn page_align(&self) -> Self {
        Self(self.0 & !(PAGE_SIZE - 1))
    }

    pub fn align_up(&self, align: usize) -> Self {
        Self((self.0 + align - 1) & !(align - 1))
    }

    pub fn align_down(&self, align: usize) -> Self {
        Self(self.0 & !(align - 1))
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
