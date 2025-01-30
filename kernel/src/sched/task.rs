use core::arch::asm;

use bitmap_allocator::BitAlloc;
use log::debug;

use crate::{
    arch::x86::structures::Registers,
    memory::{
        address::VirtualAddress,
        loader::{elf::ElfLoader, Loader},
        stack::Stack,
        PAGE_SIZE,
    },
    sched::{pid::Pid, PID_ALLOCATOR},
};

use super::memory::MemoryDescriptor;

const STACK_START: usize = 0xffff888000000000;
const STACK_SIZE: usize = 0x4000; // 16 KiB

#[derive(Debug)]
pub struct Task {
    pub pid: Pid,
    pub state: State,
    kernel_stack: Stack,
    memory_descriptor: MemoryDescriptor,
    context: Registers,
}

#[derive(Debug)]
pub enum State {
    Running,
    Sleeping,
    Waiting,
    Zombie,
}

impl Task {
    pub fn new(binary: &[u8]) -> Self {
        let pid = Pid::new(PID_ALLOCATOR.lock().alloc().unwrap());
        debug!("Creating task with PID {}", pid);
        let kernel_stack = Stack::new(
            VirtualAddress::new(STACK_START + pid.as_usize() * STACK_SIZE),
            STACK_SIZE,
        );
        let mut context = Registers::new();
        let bin_addr = VirtualAddress::new(0x400000 + (pid.as_usize() - 1) * PAGE_SIZE * 100); // TODO: Use a better dynamic address
        let loader = ElfLoader::new();
        let (memory_descriptor, rip) = loader.load(bin_addr, binary).unwrap();

        context.iret.rsp = kernel_stack.top().as_u64();
        context.iret.rip = rip.as_u64();

        Self {
            pid,
            kernel_stack,
            context,
            state: State::Running,
            memory_descriptor,
        }
    }

    pub unsafe fn save(&mut self, stack_frame: &Registers) {
        self.context.iret.rip = stack_frame.iret.rip;
        self.context.iret.rsp = stack_frame.iret.rsp;
        self.context.iret.rflags = stack_frame.iret.rflags;
        self.context.iret.cs = stack_frame.iret.cs;
        self.context.iret.ss = stack_frame.iret.ss;
        self.context.preserved.r15 = stack_frame.preserved.r15;
        self.context.preserved.r14 = stack_frame.preserved.r14;
        self.context.preserved.r13 = stack_frame.preserved.r13;
        self.context.preserved.r12 = stack_frame.preserved.r12;
        self.context.preserved.rbp = stack_frame.preserved.rbp;
        self.context.preserved.rbx = stack_frame.preserved.rbx;
        self.context.scratch.rax = stack_frame.scratch.rax;
        self.context.scratch.rcx = stack_frame.scratch.rcx;
        self.context.scratch.rdx = stack_frame.scratch.rdx;
        self.context.scratch.rdi = stack_frame.scratch.rdi;
        self.context.scratch.rsi = stack_frame.scratch.rsi;
        self.context.scratch.r8 = stack_frame.scratch.r8;
        self.context.scratch.r9 = stack_frame.scratch.r9;
        self.context.scratch.r10 = stack_frame.scratch.r10;
        self.context.scratch.r11 = stack_frame.scratch.r11;
    }

    pub unsafe fn restore(&self) -> ! {
        const PRESERVED_OFFSET: u8 = 0x48;
        const IRET_OFFSET: u8 = 0x78;
        const SS_OFFSET: u8 = IRET_OFFSET + 0x20;
        asm!(
            "mov ds, [{context} + {ss_offset}]",
            "mov es, [{context} + {ss_offset}]",
            "mov fs, [{context} + {ss_offset}]",
            "mov gs, [{context} + {ss_offset}]", // SS is handled by iret

            // setup the stack frame iret expects
            "push [{context} + {ss_offset}]", // data selector
            "push [{context} + {iret_offset} + 0x18]", // stack pointer
            "push [{context} + {iret_offset} + 0x10]", // rflags
            "push [{context} + {iret_offset} + 0x8]", // code selector
            "push [{context} + {iret_offset}]", // instruction pointer

            // restore preserved registers
            "mov r15, [{context} + {preserved_offset}]",
            "mov r14, [{context} + {preserved_offset} + 0x8]",
            "mov r13, [{context} + {preserved_offset} + 0x10]",
            "mov r12, [{context} + {preserved_offset} + 0x18]",
            "mov rbp, [{context} + {preserved_offset} + 0x20]",
            "mov rbx, [{context} + {preserved_offset} + 0x28]",

            // scratch registers are caller-saved, so they don't need to be restored

            "iretq",
            context = in(reg) &self.context,
            ss_offset = const SS_OFFSET,
            preserved_offset = const PRESERVED_OFFSET,
            iret_offset = const IRET_OFFSET,
            options(noreturn)
        );
    }
}
