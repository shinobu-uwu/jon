pub mod operations;

use bitflags::bitflags;

/// A file descriptor
pub struct FileDescriptor {
    /// The file descriptor number
    pub id: FileDescritorId,
    /// The file descriptor offset, used for seeking
    pub offset: usize,
    /// The file descriptor flags
    pub flags: FileDescriptorFlags,
}

pub struct FileDescritorId(usize);

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
