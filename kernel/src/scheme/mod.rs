use crate::sched::{fd::FileDescritorId, pid::Pid};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemeId(usize);

#[derive(Debug, Clone)]
pub struct CallerContext {
    pub pid: Pid,
}

pub trait KernelScheme: Send + Sync + 'static {
    fn open(&self, path: &str, flags: usize, ctx: CallerContext);
    fn read(&self, descriptor_id: FileDescritorId, buf: &mut [u8], count: usize);
    fn write(&self, descriptor_id: FileDescritorId, buf: &[u8], count: usize);
}
