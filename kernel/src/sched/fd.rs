use bitmap_allocator::{BitAlloc, BitAlloc4K};
use lazy_static::lazy_static;
use libjon::fd::{FileDescriptorFlags, FileDescriptorId};
use spinning_top::Spinlock;

use crate::scheme::SchemeId;

lazy_static! {
    static ref FD_ALLOCATOR: Spinlock<BitAlloc4K> = {
        let mut alloc = BitAlloc4K::DEFAULT;
        alloc.insert(1..BitAlloc4K::CAP);
        Spinlock::new(alloc)
    };
}

/// A file descriptor
#[derive(Debug, Clone)]
pub struct FileDescriptor {
    /// The file descriptor number
    pub id: FileDescriptorId,
    /// The file descriptor offset, used for seeking
    pub offset: usize,
    /// The scheme that the descriptor belongs to
    pub scheme: SchemeId,
    /// The file descriptor flags
    pub flags: FileDescriptorFlags,
}

impl FileDescriptor {
    /// Create a new file descriptor
    pub fn new(scheme: SchemeId, flags: FileDescriptorFlags) -> Self {
        Self {
            id: FileDescriptorId(FD_ALLOCATOR.lock().alloc().unwrap()),
            offset: 0,
            scheme,
            flags,
        }
    }
}

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        FD_ALLOCATOR.lock().dealloc(self.id.0);
    }
}
