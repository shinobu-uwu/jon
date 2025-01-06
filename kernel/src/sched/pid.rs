use core::{fmt::Display, usize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(usize);

impl Pid {
    pub const fn new(pid: usize) -> Self {
        Self(pid)
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
        write!(f, "PID: {}", self.0)
    }
}
