use core::arch::{asm, naked_asm};

use crate::{
    arch::x86::{
        gdt::GDT,
        structures::{Registers, Scratch},
    },
    pop_preserved, pop_scratch, push_preserved, push_scratch,
    sched::scheduler::{current_pid, current_task, remove_current_task},
    scheme::{schemes, CallerContext},
    swapgs,
};
use libjon::{
    errno::ENOENT,
    fd::{FileDescriptorFlags, FileDescriptorId},
    path::Path,
    syscall::{SYS_EXIT, SYS_OPEN, SYS_READ, SYS_WRITE},
};
use log::debug;
use x86_64::{
    registers::{
        control::{Efer, EferFlags},
        model_specific::{LStar, SFMask, Star},
        rflags::RFlags,
    },
    structures::gdt::SegmentSelector,
    VirtAddr,
};

type SyscallResult = Result<usize, i32>;

pub(super) fn init() {
    // Enable syscall/sysret
    unsafe {
        Efer::update(|efer| {
            efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS);
        });
    }

    let (kernel_cs, kernel_ss, user_cs, user_ss) = unsafe {
        (
            GDT.1.kernel_code_selector,
            GDT.1.kernel_data_selector,
            GDT.1.user_code_selector,
            GDT.1.user_data_selector,
        )
    };

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
pub unsafe extern "C" fn syscall_instruction() {
    naked_asm!(
        "swapgs;",                    // Swap KGSBASE with GSBASE, allowing fast TSS access.
        "mov gs:[{sp}], rsp;",        // Save userspace stack pointer
        "mov rsp, gs:[{ksp}];",       // Load kernel stack pointer
        "push QWORD PTR {ss_sel};",   // Push fake userspace SS (resembling iret frame)
        "push QWORD PTR gs:[{sp}];",  // Push userspace rsp
        "push r11;",                  // Push rflags
        "push QWORD PTR {cs_sel};",   // Push fake CS (resembling iret stack frame)
        "push rcx;",                  // Push userspace return pointer
        "mov r12",

        "push rax;",
        push_scratch!(),
        push_preserved!(),

        "mov rdi, rsp",

        "call {handler};",

        ".globl enter_usermode",
        "enter_usermode:",

        pop_preserved!(),
        pop_scratch!(),

        "swapgs;",

        "pop rcx;",

        // Ensure RCX is canonical (security hardening)
        "shl rcx, 16;",
        "sar rcx, 16;",

        "add rsp, 8;",              // Pop fake userspace CS
        "pop r11;",                 // Pop rflags
        "pop rsp;",                 // Restore userspace stack pointer
        "sysretq;",                 // Return into userspace; RCX=>RIP,R11=>RFLAGS
        handler = sym handle_syscall,
    sp = const(offset_of!(gdt::ProcessorControlRegion, user_rsp_tmp)),
    ksp = const(offset_of!(gdt::ProcessorControlRegion, tss) + offset_of!(TaskStateSegment, rsp)),
    ss_sel = GDT,
    cs_sel = const(SegmentSelector::new(gdt::GDT_USER_CODE as u16, x86::Ring::Ring3).bits()),
    );
}

pub unsafe extern "C" fn handle_syscall(registers: *mut Scratch) {
    let scratch = &(*registers);
    debug!("Scratch: {:#x?}", scratch);
    let (syscall_number, arg1, arg2, arg3, arg4, arg5, arg6) = (
        scratch.rax as usize,
        scratch.rdi as usize,
        scratch.rsi as usize,
        scratch.rdx as usize,
        scratch.r10 as usize,
        scratch.r8 as usize,
        scratch.r9 as usize,
    );

    debug!("Syscall {} received", syscall_number);

    let result = match syscall_number {
        SYS_EXIT => sys_exit(arg1),
        SYS_OPEN => sys_open(arg1, arg2, arg3),
        SYS_WRITE => sys_write(arg1, arg2, arg3),
        SYS_READ => sys_read(arg1, arg2, arg3),
        _ => {
            debug!("Invalid syscall number: {}", syscall_number);
            Err(ENOENT)
        }
    };

    match result {
        Ok(result) => {
            debug!("Syscall {} returned: {}", syscall_number, result);
            (*registers).rax = result as u64;
        }
        Err(errno) => {
            debug!("Syscall {} failed: {}", syscall_number, errno);
            (*registers).rax = -errno as u64;
        }
    }
}

fn sys_exit(code: usize) -> SyscallResult {
    debug!("Exiting with code: {}", code);
    remove_current_task();

    Ok(0)
}

fn sys_open(path_ptr: usize, path_len: usize, flags: usize) -> SyscallResult {
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

        match scheme.open(
            path.path,
            FileDescriptorFlags::from_bits(flags).expect("Failed to convert flags"),
            caller_context,
        ) {
            Ok(fd_id) => {
                debug!("Opened file descriptor: {:?}", fd_id);
                Ok(fd_id.0)
            }
            Err(err) => {
                debug!("Error opening file: {}", err);
                Err(err)
            }
        }
    } else {
        debug!("No scheme found for: {}", scheme_name);
        Err(ENOENT)
    }
}

fn sys_read(fd: usize, buf_ptr: usize, count: usize) -> SyscallResult {
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
    scheme.read(fd.id, buf, count)
}

fn sys_write(fd: usize, buf_ptr: usize, count: usize) -> SyscallResult {
    let task = current_task().expect("ERROR: NO CURRENT TASK");
    let fd = task
        .fds
        .iter()
        .find(|desc| desc.id == FileDescriptorId(fd))
        .expect("ERROR: FD NOT FOUND IN TASK");
    debug!("Found fd: {:?} in task", fd);
    let schemes = schemes();
    let scheme = schemes.get(fd.scheme).expect("ERROR: SCHEME NO REGISTERED");
    let buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, count) };
    debug!("Writing buffer {:x?} to fd: {:?}", buf, fd);
    scheme.write(fd.id, buf, count)
}
