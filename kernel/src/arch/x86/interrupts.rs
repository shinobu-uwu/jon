use log::debug;
use spinning_top::Spinlock;
use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder};

use crate::memory::paging::phys_to_virt;

pub static LAPIC: Spinlock<Option<LocalApic>> = Spinlock::new(None);
pub const TIMER_VECTOR: usize = 32;
pub const ERROR_VECTOR: usize = TIMER_VECTOR + 1;
pub const SPURIOUS_VECTOR: usize = ERROR_VECTOR + 1;

pub(super) fn init() {
    let phys_lapic = unsafe { xapic_base() };
    let virt_lapic = phys_to_virt(phys_lapic as usize);

    debug!(
        "Initializing LAPIC - Physical: {:#x}, Virtual: {:#x}",
        phys_lapic, virt_lapic
    );

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(TIMER_VECTOR)
        .error_vector(ERROR_VECTOR)
        .spurious_vector(SPURIOUS_VECTOR)
        .set_xapic_base(virt_lapic as u64)
        .build()
        .expect("Failed to build LAPIC");

    unsafe {
        debug!("Enabling LAPIC");
        lapic.enable();
        debug!("LAPIC enabled");
    }

    debug!("Storing LAPIC instance");
    *LAPIC.lock() = Some(lapic);
    debug!("LAPIC initialization complete");
}

#[macro_export]
macro_rules! push_scratch {
    () => {
        concat!(
            "push rax\n",
            "push rcx\n",
            "push rdx\n",
            "push rdi\n",
            "push rsi\n",
            "push r8\n",
            "push r9\n",
            "push r10\n",
            "push r11\n",
        )
    };
}

#[macro_export]
macro_rules! pop_scratch {
    () => {
        concat!(
            "pop r11\n",
            "pop r10\n",
            "pop r9\n",
            "pop r8\n",
            "pop rsi\n",
            "pop rdi\n",
            "pop rdx\n",
            "pop rcx\n",
            "pop rax\n",
        )
    };
}

#[macro_export]
macro_rules! push_preserved {
    () => {
        concat!(
            "push rbx\n",
            "push rbp\n",
            "push r12\n",
            "push r13\n",
            "push r14\n",
            "push r15\n",
        )
    };
}

#[macro_export]
macro_rules! pop_preserved {
    () => {
        concat!(
            "pop r15\n",
            "pop r14\n",
            "pop r13\n",
            "pop r12\n",
            "pop rbp\n",
            "pop rbx\n",
        )
    };
}

#[macro_export]
macro_rules! interrupt {
    ($name:ident, |$arg:ident| $code:block) => {

        #[naked]
        pub extern "x86-interrupt" fn $name(_frame: InterruptStackFrame) {
            use crate::{pop_scratch, push_scratch, push_preserved, pop_preserved, swapgs, arch::x86::structures::Registers};

            unsafe extern "C" fn inner($arg: &Registers) {
                $code
            }

            unsafe {
                core::arch::naked_asm!(
                    "cld",
                    swapgs!(),
                    push_preserved!(),
                    push_scratch!(),
                    "mov rdi, rsp",
                    "call {inner}",
                    pop_scratch!(),
                    pop_preserved!(),
                    "iretq",
                    inner = sym inner,
                );
            }
        }
    };
}

#[macro_export]
macro_rules! swapgs {
    () => {
        "
        test QWORD PTR [rsp + 8], 0x3
        jz 2f
        swapgs
        2:
    "
    };
}
