use core::arch::asm;

use gdt::set_tss_kernel_stack;
use structures::Registers;

use crate::sched::task::Task;

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

pub unsafe fn switch_to(prev: Option<&mut Task>, next: &Task, current_stack_frame: &Registers) {
    if let Some(t) = prev {
        save(&mut t.context, current_stack_frame);
    }

    set_tss_kernel_stack(next.kernel_stack.top());
    restore(&next.context);
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
pub unsafe fn restore(context: &Registers) -> ! {
    const PRESERVED_OFFSET: u8 = 0x48;
    const SCRATCH_OFFSET: u8 = 0x00;
    const IRET_OFFSET: u8 = 0x78;
    const SS_OFFSET: u8 = IRET_OFFSET + 0x20;
    asm!(
        // Setup iret stack frame first before we touch any registers
        "push [r12 + {ss_offset}]",          // SS
        "push [r12 + {iret_offset} + 0x18]", // RSP
        "push [r12 + {iret_offset} + 0x10]", // RFLAGS
        "push [r12 + {iret_offset} + 0x8]",  // CS
        "push [r12 + {iret_offset}]",        // RIP

        // Restore scratch registers
        "mov r11, [r12 + {scratch_offset}]",
        "mov r10, [r12 + {scratch_offset} + 0x8]",
        "mov r9, [r12 + {scratch_offset} + 0x10]",
        "mov r8, [r12 + {scratch_offset} + 0x18]",
        "mov rsi, [r12 + {scratch_offset} + 0x20]",
        "mov rdi,  [r12 + {scratch_offset} + 0x28]",
        "mov rdx,  [r12 + {scratch_offset} + 0x30]",
        "mov rcx, [r12 + {scratch_offset} + 0x38]",
        "mov rax, [r12 + {scratch_offset} + 0x40]",

        // Restore preserved registers LAST
        // We're using r12 as a temporary, so restore it last
        "mov r15, [r12 + {preserved_offset}]",
        "mov r14, [r12 + {preserved_offset} + 0x8]",
        "mov r13, [r12 + {preserved_offset} + 0x10]",
        "mov rbp, [r12 + {preserved_offset} + 0x20]",
        "mov rbx, [r12 + {preserved_offset} + 0x28]",
        "mov r12, [r12 + {preserved_offset} + 0x18]", // Restore r12 last

        "iretq",
        in("r12") context,
        ss_offset = const SS_OFFSET,
        preserved_offset = const PRESERVED_OFFSET,
        scratch_offset = const SCRATCH_OFFSET,
        iret_offset = const IRET_OFFSET,
        options(noreturn)
    );
}
