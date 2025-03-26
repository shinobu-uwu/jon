pub mod pipe;
pub mod serial;
pub mod vga;

use crate::sched::pid::Pid;
use alloc::{boxed::Box, sync::Arc};
use hashbrown::HashMap;
use lazy_static::lazy_static;
use libjon::fd::FileDescriptorId;
use log::debug;
use spinning_top::{
    lock_api::{RwLockReadGuard, RwLockWriteGuard},
    RawRwSpinlock, RwSpinlock,
};
use vga::VgaScheme;

lazy_static! {
    static ref SCHEMES: RwSpinlock<SchemeList> = {
        debug!("Adding kernel schemes");
        let mut list = SchemeList::new();
        debug!("Adding VGA scheme");
        let vga = VgaScheme::new();
        list.add("vga", Arc::new(vga));
        debug!("Adding pipe scheme");
        list.add("pipe", Arc::new(pipe::PipeScheme));
        let serial = serial::SerialScheme;
        list.add("serial", Arc::new(serial));
        RwSpinlock::new(list)
    };
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub struct SchemeId(usize);

#[derive(Debug, Clone)]
pub struct CallerContext {
    pub pid: Pid,
    pub scheme: SchemeId,
}

pub trait KernelScheme: Send + Sync + 'static {
    fn open(&self, path: &str, flags: usize, ctx: CallerContext) -> Result<FileDescriptorId, i32>;
    fn read(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &mut [u8],
        count: usize,
    ) -> Result<usize, i32>;
    fn write(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &[u8],
        count: usize,
    ) -> Result<usize, i32>;
    fn close(&self, descriptor_id: FileDescriptorId, ctx: CallerContext) -> Result<(), i32>;
}

pub struct SchemeList {
    schemes: HashMap<SchemeId, Arc<dyn KernelScheme>>,
    pub names: HashMap<Box<str>, SchemeId>,
    next_id: usize,
}

impl SchemeList {
    pub fn new() -> Self {
        Self {
            schemes: HashMap::new(),
            names: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn get(&self, id: SchemeId) -> Option<Arc<dyn KernelScheme>> {
        let scheme = self.schemes.get(&id)?;

        Some(Arc::clone(scheme))
    }

    pub fn get_name(&self, name: &str) -> Option<(SchemeId, Arc<dyn KernelScheme>)> {
        let id = self.names.get(name)?;
        let scheme = self.schemes.get(id)?;

        Some((*id, Arc::clone(scheme)))
    }

    pub fn add(&mut self, name: &str, scheme: Arc<dyn KernelScheme>) -> SchemeId {
        let id = SchemeId(self.next_id);
        self.next_id += 1;

        self.schemes.insert(id, scheme);
        self.names.insert(name.into(), id);

        id
    }
}

pub fn schemes() -> RwLockReadGuard<'static, RawRwSpinlock, SchemeList> {
    SCHEMES.read()
}

pub fn schemes_mut() -> RwLockWriteGuard<'static, RawRwSpinlock, SchemeList> {
    SCHEMES.write()
}
