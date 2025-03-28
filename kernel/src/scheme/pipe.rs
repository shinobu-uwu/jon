use alloc::{
    boxed::Box,
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    vec::Vec,
};
use libjon::{
    errno::ENOENT,
    fd::{FileDescriptorFlags, FileDescriptorId},
};
use log::debug;
use spinning_top::RwSpinlock;

use crate::sched::{fd::FileDescriptor, scheduler::get_task_mut};

use super::KernelScheme;

static PIPES: RwSpinlock<BTreeMap<FileDescriptorId, Pipe>> = RwSpinlock::new(BTreeMap::new());
static PATHS: RwSpinlock<BTreeMap<Box<str>, FileDescriptorId>> = RwSpinlock::new(BTreeMap::new());

pub struct PipeScheme;

impl KernelScheme for PipeScheme {
    fn open(
        &self,
        path: &str,
        _flags: usize,
        ctx: super::CallerContext,
    ) -> Result<FileDescriptorId, i32> {
        debug!("Opening  pipe: {}", path,);
        let task = get_task_mut(ctx.pid).ok_or(ENOENT)?;
        debug!("Found task: {}", task.pid);
        let mut paths = PATHS.write();

        let fd = match paths.get(path.into()) {
            Some(fd) => {
                debug!("Found existing pipe: {:?}", fd);

                if !task.fds.iter().any(|f| f.id == *fd) {
                    debug!("Adding existing pipe FD to task");
                    let descriptor = FileDescriptor {
                        id: *fd,
                        offset: 0,
                        scheme: ctx.scheme,
                        flags: FileDescriptorFlags::O_RDWR,
                    };
                    task.add_file(descriptor);
                }

                *fd
            }
            None => {
                debug!("Creating new pipe");
                let descriptor = FileDescriptor::new(ctx.scheme, FileDescriptorFlags::O_RDWR);
                let id = descriptor.id;
                debug!("Inserting pipe: {:?}", id);
                PIPES.write().insert(id, Pipe::new());
                task.add_file(descriptor);

                debug!("Inserting path: {} -> {:?}", path, id);
                paths.insert(path.into(), id);

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
        let mut pipes = PIPES.write();
        let pipe = pipes.get_mut(&descriptor_id).ok_or(ENOENT)?;

        debug!("Pipe len before pop: {}", pipe.buffer.len());
        let message = pipe.buffer.pop_front().ok_or(0)?;
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
        let pipe = pipes.get_mut(&descriptor_id).ok_or(ENOENT)?;

        debug!("Pipe len before push: {}", pipe.buffer.len());
        pipe.buffer.push_back(Vec::from(buf));
        debug!("Pipe len after push: {}", pipe.buffer.len());

        Ok(count)
    }

    fn close(
        &self,
        _descriptor_id: FileDescriptorId,
        _ctx: super::CallerContext,
    ) -> Result<(), i32> {
        todo!()
    }
}

pub struct Pipe {
    pub buffer: VecDeque<Vec<u8>>,
    readers: usize,
    writers: usize,
}

impl Pipe {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            readers: 0,
            writers: 0,
        }
    }
}
