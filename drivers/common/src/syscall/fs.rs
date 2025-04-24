use crate::syscall;

pub fn open(path: &str, flags: usize) -> Result<usize, i32> {
    syscall(56, path.as_ptr() as usize, path.len(), flags, 0, 0, 0)
}

pub fn write(fd: usize, buf: &[u8]) -> Result<usize, i32> {
    syscall(64, fd, buf.as_ptr() as usize, buf.len(), 0, 0, 0)
}

pub fn read(fd: usize, buf: &mut [u8]) -> Result<usize, i32> {
    syscall(63, fd, buf.as_mut_ptr() as usize, buf.len(), 0, 0, 0)
}

pub fn lseek(fd: usize, offset: usize, whence: usize) -> Result<usize, i32> {
    syscall(62, fd, offset, whence, 0, 0, 0)
}
