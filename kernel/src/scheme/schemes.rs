use alloc::{boxed::Box, collections::btree_map::BTreeMap};

use crate::sched::pid::Pid;

use super::KernelScheme;

static USER_SCHEME_NAMES: BTreeMap<Box<str>, Pid> = BTreeMap::new();

pub struct SchemesScheme;

impl KernelScheme for SchemesScheme {
    fn open(
        &self,
        path: &str,
        flags: libjon::fd::FileDescriptorFlags,
        ctx: super::CallerContext,
    ) -> Result<libjon::fd::FileDescriptorId, i32> {
        todo!()
    }

    fn read(
        &self,
        descriptor_id: libjon::fd::FileDescriptorId,
        buf: &mut [u8],
        count: usize,
    ) -> Result<usize, i32> {
        todo!()
    }

    fn write(
        &self,
        descriptor_id: libjon::fd::FileDescriptorId,
        buf: &[u8],
        count: usize,
    ) -> Result<usize, i32> {
        todo!()
    }

    fn close(
        &self,
        descriptor_id: libjon::fd::FileDescriptorId,
        ctx: super::CallerContext,
    ) -> Result<(), i32> {
        todo!()
    }
}
