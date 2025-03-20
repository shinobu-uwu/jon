use alloc::vec::Vec;
use bitmap_allocator::BitAlloc;
use libjon::fd::FileDescriptorId;
use log::{debug, info};

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

use super::{fd::FileDescriptor, memory::MemoryDescriptor};

const STACK_START: usize = 0xffff888000000000;
const STACK_SIZE: usize = 0x4000; // 16 KiB

#[derive(Debug)]
pub struct Task {
    pub pid: Pid,
    pub parent: Option<Pid>,
    pub state: State,
    pub quantum: u64,
    pub priority: Priority,
    pub context: Registers,
    pub fds: Vec<FileDescriptor>,
    next_fd: usize,
    pub kernel_stack: Stack,
    memory_descriptor: MemoryDescriptor,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Low,
    Normal,
    High,
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
            VirtualAddress::new(STACK_START + (pid.as_usize() - 1) * STACK_SIZE),
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
            parent: None,
            kernel_stack,
            context,
            state: State::Waiting,
            memory_descriptor,
            quantum: 0,
            priority: Priority::Normal,
            fds: Vec::new(),
            next_fd: 1,
        }
    }

    pub fn add_file(&mut self, descriptor: FileDescriptor) {
        self.fds.push(descriptor);
    }

    pub fn remove_file(&mut self, descriptor_id: FileDescriptorId) {
        self.fds.retain(|fd| fd.id != descriptor_id);
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        PID_ALLOCATOR.lock().dealloc(self.pid.as_usize());
    }
}
