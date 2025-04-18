use crate::syscall::task::getpid;

#[repr(C)]
pub struct Message {
    pub message_type: MessageType,
    pub data: usize,
    pub origin: usize,
}

#[repr(u8)]
pub enum MessageType {
    Read,
    Write,
    Delete,
    Heartbeat,
}

impl From<MessageType> for u8 {
    fn from(message_type: MessageType) -> Self {
        match message_type {
            MessageType::Read => 0,
            MessageType::Write => 1,
            MessageType::Delete => 2,
            MessageType::Heartbeat => 3,
        }
    }
}

impl Message {
    pub fn new(message_type: MessageType, data: usize) -> Self {
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
}
