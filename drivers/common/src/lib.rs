#![no_std]

pub mod framebuffer;

use core::arch::asm;

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
    println!("{}", info);
    loop {}
}

#[inline(always)]
pub unsafe extern "sysv64" fn syscall(
    number: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> usize {
    let result: usize;

    unsafe {
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
    }

    result
}

pub fn exit(code: ExitCode) -> ! {
    unsafe {
        syscall(0, code.0, 0, 0, 0, 0, 0);
    }
    loop {}
}

#[inline]
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    let s = args.as_str().unwrap();
    unsafe {
        syscall(1, s.as_ptr() as usize, s.len(), 0, 0, 0, 0);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {{
        $crate::print!("{}\n", format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! module_entrypoint {
    ($name:expr, $description:expr, $version:expr, $entrypoint:ident) => {
        #[no_mangle]
        pub extern "C" fn _start() -> ! {
            let result = $entrypoint();

            let exit_code = match result {
                Ok(_) => ExitCode(0),
                Err(code) => code,
            };

            $crate::exit(exit_code);
        }
    };
}
