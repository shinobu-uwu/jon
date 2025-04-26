use libjon::{
    errno::{EAGAIN, EINVAL, EIO},
    fd::FileDescriptorId,
};
use ps2::{error::ControllerError, flags::ControllerConfigFlags, Controller};
use spinning_top::Spinlock;

use crate::sched::{fd::FileDescriptor, scheduler::get_task_mut};

use super::{CallerContext, KernelScheme};

static CONTROLLER: Spinlock<Controller> = Spinlock::new(unsafe { Controller::new() });

pub struct Ps2Scheme;

impl Ps2Scheme {
    pub fn init() -> Result<(), ControllerError> {
        let mut controller = CONTROLLER.lock();

        controller.disable_keyboard()?;
        controller.disable_mouse()?;

        let _ = controller.read_data();

        let mut config = controller.read_config()?;
        config.set(
            ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT
                | ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT
                | ControllerConfigFlags::ENABLE_TRANSLATE,
            false,
        );
        controller.write_config(config)?;

        controller.test_controller()?;
        controller.write_config(config)?;

        controller.disable_mouse()?;

        let keyboard_works = controller.test_keyboard().is_ok();

        config = controller.read_config()?;

        if keyboard_works {
            controller.enable_keyboard()?;
            config.set(ControllerConfigFlags::DISABLE_KEYBOARD, false);
            config.set(ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT, true);
            controller.keyboard().reset_and_self_test().unwrap();
        }

        controller.write_config(config)?;

        Ok(())
    }
}

impl KernelScheme for Ps2Scheme {
    fn open(
        &self,
        path: &str,
        flags: libjon::fd::FileDescriptorFlags,
        ctx: CallerContext,
    ) -> Result<FileDescriptorId, i32> {
        let task = match get_task_mut(ctx.pid) {
            Some(task) => task,
            None => return Err(libjon::errno::ENOENT),
        };
        let descriptor = FileDescriptor::new(ctx.scheme, flags);
        let id = descriptor.id;
        task.add_file(descriptor);

        Ok(id)
    }

    fn read(
        &self,
        _descriptor_id: FileDescriptorId,
        buf: &mut [u8],
        _count: usize,
    ) -> Result<usize, i32> {
        let mut controller = CONTROLLER.lock();
        let data = controller.read_data().map_err(|e| {
            if let ControllerError::Timeout = e {
                EAGAIN
            } else {
                EIO
            }
        })?;
        buf[0] = data;

        Ok(1)
    }

    fn write(
        &self,
        descriptor_id: FileDescriptorId,
        buf: &[u8],
        count: usize,
    ) -> Result<usize, i32> {
        Err(EINVAL)
    }

    fn close(&self, descriptor_id: FileDescriptorId, ctx: CallerContext) -> Result<(), i32> {
        let task = match get_task_mut(ctx.pid) {
            Some(task) => task,
            None => return Err(libjon::errno::ENOENT),
        };
        task.remove_file(descriptor_id);
        Ok(())
    }
}
