use core::arch::asm;

use structures::Registers;

pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod memory;
pub mod structures;

pub fn init() {
    gdt::init();
    idt::init();
    memory::allocator::init();
    interrupts::init();
}

pub fn switch_to(
    prev_context: &mut Registers,
    next_context: &Registers,
    current_stack_frame: &Registers,
) {
    unsafe {
        save(prev_context, current_stack_frame);
        restore(next_context);
    }
}

pub unsafe fn save(context: &mut Registers, stack_frame: &Registers) {
    context.iret.rip = stack_frame.iret.rip;
    context.iret.rsp = stack_frame.iret.rsp;
    context.iret.rflags = stack_frame.iret.rflags;
    context.iret.cs = stack_frame.iret.cs;
    context.iret.ss = stack_frame.iret.ss;
    context.preserved.r15 = stack_frame.preserved.r15;
    context.preserved.r14 = stack_frame.preserved.r14;
    context.preserved.r13 = stack_frame.preserved.r13;
    context.preserved.r12 = stack_frame.preserved.r12;
    context.preserved.rbp = stack_frame.preserved.rbp;
    context.preserved.rbx = stack_frame.preserved.rbx;
    context.scratch.rax = stack_frame.scratch.rax;
    context.scratch.rcx = stack_frame.scratch.rcx;
    context.scratch.rdx = stack_frame.scratch.rdx;
    context.scratch.rdi = stack_frame.scratch.rdi;
    context.scratch.rsi = stack_frame.scratch.rsi;
    context.scratch.r8 = stack_frame.scratch.r8;
    context.scratch.r9 = stack_frame.scratch.r9;
    context.scratch.r10 = stack_frame.scratch.r10;
    context.scratch.r11 = stack_frame.scratch.r11;
}

// IMPORTANT: any acquired locks, or anything you must do before switching tasks,
// must be released or done here, after this the kernel will jump to the task's instruction pointer
unsafe fn restore(context: &Registers) -> ! {
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
        context = in(reg) context,
        ss_offset = const SS_OFFSET,
        preserved_offset = const PRESERVED_OFFSET,
        iret_offset = const IRET_OFFSET,
        options(noreturn)
    );
}
