use core::sync::atomic::AtomicUsize;

use alloc::{
    boxed::Box,
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    format,
    vec::Vec,
};
use libjon::{
    errno::{EAGAIN, ENOENT},
    fd::FileDescriptorId,
};
use log::debug;
use spinning_top::RwSpinlock;

use crate::sched::scheduler::get_task_mut;

use super::KernelScheme;

static NEXT_PIPE_ID: AtomicUsize = AtomicUsize::new(1);
static PIPES: RwSpinlock<BTreeMap<FileDescriptorId, Pipe>> = RwSpinlock::new(BTreeMap::new());
static PATHS: RwSpinlock<BTreeMap<Box<str>, FileDescriptorId>> = RwSpinlock::new(BTreeMap::new());

pub struct PipeScheme;

impl KernelScheme for PipeScheme {
    fn open(
        &self,
        path: &str,
        flags: usize,
        ctx: super::CallerContext,
    ) -> Result<FileDescriptorId, i32> {
        let is_write = flags & 1 == 1;
        debug!(
            "Opening {} pipe: {}",
            path,
            if is_write { "write" } else { "read" }
        );
        let task = get_task_mut(ctx.pid).ok_or(ENOENT)?;
        debug!("Found task: {}", task.pid);
        let mut paths = PATHS.write();

        let fd = match paths.get(path) {
            Some(fd) => {
                debug!("Found existing pipe: {:?}", fd);
                *fd
            }
            None => {
                debug!("Creating new pipe");
                let id = FileDescriptorId(
                    NEXT_PIPE_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed),
                );

                if is_write {
                    debug!("Inserting write pipe: {:?}", id);
                    PIPES.write().insert(id, Pipe::new(PipeEndType::Write));
                } else {
                    debug!("Inserting read pipe: {:?}", id);
                    PIPES.write().insert(id, Pipe::new(PipeEndType::Read));
                }

                debug!("Inserting path: {} -> {:?}", path, id);
                paths.insert(format!("{}/{}", task.pid, path).into(), id);

                id
            }
        };

        task.add_file(crate::sched::fd::FileDescriptor {
            id: fd,
            offset: 0,
            scheme: ctx.scheme,
            flags: libjon::fd::FileDescriptorFlags::O_RDWR,
        });

        Ok(fd)
    }

    fn read(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &mut [u8],
        count: usize,
    ) -> Result<usize, i32> {
        let mut pipes = PIPES.write();
        let pipe = pipes.get_mut(&descriptor_id).ok_or(ENOENT)?;
        let message = pipe.buffer.pop_front().ok_or(EAGAIN)?;

        let bytes_to_read = count.min(message.len());
        buf[..bytes_to_read].copy_from_slice(&message[..bytes_to_read]);

        Ok(bytes_to_read)
    }

    fn write(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &[u8],
        count: usize,
    ) -> Result<usize, i32> {
        let mut pipes = PIPES.write();
        let pipe = pipes.get_mut(&descriptor_id).ok_or(ENOENT)?;
        pipe.buffer.push_back(Vec::from(buf));

        Ok(count)
    }

    fn close(&self, descriptor_id: FileDescriptorId, ctx: super::CallerContext) -> Result<(), i32> {
        todo!()
    }
}

pub struct Pipe {
    pub end_type: PipeEndType,
    pub buffer: VecDeque<Vec<u8>>,
}

pub enum PipeEndType {
    Read,
    Write,
}

impl Pipe {
    pub fn new(end_type: PipeEndType) -> Self {
        Self {
            end_type,
            buffer: VecDeque::new(),
        }
    }

    pub fn is_read(&self) -> bool {
        matches!(self.end_type, PipeEndType::Read)
    }

    pub fn is_write(&self) -> bool {
        matches!(self.end_type, PipeEndType::Write)
    }
}
