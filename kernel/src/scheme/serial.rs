use alloc::collections::btree_set::BTreeSet;
use libjon::{
    errno::EINVAL,
    fd::{FileDescriptorFlags, FileDescriptorId},
};
use log::{debug, info};
use spinning_top::RwSpinlock;

use crate::sched::{fd::FileDescriptor, scheduler::get_task_mut};

use super::{CallerContext, KernelScheme};

static DESCRIPTORS: RwSpinlock<BTreeSet<FileDescriptorId>> = RwSpinlock::new(BTreeSet::new());

#[derive(Debug)]
pub struct SerialScheme;

impl KernelScheme for SerialScheme {
    fn open(
        &self,
        _path: &str,
        _flags: FileDescriptorFlags,
        ctx: CallerContext,
    ) -> Result<FileDescriptorId, i32> {
        let task = get_task_mut(ctx.pid).ok_or(EINVAL)?;
        let descriptor = FileDescriptor::new(ctx.scheme, FileDescriptorFlags::O_RDWR);
        let id = descriptor.id;
        DESCRIPTORS.write().insert(id);
        task.add_file(descriptor);

        Ok(id)
    }

    fn read(
        &self,
        _descriptor_id: FileDescriptorId,
        _buf: &mut [u8],
        _count: usize,
    ) -> Result<usize, i32> {
        Err(-38)
    }
    fn write(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &[u8],
        _count: usize,
    ) -> Result<usize, i32> {
        debug!("Writing to fd: {:?}", descriptor_id);
        let descriptors = DESCRIPTORS.read();
        descriptors.get(&descriptor_id).ok_or(EINVAL)?;

        let str = unsafe { core::str::from_utf8_unchecked(buf) };
        info!("{str}");

        Ok(buf.len())
    }

    fn close(&self, descriptor_id: FileDescriptorId, ctx: CallerContext) -> Result<(), i32> {
        debug!("Closing fd: {:?}", descriptor_id);
        let task = get_task_mut(ctx.pid).ok_or(EINVAL)?;
        task.remove_file(descriptor_id);
        DESCRIPTORS.write().remove(&descriptor_id);

        Ok(())
    }
}
