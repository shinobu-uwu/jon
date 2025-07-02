use alloc::{string::String, vec::Vec};
use libjon::fd::FileDescriptorId;
use log::{debug, info};
use spinning_top::Spinlock;

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

pub const BINARY_START: usize = 0x400000;
const KERNEL_STACK_START: usize = 0xffff888000000000;
const USER_STACK_START: usize = 0x0000700000000000;
const STACK_SIZE: usize = 0x8000; // 32 KiB
static IDLE_BINARY: Spinlock<Option<(MemoryDescriptor, VirtualAddress)>> = Spinlock::new(None);
static LOADER: Spinlock<ElfLoader> = Spinlock::new(ElfLoader::new());
pub const BINARIES: [&[u8]; 4] = [
    include_bytes!(
        "../../../drivers/reincarnation/target/x86_64-unknown-none/release/reincarnation"
    ),
    include_bytes!("../../../drivers/task_manager/target/x86_64-unknown-none/release/task_manager"),
    include_bytes!("../../../drivers/random/target/x86_64-unknown-none/release/random"),
    include_bytes!("../../../drivers/random_echo/target/x86_64-unknown-none/release/random_echo"),
];

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
    pub memory_descriptor: MemoryDescriptor,
    pub next_fd: usize,
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
    Stopped,
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
        let mut context = Registers::new();
        let bin_addr = VirtualAddress::new(BINARY_START + (pid.as_usize() - 1) * PAGE_SIZE * 128); // TODO: Use a better dynamic address
        let loader = ElfLoader::new();
        let (memory_descriptor, rip) = loader.load(bin_addr, binary).unwrap();
        debug!("Loaded binary at {:#x?}", bin_addr);

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

    pub fn reincarnation() -> Self {
        Self::new("reincarnation", &BINARIES[0][..])
    }

    pub fn task_manager() -> Self {
        Self::new("task_manager", &BINARIES[1][..])
    }

    pub fn random() -> Self {
        Self::new("random", &BINARIES[2][..])
    }

    pub fn random_echo() -> Self {
        Self::new("random-echo", &BINARIES[3][..])
    }

    pub fn idle() -> Self {
        let pid = Pid::new(Pid::next_pid());
        let mut binary = IDLE_BINARY.lock();

        if binary.is_none() {
            *binary = Some(
                LOADER
                    .lock()
                    .load(
                        VirtualAddress::new(BINARY_START + (pid.as_usize() - 1) * PAGE_SIZE * 20),
                        include_bytes!(
                            "../../../drivers/idle/target/x86_64-unknown-none/release/idle"
                        ),
                    )
                    .unwrap(),
            );
        }
        drop(binary);

        let kernel_stack = Stack::new(
            VirtualAddress::new(KERNEL_STACK_START + (pid.as_usize() - 1) * STACK_SIZE),
            PAGE_SIZE,
        );
        let user_stack = Stack::new(
            VirtualAddress::new(USER_STACK_START + (pid.as_usize() - 1) * STACK_SIZE),
            PAGE_SIZE,
        );
        let mut context = Registers::new();
        context.iret.rsp = 0;
        context.iret.rip = IDLE_BINARY.lock().as_ref().unwrap().1.as_u64();

        Task {
            pid,
            parent: None,
            name: String::from("idle"),
            state: State::Waiting,
            quantum: 0,
            priority: Priority::Normal,
            context,
            fds: Vec::new(),
            kernel_stack,
            user_stack,
            memory_descriptor: IDLE_BINARY.lock().as_ref().unwrap().0.clone(),
            next_fd: 1,
        }
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
