use core::ptr::copy_nonoverlapping;

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use libjon::{
    errno::{EINVAL, ENOENT},
    fd::{FileDescriptorFlags, FileDescriptorId},
};
use limine::request::FramebufferRequest;
use log::{debug, info};
use spinning_top::RwSpinlock;

use crate::sched::{fd::FileDescriptor, scheduler::get_task_mut};

use super::{CallerContext, KernelScheme};

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static DESCRIPTORS: RwSpinlock<BTreeMap<FileDescriptorId, FramebufferIndex>> =
    RwSpinlock::new(BTreeMap::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FramebufferIndex(usize);

#[derive(Debug)]
pub struct VgaScheme {
    pub framebuffers: Arc<RwSpinlock<Vec<Framebuffer>>>,
}

#[derive(Debug)]
pub struct Framebuffer {
    pub width: u64,
    pub height: u64,
    pub bpp: u16,
    pub inner: &'static mut [u8],
}

impl VgaScheme {
    pub fn new() -> Self {
        let res = FRAMEBUFFER_REQUEST.get_response().unwrap();
        let fbs: Vec<Framebuffer> = res
            .framebuffers()
            .into_iter()
            .map(|fb| unsafe {
                let inner = core::slice::from_raw_parts_mut(
                    fb.addr() as *mut u8,
                    (fb.width() * fb.height() * fb.bpp() as u64 / 8) as usize,
                );
                Framebuffer {
                    width: fb.width(),
                    height: fb.height(),
                    bpp: fb.bpp(),
                    inner,
                }
            })
            .collect();
        Self {
            framebuffers: Arc::new(RwSpinlock::new(fbs)),
        }
    }
}

impl KernelScheme for VgaScheme {
    fn open(
        &self,
        path: &str,
        _flags: FileDescriptorFlags,
        ctx: CallerContext,
    ) -> Result<FileDescriptorId, i32> {
        info!("Opening framebuffer: {}", path);
        let index: usize = path.parse().map_err(|_| EINVAL)?;
        let task = get_task_mut(ctx.pid).ok_or(EINVAL)?;
        self.framebuffers.clone().read().get(index).ok_or(ENOENT)?;

        let descriptor = FileDescriptor::new(ctx.scheme, FileDescriptorFlags::O_RDWR);
        let id = descriptor.id;
        task.add_file(descriptor);
        DESCRIPTORS.write().insert(id, FramebufferIndex(index));

        Ok(id)
    }

    fn read(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &mut [u8],
        count: usize,
    ) -> Result<usize, i32> {
        let descriptors = DESCRIPTORS.read();
        let framebuffer_index = descriptors.get(&descriptor_id).ok_or(EINVAL)?;

        let framebuffers = self.framebuffers.read();
        let framebuffer = framebuffers.get(framebuffer_index.0).ok_or(ENOENT)?;

        let framebuffer_size = framebuffer.inner.len();
        let bytes_to_read = count.min(framebuffer_size);

        unsafe {
            copy_nonoverlapping(framebuffer.inner.as_ptr(), buf.as_mut_ptr(), bytes_to_read);
        }

        Ok(bytes_to_read)
    }

    fn write(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &[u8],
        count: usize,
    ) -> Result<usize, i32> {
        let descriptors = DESCRIPTORS.read();
        let framebuffer_index = descriptors.get(&descriptor_id).ok_or(EINVAL)?;

        let mut framebuffers = self.framebuffers.write();
        let framebuffer = framebuffers.get_mut(framebuffer_index.0).ok_or(ENOENT)?;

        let framebuffer_size = framebuffer.inner.len();
        let bytes_to_write = count.min(framebuffer_size);

        unsafe {
            copy_nonoverlapping(buf.as_ptr(), framebuffer.inner.as_mut_ptr(), bytes_to_write);
        }

        Ok(bytes_to_write)
    }

    fn close(&self, descriptor_id: FileDescriptorId, ctx: CallerContext) -> Result<(), i32> {
        debug!("Closing fd: {:?}", descriptor_id);
        let task = get_task_mut(ctx.pid).ok_or(EINVAL)?;
        task.remove_file(descriptor_id);
        DESCRIPTORS.write().remove(&descriptor_id);

        Ok(())
    }
}
