use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDescriptorId(pub usize);

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct FileDescriptorFlags: usize {
        const O_RDONLY = 0x1;
        const O_WRONLY = 0x2;
        const O_RDWR = Self::O_RDONLY.bits() | Self::O_WRONLY.bits();
        const O_APPEND = 0x8;
        const O_CREAT = 0o100;
        const O_EXCL = 0x400;
        const O_TRUNC = 0x2000;
    }
}
