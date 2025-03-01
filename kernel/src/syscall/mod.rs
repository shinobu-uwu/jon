use core::arch::{asm, naked_asm};

use crate::{
    arch::x86::gdt::GDT,
    path::Path,
    pop_preserved, pop_scratch, push_preserved, push_scratch,
    sched::{
        pid::Pid,
        scheduler::{current_pid, current_task, remove_current_task},
    },
    scheme::{schemes, CallerContext},
};
use libjon::fd::FileDescriptorId;
use log::{debug, info};
use x86_64::{
    registers::{
        control::{Efer, EferFlags},
        model_specific::{LStar, SFMask, Star},
        rflags::RFlags,
    },
    VirtAddr,
};

static mut COUNT: usize = 0;

static SYSCALLS: &[Option<fn(usize, usize, usize, usize, usize, usize) -> usize>] = &[
    Some(sys_exit),
    Some(sys_print),
    Some(sys_open),
    Some(sys_read),
];

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

    LStar::write(VirtAddr::new(syscall_instruction as u64));
    SFMask::write(RFlags::INTERRUPT_FLAG);
}

#[naked]
#[allow(named_asm_labels)]
pub unsafe extern "C" fn syscall_instruction() {
    naked_asm!(
        "swapgs;",

        push_scratch!(),
        push_preserved!(),

        "call {handler};",


        ".globl enter_usermode",
        "enter_usermode:",
        pop_preserved!(),
        pop_scratch!(),

        // We must ensure RCX is canonical; if it is not when running sysretq, the consequences can be
        // fatal from a security perspective.
        //
        // See https://xenproject.org/2012/06/13/the-intel-sysret-privilege-escalation/.
        //
        // This is not just theoretical; ptrace allows userspace to change RCX (via RIP) of target
        // processes.
        //
        // While we could also conditionally IRETQ here, an easier method is to simply sign-extend RCX:

        // Shift away the upper 16 bits (0xBAAD_8000_DEAD_BEEF => 0x8000_DEAD_BEEF_XXXX).
        "shl rcx, 16;",
        // Shift arithmetically right by 16 bits, effectively extending the 47th sign bit to bits
        // 63:48 (0x8000_DEAD_BEEF_XXXX => 0xFFFF_8000_DEAD_BEEF).
        "sar rcx, 16;",

        "swapgs;",
        "sysretq;",                 // Return into userspace; RCX=>RIP,R11=>RFLAGS
        handler = sym handle_syscall,
    );
}

pub unsafe extern "C" fn handle_syscall() -> usize {
    let (syscall_number, arg1, arg2, arg3, arg4, arg5, arg6): (
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    );

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

    if arg1 == 0 {
        loop {}
    }

    debug!("Syscall {} received", syscall_number);

    let result = match SYSCALLS.get(syscall_number) {
        Some(Some(syscall)) => {
            debug!("Calling syscall: {}", syscall_number);
            let ret = syscall(arg1, arg2, arg3, arg4, arg5, arg6);
            debug!("Returning {} from syscall {}", ret, syscall_number);
            ret
        }
        _ => {
            debug!("Invalid syscall number: {}", syscall_number);
            usize::MAX
        }
    };

    unsafe {
        asm!(
            "mov rax, {}",
            in(reg) result,
        );
    }

    result
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

fn sys_exit(code: usize, _: usize, _: usize, _: usize, _: usize, _: usize) -> usize {
    debug!("Exiting with code: {}", code);
    remove_current_task();

    0
}

fn sys_open(path_ptr: usize, path_len: usize, flags: usize, _: usize, _: usize, _: usize) -> usize {
    debug!("Opening file");
    debug!("Path pointer: {:#x?}", path_ptr);
    debug!("Path length: {}", path_len);
    let path = unsafe {
        let slice = core::slice::from_raw_parts(path_ptr as *const u8, path_len);
        let str = core::str::from_utf8_unchecked(slice);
        Path::from(str)
    };

    debug!("sys_open called with path: {}", path);

    let scheme_name = path.scheme;

    let scheme = schemes();
    if let Some((id, scheme)) = scheme.get_name(scheme_name) {
        let caller_context = CallerContext {
            pid: current_pid().expect("ERR: NO CURRENT PID"),
            scheme: id,
        };

        match scheme.open(path.path, flags, caller_context) {
            Ok(fd_id) => {
                debug!("Opened file descriptor: {:?}", fd_id);
                fd_id.0
            }
            Err(err) => {
                debug!("Error opening file: {}", err);
                usize::MAX
            }
        }
    } else {
        debug!("No scheme found for: {}", scheme_name);
        usize::MAX
    }
}

fn sys_read(fd: usize, buf_ptr: usize, count: usize, _: usize, _: usize, _: usize) -> usize {
    let task = current_task().expect("ERROR: NO CURRENT TASK");
    let fd = task
        .fds
        .iter()
        .find(|desc| desc.id == FileDescriptorId(fd))
        .expect("ERROR: FD NOT FOUND IN TASK");
    let schemes = schemes();
    let scheme = schemes.get(fd.scheme).expect("ERROR: SCHEME NO REGISTERED");
    debug!("Reading from fd: {:?}", fd);
    debug!("Reading into buffer: {:#x?}", buf_ptr);
    debug!("Reading count: {}", count);
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, count) };
    match scheme.read(fd.id, buf, count) {
        Ok(n) => n,
        Err(_) => usize::MAX,
    }
}

fn sys_write(fd: usize, buf_ptr: usize, count: usize, _: usize, _: usize, _: usize) -> usize {
    let task = current_task().expect("ERROR: NO CURRENT TASK");
    let fd = task
        .fds
        .iter()
        .find(|desc| desc.id == FileDescriptorId(fd))
        .expect("ERROR: FD NOT FOUND IN TASK");
    let schemes = schemes();
    let scheme = schemes.get(fd.scheme).expect("ERROR: SCHEME NO REGISTERED");
    let buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, count) };
    match scheme.write(fd.id, buf, count) {
        Ok(n) => n,
        Err(_) => usize::MAX,
    }
}
