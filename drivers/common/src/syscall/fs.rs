use crate::syscall;

pub fn open(path: &str, flags: usize) -> usize {
    unsafe { syscall(56, path.as_ptr() as usize, path.len(), flags, 0, 0, 0) }
}

pub fn write(fd: usize, buf: &[u8]) -> usize {
    unsafe { syscall(64, fd, buf.as_ptr() as usize, buf.len(), 0, 0, 0) }
}

pub fn read(fd: usize, buf: &[u8]) -> usize {
    unsafe { syscall(63, fd, buf.as_ptr() as usize, buf.len(), 0, 0, 0) }
}
