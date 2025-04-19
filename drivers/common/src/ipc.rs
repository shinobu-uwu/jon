use crate::syscall::task::getpid;

#[repr(C)]
#[derive(Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub data: [u8; 16],
    pub origin: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Read,
    Write,
    Delete,
    Heartbeat,
}

impl Message {
    pub fn new(message_type: MessageType, data: [u8; 16]) -> Self {
        let origin = getpid().unwrap_or(0);

        Self {
            message_type,
            data,
            origin,
        }
    }

    pub fn from_bytes(buf: &[u8]) -> Self {
        unsafe { core::ptr::read(buf.as_ptr() as *const Message) }
    }

    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Message as *const u8,
                core::mem::size_of::<Message>(),
            )
        }
    }
}
