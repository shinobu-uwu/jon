#[repr(C)]
pub struct Message {
    message_type: MessageType,
    data: usize,
}

#[repr(u8)]
pub enum MessageType {
    Read,
    Write,
    Delete,
}
