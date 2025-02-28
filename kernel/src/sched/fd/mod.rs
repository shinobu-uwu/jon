use bitflags::bitflags;

use crate::scheme::SchemeId;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileDescriptorId(pub usize);

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FileDescriptorFlags: u32 {
        const O_RDONLY = 0x1;
        const O_WRONLY = 0x2;
        const O_RDWR = Self::O_RDONLY.bits() | Self::O_WRONLY.bits();
        const O_APPEND = 0x8;
        const O_CREAT = 0x200;
        const O_EXCL = 0x400;
        const O_TRUNC = 0x2000;
    }
}
