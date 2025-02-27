use crate::sched::{fd::FileDescritorId, pid::Pid};
use alloc::boxed::Box;
use hashbrown::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SCHEMES: SchemeList = SchemeList::new();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemeId(usize);

#[derive(Debug, Clone)]
pub struct CallerContext {
    pub pid: Pid,
}

pub trait KernelScheme: Send + Sync + 'static {
    fn open(&self, path: &str, flags: usize, ctx: CallerContext) -> Result<FileDescritorId, i32>;
    fn read(
        &self,
        descriptor_id: FileDescritorId,
        buf: &mut [u8],
        count: usize,
    ) -> Result<usize, i32>;
    fn write(&self, descriptor_id: FileDescritorId, buf: &[u8], count: usize)
        -> Result<usize, i32>;
}

pub struct SchemeList {
    schemes: HashMap<SchemeId, Box<dyn KernelScheme>>,
    pub names: HashMap<SchemeId, Box<str>>,
    next_id: usize,
}

impl SchemeList {
    pub fn new() -> Self {
        Self {
            schemes: HashMap::new(),
            names: HashMap::new(),
            next_id: 0,
        }
    }
}
