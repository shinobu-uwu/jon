use crate::syscall;

pub fn getpid() -> Result<usize, i32> {
    syscall(39, 0, 0, 0, 0, 0, 0)
}
