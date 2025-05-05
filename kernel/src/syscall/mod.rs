use core::{arch::naked_asm, mem::offset_of};

use crate::{
    arch::x86::{
        cpu::{ProcessorControlRegion, PCRS},
        memory::{PMM, VMM},
        structures::Scratch,
    },
    memory::{address::VirtualAddress, paging::PageFlags, physical::PhysicalMemoryManager},
    pop_scratch, push_scratch,
    sched::{
        pid::Pid,
        scheduler::{
            self, current_pid, current_task, current_task_mut, remove_current_task, remove_task,
        },
    },
    scheme::{schemes, CallerContext},
};
use libjon::{
    errno::{EINTR, EINVAL, ENOENT, ENOMEM, ESRCH},
    fd::{FileDescriptorFlags, FileDescriptorId},
    path::Path,
    syscall::{SYS_BRK, SYS_EXIT, SYS_GETPID, SYS_KILL, SYS_LSEEK, SYS_OPEN, SYS_READ, SYS_WRITE},
};
use log::{debug, error, info};
use x86_64::{
    registers::{
        control::{Efer, EferFlags},
        model_specific::{LStar, SFMask, Star},
        rflags::RFlags,
    },
    VirtAddr,
};

type SyscallResult = Result<usize, i32>;

pub(super) fn init(cpu_id: u32) {
    let pcr = unsafe { PCRS.get_mut(cpu_id as usize).unwrap() };
    // Enable syscall/sysret
    unsafe {
        Efer::update(|efer| {
            efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS);
        });
    }
    let selectors = pcr.selectors.as_ref().unwrap();

    let (kernel_cs, kernel_ss, user_cs, user_ss) = (
        selectors.kernel_code_selector,
        selectors.kernel_data_selector,
        selectors.user_code_selector,
        selectors.user_data_selector,
    );

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
        "swapgs;",                    // Swap KGSBASE with GSBASE, allowing fast TSS access.
        "mov gs:[{sp}], rsp;",        // Save userspace stack pointer
        "mov rsp, gs:[{ksp}];",       // Load kernel stack pointer

        "push r11;",
        "push rcx;",

        // Push context registers
        push_scratch!(),

        "mov rdi, rsp;",
        "call {handler};",

        pop_scratch!(),

        // Restore user GSBASE by swapping GSBASE and KGSBASE.
        "swapgs;",

        "pop rcx",
        "pop r11;",
        "mov rsp, gs:[{sp}];",        // Restore userspace stack pointer
        "sysretq;",                 // Return into userspace; RCX=>RIP,R11=>RFLAGS
        handler = sym handle_syscall,
        sp = const(offset_of!(ProcessorControlRegion, user_rsp)),
        ksp = const(offset_of!(ProcessorControlRegion, kernel_rsp)),
    );
}

pub unsafe extern "C" fn handle_syscall(registers: *mut Scratch) {
    let scratch = &(*registers);
    let (syscall_number, arg1, arg2, arg3, _arg4, _arg5, _arg6) = (
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
        SYS_GETPID => sys_getpid(),
        SYS_LSEEK => sys_lseek(arg1, arg2, arg3),
        SYS_BRK => sys_brk(arg1),
        SYS_KILL => sys_kill(arg1),
        _ => {
            error!("Invalid syscall number: {}", syscall_number);
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
    let task = current_task().ok_or(EINTR)?;
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
    let task = current_task().ok_or(EINTR)?;
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

fn sys_getpid() -> SyscallResult {
    let task = current_task().ok_or(EINTR)?;
    debug!("Current PID: {}", task.pid);

    Ok(task.pid.as_usize())
}

fn sys_lseek(descriptor_id: usize, offset: usize, whence: usize) -> SyscallResult {
    let task = current_task().ok_or(EINTR)?;
    let fd = task
        .fds
        .iter()
        .find(|desc| desc.id == FileDescriptorId(descriptor_id))
        .expect("ERROR: FD NOT FOUND IN TASK");
    let ctx = CallerContext {
        pid: task.pid,
        scheme: fd.scheme,
    };
    let schemes = schemes();
    let scheme = schemes.get(fd.scheme).expect("ERROR: SCHEME NO REGISTERED");
    info!("Seeking in fd: {:?}", fd);
    scheme.lseek(fd.id, offset, whence.into(), ctx)
}

fn sys_brk(increment: usize) -> SyscallResult {
    let task = current_task_mut().ok_or(EINTR)?;

    // For now we only support incrementing brk once
    if task.memory_descriptor.brk != 0 {
        return Err(EINVAL);
    }

    let brk_start = VirtualAddress::new(0x6000_0000 + (increment * task.pid.as_usize()));
    let phys = PMM
        .lock()
        .allocate_contiguous(increment)
        .map_err(|_| ENOMEM)?;
    VMM.lock()
        .map_range(
            brk_start,
            phys,
            increment,
            PageFlags::WRITABLE | PageFlags::USER_ACCESSIBLE | PageFlags::PRESENT,
        )
        .map_err(|_| ENOMEM)?;

    let new_brk = task.memory_descriptor.brk + increment as u64;
    task.memory_descriptor.brk = new_brk;

    Ok(brk_start.as_usize())
}

fn sys_kill(pid: usize) -> SyscallResult {
    let pid = Pid::new(pid);
    match scheduler::get_task(pid) {
        Some(task) => {
            if task.pid == current_pid().unwrap() {
                error!("ERROR: Cannot kill self");
                return Err(EINVAL);
            }

            let found = remove_task(pid);

            if !found {
                return Ok(0);
            }

            for fd in task.fds.iter() {
                let schemes = schemes();
                let scheme = schemes.get(fd.scheme).expect("ERROR: SCHEME NO REGISTERED");
                let current_pid = current_pid().expect("ERROR: NO CURRENT PID");
                scheme
                    .close(
                        fd.id,
                        CallerContext {
                            pid: current_pid,
                            scheme: fd.scheme,
                        },
                    )
                    .unwrap();
            }

            Ok(1)
        }
        None => {
            error!("ERROR: PID {} NOT FOUND", pid);
            Err(ESRCH)
        }
    }
}
