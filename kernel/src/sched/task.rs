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
    pub affinity_mask: CpuAffinityMask,
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
            VirtualAddress::new(KERNEL_STACK_START + pid.as_usize() * STACK_SIZE),
            STACK_SIZE,
        );
        let user_stack = Stack::new(
            VirtualAddress::new(USER_STACK_START + pid.as_usize() * STACK_SIZE),
            STACK_SIZE,
        );
        debug!("Finished creating stack");
        let mut context = Registers::new();
        let bin_addr = VirtualAddress::new(BINARY_START + (pid.as_usize() - 1) * PAGE_SIZE * 20); // TODO: Use a better dynamic address
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
            affinity_mask: CpuAffinityMask::new_all_cpus(),
        }
    }

    pub fn idle() -> Self {
        let mut binary = IDLE_BINARY.lock();
        if binary.is_none() {
            let loader = ElfLoader::new();
            *binary = Some(
                loader
                    .load(
                        VirtualAddress::new(BINARY_START),
                        include_bytes!(
                            "../../../drivers/idle/target/x86_64-unknown-none/release/idle"
                        ),
                    )
                    .unwrap(),
            );
        }

        let kernel_stack = Stack::new(VirtualAddress::new(KERNEL_STACK_START), PAGE_SIZE);
        let user_stack = Stack::new(VirtualAddress::new(USER_STACK_START), PAGE_SIZE);
        let mut context = Registers::new();
        context.iret.rsp = user_stack.top().as_u64();
        context.iret.rip = binary.as_ref().unwrap().1.as_u64();

        Task {
            pid: Pid::new(Pid::next_pid()),
            parent: None,
            name: String::from("idle"),
            state: State::Waiting,
            quantum: 0,
            priority: Priority::Normal,
            context,
            fds: Vec::new(),
            kernel_stack,
            user_stack,
            memory_descriptor: binary.as_ref().unwrap().0.clone(),
            next_fd: 1,
            affinity_mask: CpuAffinityMask::new_all_cpus(),
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

    pub fn set_cpu_affinity(&mut self, cpu_id: u64) {
        self.affinity_mask = CpuAffinityMask::new_single_cpu(cpu_id);
    }

    pub fn set_cpu_affinity_mask(&mut self, mask: CpuAffinityMask) {
        self.affinity_mask = mask;
    }

    pub fn clear_cpu_affinity(&mut self) {
        self.affinity_mask = CpuAffinityMask::new_all_cpus();
    }

    pub fn has_cpu_affinity_mask(&self) -> bool {
        self.affinity_mask.has_restrictions()
    }

    pub fn can_run_on_cpu(&self, cpu_id: u64) -> bool {
        self.affinity_mask.can_run_on_cpu(cpu_id)
    }

    pub fn preferred_cpu(&self) -> Option<u64> {
        self.affinity_mask.first_cpu()
    }

    pub fn has_cpu_affinity(&self, cpu_id: u64) -> bool {
        self.affinity_mask.can_run_on_cpu(cpu_id)
    }
}

/// A CPU affinity mask to specify which CPUs a task can run on
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuAffinityMask {
    /// Each bit represents a CPU (1 = can run on this CPU)
    mask: u64,
}

impl CpuAffinityMask {
    /// Create a new affinity mask allowing execution on all CPUs
    pub fn new_all_cpus() -> Self {
        Self { mask: u64::MAX }
    }

    /// Create a new affinity mask for a specific CPU only
    pub fn new_single_cpu(cpu_id: u64) -> Self {
        Self { mask: 1 << cpu_id }
    }

    /// Check if a CPU is allowed in this mask
    pub fn can_run_on_cpu(&self, cpu_id: u64) -> bool {
        if cpu_id >= 64 {
            return false; // We only support up to 64 CPUs with this mask
        }
        (self.mask & (1 << cpu_id)) != 0
    }

    /// Add a CPU to the allowed set
    pub fn add_cpu(&mut self, cpu_id: u64) {
        if cpu_id < 64 {
            self.mask |= 1 << cpu_id;
        }
    }

    /// Remove a CPU from the allowed set
    pub fn remove_cpu(&mut self, cpu_id: u64) {
        if cpu_id < 64 {
            self.mask &= !(1 << cpu_id);
        }
    }

    /// Get the first allowed CPU
    pub fn first_cpu(&self) -> Option<u64> {
        if self.mask == 0 {
            None
        } else {
            Some(self.mask.trailing_zeros() as u64)
        }
    }

    /// Check if this mask has any restrictions
    pub fn has_restrictions(&self) -> bool {
        self.mask != u64::MAX
    }
}
