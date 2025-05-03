#[repr(C)]
#[derive(Debug)]
pub struct Proc {
    pub pid: usize,
    pub name: [u8; 16],
    pub state: State,
    pub priority: Priority,
}

impl Proc {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        unsafe { core::ptr::read(bytes.as_ptr() as *const Proc) }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Low,
    Normal,
    High,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running,
    Blocked,
    Waiting,
    Stopped,
}
