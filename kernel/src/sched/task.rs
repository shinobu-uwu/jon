use core::arch::asm;

use bitmap_allocator::BitAlloc;
use log::debug;
use x86_64::structures::idt::InterruptStackFrame;

use crate::{
    arch::x86::{gdt::GDT, interrupts::LAPIC},
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
    context: Context,
}

#[derive(Debug)]
pub enum State {
    Running,
    Sleeping,
    Waiting,
    Zombie,
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct Context {
    // Callee-saved registers
    rsp: u64, // Stack pointer
    // Return address for context switch
    rip: u64,
    rbp: u64, // Base pointer
    rax: u64,
    rbx: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
}

impl Task {
    pub fn new(binary: &[u8]) -> Self {
        let pid = Pid::new(PID_ALLOCATOR.lock().alloc().unwrap());
        debug!("Creating task with PID {}", pid);
        let kernel_stack = Stack::new(
            VirtualAddress::new(STACK_START + pid.as_usize() * STACK_SIZE),
            STACK_SIZE,
        );
        let mut context = Context::default();
        let bin_addr = VirtualAddress::new(0x400000 + (pid.as_usize() - 1) * PAGE_SIZE * 100); // TODO: Use a better dynamic address
        let loader = ElfLoader::new();
        let (memory_descriptor, rip) = loader.load(bin_addr, binary).unwrap();

        context.rsp = kernel_stack.top().as_u64();
        context.rip = rip.as_u64();

        Self {
            pid,
            kernel_stack,
            context,
            state: State::Running,
            memory_descriptor,
        }
    }

    pub unsafe fn save(&mut self, stack_frame: &InterruptStackFrame) {
        self.context.rip = stack_frame.instruction_pointer.as_u64();
        self.context.rsp = stack_frame.stack_pointer.as_u64();
        // debug!("Saved {:#x?}", self);
    }

    pub unsafe fn restore(&self) -> ! {
        debug!("Restoring task {:#x?}", self,);
        LAPIC.lock().as_mut().unwrap().end_of_interrupt();
        asm!(
            "mov ds, [{gdt} + 6]",
            "mov es, [{gdt} + 6]",
            "mov fs, [{gdt} + 6]",
            "mov gs, [{gdt} + 6]", // SS is handled by iret
            // setup the stack frame iret expects
            "push [{gdt} + 6]", // data selector
            "push [{context}]",          // stack pointer
            "push 0x3202",
            "push [{gdt} + 4]", // code selector
            "push [{context} + 8]", // instruction pointer
            "iretq",
            gdt = in(reg) &GDT.1, // each selector is 16 bits
            context = in(reg) &self.context,
            options(noreturn)
        );
    }
}
