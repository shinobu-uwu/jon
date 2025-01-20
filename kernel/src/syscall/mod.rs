use core::arch::{asm, naked_asm};

use crate::arch::x86::gdt::GDT;
use log::{debug, info};
use x86_64::{
    registers::{
        control::{Efer, EferFlags},
        model_specific::{LStar, SFMask, Star},
        rflags::RFlags,
    },
    VirtAddr,
};

static SYSCALLS: &[Option<fn(usize, usize, usize, usize, usize, usize) -> usize>] =
    &[None, Some(sys_print)];

pub(super) fn init() {
    // Enable syscall/sysret
    unsafe {
        Efer::update(|efer| {
            efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS);
        });
    }

    let kernel_cs = GDT.1.kernel_code_selector;
    let kernel_ss = GDT.1.kernel_data_selector;
    let user_cs = GDT.1.user_code_selector;
    let user_ss = GDT.1.user_data_selector;

    debug!("User CS: {:#x?}", user_cs.0);
    debug!("User SS: {:#x?}", user_ss.0);
    debug!("Kernel CS: {:#x?}", kernel_cs.0);
    debug!("Kernel SS: {:#x?}", kernel_ss.0);

    match Star::write(user_cs, user_ss, kernel_cs, kernel_ss) {
        Ok(_) => {
            debug!("STAR MSR set successfully");
        }
        Err(e) => {
            panic!("Error setting STAR: {}", e)
        }
    }

    LStar::write(VirtAddr::new(syscall_handler as u64));
    SFMask::write(RFlags::INTERRUPT_FLAG);
}

#[naked]
unsafe extern "sysv64" fn syscall_handler() -> ! {
    naked_asm!(
        "swapgs",

        "push rcx",
        "push r11",
        "push rax",
        "push rdi",
        "push rsi",
        "push rdx",
        "push r10",
        "push r8",
        "push r9",

        "call {handler}",

        "pop r9",
        "pop r8",
        "pop r10",
        "pop rdx",
        "pop rsi",
        "pop rdi",
        "pop rax",
        "pop r11",
        "pop rcx",

        "swapgs",
        "sysretq",
        handler = sym handle_syscall,
    );
}

fn handle_syscall() -> usize {
    let (syscall_number, arg1, arg2, arg3, arg4, arg5, arg6): (
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    );

    unsafe {
        asm!(
            "mov {}, rax",
            "mov {}, rdi",
            "mov {}, rsi",
            "mov {}, rdx",
            "mov {}, r10",
            "mov {}, r8",
            "mov {}, r9",
            out(reg) syscall_number,
            out(reg) arg1,
            out(reg) arg2,
            out(reg) arg3,
            out(reg) arg4,
            out(reg) arg5,
            out(reg) arg6,
        );
    }

    debug!("Syscall {} received", syscall_number);

    match SYSCALLS.get(syscall_number) {
        Some(Some(syscall)) => {
            debug!("Calling syscall: {}", syscall_number);
            syscall(arg1, arg2, arg3, arg4, arg5, arg6)
        }
        _ => {
            debug!("Invalid syscall number: {}", syscall_number);
            usize::MAX
        }
    }
}

fn sys_print(string_ptr: usize, length: usize, _: usize, _: usize, _: usize, _: usize) -> usize {
    debug!(
        "Printing string: {:#x?} with length: {}",
        string_ptr, length
    );
    let string = unsafe { core::slice::from_raw_parts(string_ptr as *const u8, length) };
    let string = core::str::from_utf8(string).unwrap();
    info!("{}", string);
    0
}
