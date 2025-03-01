use libjon::fd::{FileDescriptorFlags, FileDescriptorId};

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
