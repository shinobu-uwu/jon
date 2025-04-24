#![no_std]
#![feature(never_type, let_chains)]

use core::arch::asm;
use core::fmt::Write;

use heapless::String;

pub mod daemon;
pub mod ipc;
pub mod syscall;

#[derive(Debug)]
pub struct ModuleInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub version: &'static str,
}

#[derive(Debug)]
pub struct ExitCode(pub usize);

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    let serial = match syscall::fs::open("serial:", 0x0) {
        Ok(fd) => fd,
        Err(_) => loop {},
    };

    let mut message = String::<256>::new();

    match info.location() {
        Some(l) => write!(
            message,
            "Error: {} at {} {}:{}",
            info.message(),
            l.file(),
            l.line(),
            l.column()
        )
        .unwrap(),
        None => write!(message, "Error: {}", info.message().as_str().unwrap()).unwrap(),
    };

    syscall::fs::write(serial, message.as_bytes()).unwrap();

    exit(ExitCode(1));
}

#[inline(always)]
pub fn syscall(
    number: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> Result<usize, i32> {
    // Result doesn't have a stable representation, so we can't return it from an extern "sysv64"
    // function
    unsafe extern "sysv64" fn inner_syscall(
        number: usize,
        arg1: usize,
        arg2: usize,
        arg3: usize,
        arg4: usize,
        arg5: usize,
        arg6: usize,
    ) -> isize {
        let result: isize;

        asm!(
            "syscall",
            in("rax") number,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            in("r8") arg5,
            in("r9") arg6,
            out("rcx") _,
            out("r11") _,
            lateout("rax") result,
        );

        result
    }

    let result = unsafe { inner_syscall(number, arg1, arg2, arg3, arg4, arg5, arg6) };

    if result < 0 {
        Err(-result as i32)
    } else {
        Ok(result as usize)
    }
}

pub fn exit(code: ExitCode) -> ! {
    syscall(93, code.0, 0, 0, 0, 0, 0).unwrap();
    loop {}
}
