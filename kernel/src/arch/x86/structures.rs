use super::gdt::GDT;

#[derive(Debug, Default)]
#[repr(C)]
pub struct Registers {
    pub scratch: Scratch,
    pub preserved: Preserved,
    pub iret: Iret,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Scratch {
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: u64,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Preserved {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Iret {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl Registers {
    pub fn new() -> Self {
        let mut stack = Self::default();
        stack.iret.rflags = x86_64::registers::rflags::RFlags::INTERRUPT_FLAG.bits();
        unsafe {
            stack.iret.cs = GDT.1.user_code_selector.0 as u64;
            stack.iret.ss = GDT.1.user_data_selector.0 as u64;
        }

        stack
    }
}
