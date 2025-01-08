use core::arch::asm;

use bitmap_allocator::BitAlloc;
use log::debug;
use x86_64::registers::segmentation::{Segment, CS, SS};

use crate::{memory::address::VirtualAddress, sched::stack::KernelStack};

use super::{pid::Pid, PID_ALLOCATOR};

#[derive(Debug)]
/// A task is a unit of execution, it has a PID, a context (registers), and a stack
pub struct Task {
    /// The PID of the task
    pub pid: Pid,
    /// The CPU context of the task
    pub context: Context,
    /// The stack of the task
    pub stack: KernelStack,
}

#[derive(Debug)]
pub enum TaskState {
    Running,
    Sleeping,
    Stopped,
    Zombie,
}

pub const STACK_START: usize = 0xffff888000000000;
pub const STACK_SIZE: usize = 0x4000;

impl Task {
    pub fn new(entry_point: extern "C" fn()) -> Self {
        let pid = Pid::new(PID_ALLOCATOR.lock().alloc().unwrap());
        let mut context = Context::default();
        let stack_base = VirtualAddress::new(STACK_START + pid.as_usize() * STACK_SIZE);
        let stack = KernelStack::new(stack_base, STACK_SIZE);

        context.rip = entry_point as u64;
        context.rsp = stack.top.as_u64();

        Self {
            pid,
            context,
            stack,
        }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        PID_ALLOCATOR.lock().dealloc(self.pid.as_usize());
    }
}

#[repr(C)]
#[derive(Debug, Default)]
// TODO make it generic, for now this is x86 only
pub struct Context {
    pub rsp: u64,
    pub rip: u64,
}

impl Context {
    pub unsafe fn save(&mut self) {
        todo!();
    }

    pub unsafe fn restore(&self) {
        debug!("Restoring context {:#x?}", self);
        asm!(
            "mov rsp, {}",
            "jmp {}",
            in(reg) self.rsp,
            in(reg) self.rip,
        );
    }
}
