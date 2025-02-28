use core::{
    ptr::copy_nonoverlapping,
    sync::atomic::{self, AtomicUsize},
};

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use libc::{EINVAL, ENOENT};
use limine::{framebuffer::Framebuffer, request::FramebufferRequest};
use spinning_top::RwSpinlock;

use crate::sched::{
    fd::{FileDescriptorFlags, FileDescriptorId},
    scheduler::get_task_mut,
};

use super::{CallerContext, KernelScheme};

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static DESCRIPTORS: RwSpinlock<BTreeMap<FileDescriptorId, FramebufferIndex>> =
    RwSpinlock::new(BTreeMap::new());
static NEXT_FD: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FramebufferIndex(usize);

pub struct VgaScheme {
    pub framebuffers: Arc<RwSpinlock<Vec<Framebuffer<'static>>>>,
}

impl VgaScheme {
    pub fn new() -> Self {
        let res = FRAMEBUFFER_REQUEST.get_response().unwrap();
        let fbs = res.framebuffers().into_iter().collect();
        Self {
            framebuffers: Arc::new(RwSpinlock::new(fbs)),
        }
    }
}

impl KernelScheme for VgaScheme {
    fn open(&self, path: &str, _flags: usize, ctx: CallerContext) -> Result<FileDescriptorId, i32> {
        let n = &path[2..];
        let index: usize = n.parse().map_err(|_| EINVAL)?;
        let task = get_task_mut(ctx.pid).ok_or(EINVAL)?;
        self.framebuffers.clone().read().get(index).ok_or(ENOENT)?;

        let id = FileDescriptorId(NEXT_FD.fetch_add(1, atomic::Ordering::Relaxed));
        task.add_file(crate::sched::fd::FileDescriptor {
            id,
            offset: 0,
            scheme: ctx.scheme,
            flags: FileDescriptorFlags::O_RDWR,
        });
        DESCRIPTORS.write().insert(id, FramebufferIndex(index));

        Ok(id)
    }

    fn read(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &mut [u8],
        count: usize,
    ) -> Result<usize, i32> {
        todo!()
    }

    fn write(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &[u8],
        count: usize,
    ) -> Result<usize, i32> {
        // Fetch the framebuffer index using the descriptor
        let descriptors = DESCRIPTORS.read();
        let framebuffer_index = descriptors.get(&descriptor_id).ok_or(EINVAL)?;

        // Get the framebuffer from the framebuffers list
        let framebuffers = self.framebuffers.read();
        let framebuffer = framebuffers.get(framebuffer_index.0).ok_or(ENOENT)?;

        // Write the buffer to the framebuffer
        let framebuffer_size = framebuffer.len();
        let bytes_to_write = count.min(framebuffer_size); // Limit the number of bytes to write

        unsafe {
            copy_nonoverlapping(buf.as_ptr(), framebuffer.as_mut_ptr(), bytes_to_write);
        }

        // Return the number of bytes written
        Ok(bytes_to_write)
    }

    fn close(&self, descriptor_id: FileDescriptorId) -> Result<(), i32> {
        todo!()
    }
}
