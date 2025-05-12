use alloc::{
    boxed::Box,
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    format,
    vec::Vec,
};
use libjon::{
    errno::{EAGAIN, EINVAL, ENOENT},
    fd::{FileDescriptorFlags, FileDescriptorId},
};
use log::{debug, info};
use spinning_top::{RwSpinlock, Spinlock};

use crate::sched::{fd::FileDescriptor, pid::Pid, scheduler::get_task_mut};

use super::{CallerContext, KernelScheme};

static NEXT_PIPE_ID: Spinlock<u32> = Spinlock::new(1);
static PIPES: RwSpinlock<BTreeMap<PipeId, Pipe>> = RwSpinlock::new(BTreeMap::new());
static FDS: RwSpinlock<BTreeMap<FileDescriptorId, PipeId>> = RwSpinlock::new(BTreeMap::new());
static PATHS: RwSpinlock<BTreeMap<Box<str>, PipeId>> = RwSpinlock::new(BTreeMap::new());

pub struct PipeScheme;

impl KernelScheme for PipeScheme {
    fn open(
        &self,
        path: &str,
        flags: FileDescriptorFlags,
        ctx: CallerContext,
    ) -> Result<FileDescriptorId, i32> {
        debug!("Opening  pipe: {}", path,);
        let task = get_task_mut(ctx.pid).ok_or(ENOENT)?;
        debug!("Found task: {}", task.pid);

        let mut paths = PATHS.write();

        let is_read = flags.contains(FileDescriptorFlags::O_RDONLY)
            || flags.contains(FileDescriptorFlags::O_RDWR);
        let is_write = flags.contains(FileDescriptorFlags::O_WRONLY)
            || flags.contains(FileDescriptorFlags::O_RDWR);

        let fd = match paths.get(path.into()) {
            Some(pipe_id) => {
                debug!("Found existing pipe with id {:?}", pipe_id);

                if flags.contains(FileDescriptorFlags::O_CREAT) {
                    return Err(EINVAL);
                }

                let mut pipes = PIPES.write();
                let pipe = pipes.get_mut(pipe_id).ok_or(ENOENT)?;

                if is_read {
                    pipe.readers.push(ctx.pid);
                }
                if is_write {
                    pipe.writers.push(ctx.pid);
                }

                let descriptor = FileDescriptor::new(ctx.scheme, flags);
                let descriptor_id = descriptor.id;
                FDS.write().insert(descriptor_id, *pipe_id);
                task.add_file(descriptor);

                descriptor_id
            }
            None => {
                debug!("Creating new pipe");

                if !flags.contains(FileDescriptorFlags::O_CREAT) {
                    return Err(EINVAL);
                }

                let descriptor = FileDescriptor::new(ctx.scheme, flags);
                let id = descriptor.id;
                debug!("Inserting pipe: {:?}", id);
                let mut pipe = Pipe::new();

                match (is_read, is_write) {
                    (true, true) => {
                        pipe.readers.push(ctx.pid);
                        pipe.writers.push(ctx.pid);
                    }
                    (true, false) => {
                        pipe.readers.push(ctx.pid);
                    }
                    (false, true) => {
                        pipe.writers.push(ctx.pid);
                    }
                    (false, false) => {
                        return Err(EINVAL);
                    }
                }

                let pipe_id = pipe.id;
                PIPES.write().insert(pipe.id, pipe);
                task.add_file(descriptor);

                debug!("Inserting path: {} -> {:?}", path, id);
                let real_path = format!("{}/{}", task.pid, path);
                paths.insert(real_path.into(), pipe_id);
                FDS.write().insert(id, pipe_id);

                id
            }
        };

        Ok(fd)
    }

    fn read(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &mut [u8],
        count: usize,
    ) -> Result<usize, i32> {
        debug!("Reading from pipe: {:?}", descriptor_id);
        let mut pipes = PIPES.write();
        let fds = FDS.read();
        let pipe_id = fds.get(&descriptor_id).ok_or(ENOENT)?;
        let pipe = pipes.get_mut(pipe_id).ok_or(ENOENT)?;

        debug!("Pipe len before pop: {}", pipe.buffer.len());
        let message = pipe.buffer.pop_front().ok_or(EAGAIN)?;
        debug!("Pipe len after pop: {}", pipe.buffer.len());

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
        let fds = FDS.read();
        let pipe_id = fds.get(&descriptor_id).ok_or(ENOENT)?;
        let pipe = pipes.get_mut(&pipe_id).ok_or(ENOENT)?;

        debug!("Pipe len before push: {}", pipe.buffer.len());
        pipe.buffer.push_back(Vec::from(buf));
        debug!("Pipe len after push: {}", pipe.buffer.len());

        Ok(count)
    }

    fn close(&self, descriptor_id: FileDescriptorId, _ctx: CallerContext) -> Result<(), i32> {
        let mut fds = FDS.write();
        let mut pipes = PIPES.write();
        let mut paths = PATHS.write();
        let pipe_id = fds.remove(&descriptor_id).ok_or(ENOENT)?;
        pipes.remove(&pipe_id);
        paths.retain(|_, id| id != &pipe_id);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PipeId(u32);

impl PipeId {
    pub fn new() -> Self {
        let id = *NEXT_PIPE_ID.lock();
        *NEXT_PIPE_ID.lock() += 1;

        Self(id)
    }
}

#[derive(Debug)]
pub struct Pipe {
    pub id: PipeId,
    pub buffer: VecDeque<Vec<u8>>,
    readers: Vec<Pid>,
    writers: Vec<Pid>,
}

impl Pipe {
    pub fn new() -> Self {
        Self {
            id: PipeId::new(),
            buffer: VecDeque::new(),
            readers: Vec::new(),
            writers: Vec::new(),
        }
    }
}
