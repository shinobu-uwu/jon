use crate::sched::{
    fd::FileDescriptor,
    pid::Pid,
    scheduler::{get_task, get_task_mut, get_tasks},
    task::{Priority, State},
};

use super::KernelScheme;
use alloc::collections::btree_map::BTreeMap;
use lazy_static::lazy_static;
use libjon::fd::FileDescriptorId;
use spinning_top::RwSpinlock;

lazy_static! {
    static ref HANDLES: RwSpinlock<BTreeMap<FileDescriptorId, usize>> =
        RwSpinlock::new(BTreeMap::new());
}

#[repr(C)]
pub struct Proc {
    pub pid: usize,
    pub name: [u8; 16],
    pub state: State,
    pub priority: Priority,
}

impl Proc {
    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Proc as *const u8,
                core::mem::size_of::<Proc>(),
            )
        }
    }
}

pub struct ProcScheme;

impl KernelScheme for ProcScheme {
    fn open(
        &self,
        path: &str,
        flags: libjon::fd::FileDescriptorFlags,
        ctx: super::CallerContext,
    ) -> Result<FileDescriptorId, i32> {
        let task = match get_task_mut(ctx.pid) {
            Some(task) => task,
            None => return Err(libjon::errno::ENOENT),
        };
        let descriptor = FileDescriptor::new(ctx.scheme, flags);
        let id = descriptor.id;
        let pid: usize = if path == "" {
            0
        } else {
            path.parse().map_err(|_| libjon::errno::EINVAL)?
        };
        HANDLES.write().insert(descriptor.id, pid);
        task.add_file(descriptor);

        Ok(id)
    }

    fn read(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &mut [u8],
        count: usize,
    ) -> Result<usize, i32> {
        let handles = HANDLES.read();
        let pid = handles.get(&descriptor_id).ok_or(libjon::errno::ENOENT)?;

        let mut offset = 0;

        if *pid == 0 {
            let tasks = get_tasks();

            for task in tasks.iter() {
                let mut name_buf = [0u8; 16];
                let len = task.name.len().min(15);
                name_buf[..len].copy_from_slice(task.name.as_bytes());
                name_buf[len] = 0; // null terminator
                let proc = Proc {
                    pid: task.pid.as_usize(),
                    name: name_buf,
                    state: task.state,
                    priority: task.priority,
                };
                let bytes = proc.to_bytes();

                if offset + bytes.len() > count || offset + bytes.len() > buf.len() {
                    break;
                }
                buf[offset..offset + bytes.len()].copy_from_slice(bytes);
                offset += bytes.len();
            }

            return Ok(offset);
        }

        match get_task(Pid::new(*pid)) {
            Some(task) => {
                let mut name_buf = [0u8; 16];
                let len = task.name.len().min(15);
                name_buf[..len].copy_from_slice(task.name.as_bytes());
                name_buf[len] = 0; // null terminator
                let proc = Proc {
                    pid: task.pid.as_usize(),
                    name: name_buf,
                    state: task.state,
                    priority: task.priority,
                };
                let bytes = proc.to_bytes();

                if offset + bytes.len() > count || offset + bytes.len() > buf.len() {
                    return Ok(offset);
                }

                buf[offset..offset + bytes.len()].copy_from_slice(bytes);
                offset += bytes.len();

                Ok(offset)
            }
            None => Err(libjon::errno::ENOENT),
        }
    }

    fn write(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &[u8],
        count: usize,
    ) -> Result<usize, i32> {
        todo!()
    }

    fn close(&self, descriptor_id: FileDescriptorId, ctx: super::CallerContext) -> Result<(), i32> {
        let mut handles = HANDLES.write();
        handles
            .remove(&descriptor_id)
            .ok_or(libjon::errno::ENOENT)?;
        let task = get_task_mut(ctx.pid).ok_or(libjon::errno::ENOENT)?;
        task.remove_file(descriptor_id);

        Ok(())
    }
}
