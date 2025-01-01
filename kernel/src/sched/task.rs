use core::arch::asm;

use alloc::vec;
use alloc::vec::Vec;
use bitmap_allocator::BitAlloc;

use super::{pid::Pid, PID_ALLOCATOR};

#[derive(Debug)]
pub struct Task {
    pub pid: Pid,
    pub state: TaskState,
    pub parent: Option<Pid>,
    pub context: Context,
    pub stack: Vec<u8>,
}

#[derive(Debug)]
pub enum TaskState {
    Running,
    Sleeping,
    Stopped,
    Zombie,
}

impl Task {
    pub fn new(parent: Option<Pid>, entry_point: fn()) -> Self {
        let pid = Pid::new(PID_ALLOCATOR.lock().alloc().expect("Failed to alloc pid"));
        let mut context = Context::default();
        let stack = vec![0; 1024];
        let stack_top = stack.as_ptr() as u64 + 1024 * 1024; // stacks grow from top to bottom

        context.rip = entry_point as u64;
        context.rsp = stack_top;

        Self {
            pid,
            state: TaskState::Running,
            parent,
            context,
            stack,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
// TODO make it generic, for now this is x86 only
pub struct Context {
    // General-purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64, // Stack pointer

    // Extended registers
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // Instruction pointer and flags
    pub rip: u64,    // Instruction pointer
    pub rflags: u64, // CPU flags

    // Segment registers (for task switching to user space)
    pub cs: u64, // Code segment
    pub ds: u64, // Data segment
    pub es: u64, // Extra segment
    pub fs: u64,
    pub gs: u64,
    pub ss: u64, // Stack segment
}

impl Context {
    pub unsafe fn save(&mut self) {
        asm!(
            // General purpose registers
            "mov [{0} + 0], rax",    // offset 0: save rax
            "mov [{0} + 8], rbx",    // offset 8: save rbx
            "mov [{0} + 16], rcx",   // offset 16: save rcx
            "mov [{0} + 24], rdx",   // offset 24: save rdx
            "mov [{0} + 32], rsi",   // offset 32: save rsi
            "mov [{0} + 40], rdi",   // offset 40: save rdi
            "mov [{0} + 48], rbp",   // offset 48: save base pointer
            "mov [{0} + 56], rsp",   // offset 56: save stack pointer

            // Extended registers
            "mov [{0} + 64], r8",    // offset 64: save r8
            "mov [{0} + 72], r9",    // offset 72: save r9
            "mov [{0} + 80], r10",   // offset 80: save r10
            "mov [{0} + 88], r11",   // offset 88: save r11
            "mov [{0} + 96], r12",   // offset 96: save r12
            "mov [{0} + 104], r13",  // offset 104: save r13
            "mov [{0} + 112], r14",  // offset 112: save r14
            "mov [{0} + 120], r15",  // offset 120: save r15

            // Instruction pointer and flags
            "pushfq",                // push flags onto stack
            "pop rax",               // offset 136: save flags
            "lea rax, [rip+0]",      // get current instruction pointer
            "mov [{0} + 128], rax",  // offset 128: save instruction pointer

            // Segment registers
            "mov [{0} + 144], cs",   // offset 144: save code segment
            "mov [{0} + 152], ds",   // offset 152: save data segment
            "mov [{0} + 160], es",   // offset 160: save extra segment
            "mov [{0} + 168], fs",   // offset 168: save fs
            "mov [{0} + 176], gs",   // offset 176: save gs
            "mov [{0} + 184], ss",   // offset 184: save stack segment

            in(reg) self as *mut Context  // Input: pointer to Context struct
        );
    }

    pub unsafe fn restore(&self) {
        asm!(
            // Segment registers first
            "mov cs, [{0} + 144]",   // restore code segment
            "mov ds, [{0} + 152]",   // restore data segment
            "mov es, [{0} + 160]",   // restore extra segment
            "mov fs, [{0} + 168]",   // restore fs
            "mov gs, [{0} + 176]",   // restore gs
            "mov ss, [{0} + 184]",   // restore stack segment

            // General purpose registers
            "mov rax, [{0} + 0]",    // restore rax
            "mov rbx, [{0} + 8]",    // restore rbx
            "mov rcx, [{0} + 16]",   // restore rcx
            "mov rdx, [{0} + 24]",   // restore rdx
            "mov rsi, [{0} + 32]",   // restore rsi
            "mov rdi, [{0} + 40]",   // restore rdi
            "mov rbp, [{0} + 48]",   // restore base pointer

            // Extended registers
            "mov r8,  [{0} + 64]",   // restore r8
            "mov r9,  [{0} + 72]",   // restore r9
            "mov r10, [{0} + 80]",   // restore r10
            "mov r11, [{0} + 88]",   // restore r11
            "mov r12, [{0} + 96]",   // restore r12
            "mov r13, [{0} + 104]",  // restore r13
            "mov r14, [{0} + 112]",  // restore r14
            "mov r15, [{0} + 120]",  // restore r15

            // Flags
            "push [{0} + 136]",      // push saved flags
            "popfq",                 // restore flags

            // Stack and instruction pointer last
            "mov rsp, [{0} + 56]",   // restore stack pointer
            "jmp [{0} + 128]",       // jump to saved instruction pointer

            in(reg) self as *const Context  // Input: pointer to Context struct
        );
    }
}
