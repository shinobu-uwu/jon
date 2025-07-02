use alloc::{
    boxed::Box,
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    format,
    vec::Vec,
};
use libjon::{
    errno::{EAGAIN, EBADF, EINVAL, ENOENT},
    fd::{FileDescriptorFlags, FileDescriptorId},
};
use log::debug;
use spinning_top::{RwSpinlock, Spinlock};

use crate::sched::{fd::FileDescriptor, scheduler::get_task_mut};

use super::{CallerContext, KernelScheme};

static NEXT_PIPE_ID: Spinlock<u32> = Spinlock::new(1);
static PIPES: RwSpinlock<BTreeMap<PipeId, Pipe>> = RwSpinlock::new(BTreeMap::new());
pub static FDS: RwSpinlock<BTreeMap<FileDescriptorId, PipeId>> = RwSpinlock::new(BTreeMap::new());
static PATHS: RwSpinlock<BTreeMap<Box<str>, PipeId>> = RwSpinlock::new(BTreeMap::new());

pub struct PipeScheme;

impl PipeScheme {
    fn with_pipe_mut<F>(&self, descriptor_id: FileDescriptorId, f: F) -> Result<usize, i32>
    where
        F: FnOnce(&mut Pipe) -> Result<usize, i32>,
    {
        let fds = FDS.read();
        let pipe_id = *fds.get(&descriptor_id).ok_or(EBADF)?;
        drop(fds);

        let mut pipes = PIPES.write();
        let pipe = pipes.get_mut(&pipe_id).ok_or(EINVAL)?;
        f(pipe)
    }
}

impl KernelScheme for PipeScheme {
    fn open(
        &self,
        path: &str,
        flags: FileDescriptorFlags,
        ctx: CallerContext,
    ) -> Result<FileDescriptorId, i32> {
        debug!("Opening pipe: {}", path);
        let task = get_task_mut(ctx.pid).ok_or(ENOENT)?;
        debug!("Found task: {}", task.pid);

        let is_read = flags.contains(FileDescriptorFlags::O_RDONLY)
            || flags.contains(FileDescriptorFlags::O_RDWR);
        let is_write = flags.contains(FileDescriptorFlags::O_WRONLY)
            || flags.contains(FileDescriptorFlags::O_RDWR);

        let mut fds = FDS.write();
        let mut pipes = PIPES.write();
        let mut paths = PATHS.write();

        let fd = match paths.get(path.into()) {
            Some(pipe_id) => {
                debug!("Found existing pipe with id {:?}", pipe_id);

                if flags.contains(FileDescriptorFlags::O_CREAT) {
                    return Err(EINVAL);
                }

                let pipe = pipes.get_mut(pipe_id).ok_or(ENOENT)?;
                let descriptor = FileDescriptor::new(ctx.scheme, flags);
                let descriptor_id = descriptor.id;

                if is_read {
                    pipe.readers.push(descriptor_id);
                }
                if is_write {
                    pipe.writers.push(descriptor_id);
                }

                fds.insert(descriptor_id, *pipe_id);
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
                let mut pipe = Pipe::new(id);

                match (is_read, is_write) {
                    (true, true) => {
                        pipe.readers.push(id);
                        pipe.writers.push(id);
                    }
                    (true, false) => {
                        pipe.readers.push(id);
                    }
                    (false, true) => {
                        pipe.writers.push(id);
                    }
                    (false, false) => {
                        return Err(EINVAL);
                    }
                }

                let pipe_id = pipe.id;
                pipes.insert(pipe.id, pipe);
                task.add_file(descriptor);

                debug!("Inserting path: {} -> {:?}", path, id);
                let real_path = format!("{}/{}", task.pid, path);
                paths.insert(real_path.into(), pipe_id);
                fds.insert(id, pipe_id);

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
        self.with_pipe_mut(descriptor_id, |pipe| {
            let message = pipe.buffer.pop_front().ok_or(EAGAIN)?;
            let bytes_to_read = count.min(message.len());
            buf[..bytes_to_read].copy_from_slice(&message[..bytes_to_read]);

            Ok(bytes_to_read)
        })
    }

    fn write(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &[u8],
        count: usize,
    ) -> Result<usize, i32> {
        self.with_pipe_mut(descriptor_id, |pipe| {
            let bytes_to_write = count.min(buf.len());
            let message = Vec::from(&buf[..bytes_to_write]);
            pipe.buffer.push_back(message);

            Ok(bytes_to_write)
        })
    }

    fn close(&self, descriptor_id: FileDescriptorId, _ctx: CallerContext) -> Result<(), i32> {
        let mut close_queue = VecDeque::new();
        close_queue.push_back(descriptor_id);

        while let Some(current_fd) = close_queue.pop_front() {
            let pipe_id = {
                let mut fds = FDS.write();
                fds.remove(&current_fd)
            };

            let pipe_id = match pipe_id {
                Some(id) => id,
                None => continue,
            };

            let (is_root, other_fds) = {
                let mut pipes = PIPES.write();
                if let Some(pipe) = pipes.get_mut(&pipe_id) {
                    if pipe.root == current_fd {
                        let others = pipe
                            .readers
                            .iter()
                            .chain(pipe.writers.iter())
                            .filter(|&&fd| fd != current_fd)
                            .cloned()
                            .collect();
                        (true, others)
                    } else {
                        pipe.readers.retain(|&fd| fd != current_fd);
                        pipe.writers.retain(|&fd| fd != current_fd);
                        (false, Vec::new())
                    }
                } else {
                    continue;
                }
            };

            if is_root {
                let path_to_remove = {
                    let paths = PATHS.read();
                    paths
                        .iter()
                        .find(|(_, id)| **id == pipe_id)
                        .map(|(p, _)| p.clone())
                };

                if let Some(path) = path_to_remove {
                    let mut paths = PATHS.write();
                    paths.remove(&path);
                    debug!("Removed path: {}", path);
                }

                {
                    let mut pipes = PIPES.write();
                    pipes.remove(&pipe_id);
                    debug!("Removed pipe: {:?}", pipe_id);
                }

                {
                    let mut fds = FDS.write();
                    for fd in &other_fds {
                        fds.remove(fd);
                    }
                }

                close_queue.extend(other_fds);
            }
        }

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
    pub root: FileDescriptorId,
    pub buffer: VecDeque<Vec<u8>>,
    readers: Vec<FileDescriptorId>,
    writers: Vec<FileDescriptorId>,
}

impl Pipe {
    pub fn new(root: FileDescriptorId) -> Self {
        Self {
            id: PipeId::new(),
            root,
            buffer: VecDeque::new(),
            readers: Vec::new(),
            writers: Vec::new(),
        }
    }
}
