mod pipe;
mod proc;
pub mod ps2;
mod schemes;
mod serial;
pub mod vga;

use crate::sched::pid::Pid;
use alloc::{boxed::Box, sync::Arc};
use hashbrown::HashMap;
use lazy_static::lazy_static;
use libjon::fd::{FileDescriptorFlags, FileDescriptorId};
use log::debug;
use spinning_top::{
    lock_api::{RwLockReadGuard, RwLockWriteGuard},
    RawRwSpinlock, RwSpinlock,
};
use vga::VgaScheme;

lazy_static! {
    pub static ref SCHEMES: RwSpinlock<SchemeList> = {
        debug!("Adding kernel schemes");
        let mut list = SchemeList::new();
        debug!("Adding VGA scheme");
        let vga = VgaScheme::new();
        list.add("vga", Arc::new(vga));
        debug!("Adding pipe scheme");
        list.add("pipe", Arc::new(pipe::PipeScheme));
        debug!("Adding serial scheme");
        list.add("serial", Arc::new(serial::SerialScheme));
        debug!("Adding ps2 scheme");
        ps2::Ps2Scheme::init().unwrap();
        list.add("ps2", Arc::new(ps2::Ps2Scheme));
        debug!("Adding proc scheme");
        list.add("proc", Arc::new(proc::ProcScheme));
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

impl CallerContext {
    pub fn new(pid: Pid, scheme: SchemeId) -> Self {
        Self { pid, scheme }
    }
}

pub trait KernelScheme: Send + Sync + 'static {
    fn open(
        &self,
        path: &str,
        flags: FileDescriptorFlags,
        ctx: CallerContext,
    ) -> Result<FileDescriptorId, i32>;

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

    fn lseek(
        &self,
        descriptor_id: FileDescriptorId,
        offset: usize,
        whence: Whence,
        ctx: CallerContext,
    ) -> Result<usize, i32> {
        Err(38)
    }
}

#[repr(i32)]
pub enum Whence {
    Set = 0,
    Current = 1,
}

impl From<usize> for Whence {
    fn from(value: usize) -> Self {
        match value {
            0 => Whence::Set,
            1 => Whence::Current,
            _ => panic!("Invalid whence value"),
        }
    }
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
