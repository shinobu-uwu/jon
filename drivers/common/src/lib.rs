#![no_std]

use core::{arch::asm, fmt::Write};

use heapless::String;

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
    match syscall::fs::write(serial, b"Task panicked") {
        Ok(_) => exit(ExitCode(1)),
        Err(_) => loop {},
    };
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
    syscall(0, code.0, 0, 0, 0, 0, 0).unwrap();
    loop {}
}

#[macro_export]
macro_rules! daemon_entrypoint {
    ($name:expr, $description:expr, $version:expr, $entrypoint:ident) => {
        use core::mem::size_of;
        use $crate::syscall::fs::*;

        #[no_mangle]
        pub extern "C" fn _start() -> ! {
            let serial = open("serial:", 0x0).unwrap();
            let read_pipe = open(concat!("pipe:", $name, "/read"), 0x1).unwrap();
            let write_pipe = open(concat!("pipe:", $name, "/write"), 0x2).unwrap();
            let mut buf = [0u8; 256];
            let a = $entrypoint();
            write(serial, a.unwrap().as_bytes()).unwrap();
            read(serial, &mut buf).unwrap();

            loop {}
        }
    };
}

pub fn usize_to_str(value: usize) -> String<20> {
    let mut s = String::<20>::new();
    write!(&mut s, "{}", value).unwrap();
    s
}
