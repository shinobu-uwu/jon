use crate::syscall;

pub fn getpid() -> Result<usize, i32> {
    syscall(39, 0, 0, 0, 0, 0, 0)
}

pub fn brk(increment: usize) -> Result<usize, i32> {
    syscall(12, increment, 0, 0, 0, 0, 0)
}

pub fn kill(pid: usize) -> Result<usize, i32> {
    syscall(62, pid, 0, 0, 0, 0, 0)
}
