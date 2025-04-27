use alloc::{string::String, vec::Vec};
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
    sched::pid::Pid,
};

use super::{fd::FileDescriptor, memory::MemoryDescriptor};

const KERNEL_STACK_START: usize = 0xffff888000000000;
const USER_STACK_START: usize = 0x0000700000000000;
const STACK_SIZE: usize = 0x8000; // 32 KiB

#[derive(Debug)]
pub struct Task {
    pub pid: Pid,
    pub parent: Option<Pid>,
    pub name: String,
    pub state: State,
    pub quantum: u64,
    pub priority: Priority,
    pub context: Registers,
    pub fds: Vec<FileDescriptor>,
    pub kernel_stack: Stack,
    pub user_stack: Stack,
    next_fd: usize,
    memory_descriptor: MemoryDescriptor,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Low,
    Normal,
    High,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running,
    Blocked,
    Waiting,
    Zombie,
}

impl Task {
    pub fn new(name: &str, binary: &[u8]) -> Self {
        let pid = Pid::new(Pid::next_pid());
        info!("Creating task {} with PID {}", name, pid);
        let kernel_stack = Stack::new(
            VirtualAddress::new(KERNEL_STACK_START + (pid.as_usize() - 1) * STACK_SIZE),
            STACK_SIZE,
        );
        let user_stack = Stack::new(
            VirtualAddress::new(USER_STACK_START + (pid.as_usize() - 1) * STACK_SIZE),
            STACK_SIZE,
        );
        debug!("Finished creating stack");
        let mut context = Registers::new();
        let bin_addr = VirtualAddress::new(0x400000 + (pid.as_usize() - 1) * PAGE_SIZE * 20); // TODO: Use a better dynamic address
        let loader = ElfLoader::new();
        debug!("Loading binary");
        let (memory_descriptor, rip) = loader.load(bin_addr, binary).unwrap();
        info!("Loaded binary at {:#x?}", bin_addr);

        context.iret.rsp = user_stack.top().as_u64();
        context.iret.rip = rip.as_u64();

        Self {
            pid,
            name: String::from(name),
            parent: None,
            kernel_stack,
            user_stack,
            context,
            state: State::Waiting,
            memory_descriptor,
            quantum: 0,
            priority: Priority::Normal,
            fds: Vec::new(),
            next_fd: 1,
        }
    }

    pub fn idle() -> Self {
        Task::new(
            "idle",
            include_bytes!("../../../drivers/idle/target/x86_64-unknown-none/release/idle"),
        )
    }

    pub fn add_file(&mut self, descriptor: FileDescriptor) {
        debug!("Adding file descriptor: {:?}", descriptor);
        self.fds.push(descriptor);
        self.next_fd += 1;
    }

    pub fn remove_file(&mut self, descriptor_id: FileDescriptorId) {
        self.fds.retain(|fd| fd.id != descriptor_id);
    }
}
