use core::{fmt::Display, usize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(usize);

static mut NEXT_PID: usize = 1;

impl Pid {
    pub const fn new(pid: usize) -> Self {
        Self(pid)
    }

    pub fn next_pid() -> usize {
        unsafe {
            let pid = NEXT_PID;
            NEXT_PID += 1;
            pid
        }
    }

    pub const fn is_root(&self) -> bool {
        self.0 == 0
    }

    pub const fn as_64(&self) -> u64 {
        self.0 as u64
    }

    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub const MAX: Pid = Pid::new(usize::MAX);
}

impl Display for Pid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
