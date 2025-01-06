use core::arch::asm;

use bitmap_allocator::BitAlloc;
use x86_64::registers::segmentation::{Segment, CS, SS};

use crate::{memory::address::VirtualAddress, sched::stack::KernelStack};

use super::{pid::Pid, PID_ALLOCATOR};

#[derive(Debug)]
pub struct Task {
    pub pid: Pid,
    pub context: Context,
    stack: KernelStack,
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
        let stack_start = VirtualAddress::new(STACK_START + pid.as_usize() * STACK_SIZE);
        let stack = KernelStack::new(stack_start, STACK_SIZE);

        context.rip = entry_point as u64;
        // stacks grow from top to bottom
        context.rsp = stack.base.as_u64() + stack.size as u64;
        context.cs = CS::get_reg().0 as u64;
        context.ss = SS::get_reg().0 as u64;
        context.rflags = 0x202;

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
    // Callee-saved registers
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // Essential for task switching
    pub rsp: u64,
    pub rip: u64,
    pub rflags: u64,

    // Segment registers (if needed)
    pub cs: u64,
    pub ss: u64,
}

impl Context {
    pub unsafe fn save(&mut self) {
        asm!(
            // Callee-saved registers
            "mov [{0} + 0], rbx",   // offset 0: save rbx
            "mov [{0} + 8], rbp",   // offset 8: save rbp
            "mov [{0} + 16], r12",  // offset 16: save r12
            "mov [{0} + 24], r13",  // offset 24: save r13
            "mov [{0} + 32], r14",  // offset 32: save r14
            "mov [{0} + 40], r15",  // offset 40: save r15

            // Stack pointer
            "mov [{0} + 48], rsp",  // offset 48: save stack pointer

            // Flags
            "pushfq",               // push flags onto stack
            "pop rax",
            "mov [{0} + 56], rax",  // offset 56: save flags

            // Instruction pointer
            "lea rax, [rip+0]",     // get current instruction pointer
            "mov [{0} + 64], rax",  // offset 64: save instruction pointer

            in(reg) self as *mut Context
        );
    }

    pub unsafe fn restore(&self) {
        asm!(
            // Callee-saved registers
            "mov rbx, [{0} + 0]",   // restore rbx
            "mov rbp, [{0} + 8]",   // restore rbp
            "mov r12, [{0} + 16]",  // restore r12
            "mov r13, [{0} + 24]",  // restore r13
            "mov r14, [{0} + 32]",  // restore r14
            "mov r15, [{0} + 40]",  // restore r15

            // Flags
            "push [{0} + 56]",      // push saved flags
            "popfq",                // restore flags

            // Stack pointer and jump to saved instruction
            "mov rsp, [{0} + 48]",  // restore stack pointer
            "jmp [{0} + 64]",       // jump to saved instruction pointer

            in(reg) self as *const Context
        );
    }
}
