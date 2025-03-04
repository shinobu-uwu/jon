use crate::syscall;

pub fn open(path: &str) -> usize {
    unsafe { syscall(2, path.as_ptr() as usize, path.len(), 0, 0, 0, 0) }
}
