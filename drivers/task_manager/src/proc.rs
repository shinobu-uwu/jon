use alloc::{format, vec::Vec};
use jon_common::{
    ipc::Message,
    syscall::{
        fs::{open, read, write},
        task::kill,
    },
};

use crate::{SERIAL_FD, log};

#[repr(C)]
#[derive(Debug)]
pub struct Proc {
    pub pid: usize,
    pub name: [u8; 16],
    pub state: State,
    pub priority: Priority,
}

impl Proc {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        unsafe { core::ptr::read(bytes.as_ptr() as *const Proc) }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Low,
    Normal,
    High,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running,
    Blocked,
    Waiting,
    Stopped,
}

pub fn list_procs(proc_fd: usize) -> Vec<Proc> {
    let mut buf = [0u8; 128 * size_of::<Proc>()];
    let bytes_read = read(proc_fd, &mut buf).unwrap();
    let procs_buf = &buf[..bytes_read];

    procs_buf
        .windows(size_of::<Proc>())
        .step_by(size_of::<Proc>())
        .map(|bytes| Proc::from_bytes(bytes))
        .collect()
}

pub fn kill_proc(proc: &Proc) {
    if proc.state != State::Running && proc.state != State::Waiting {
        write(*SERIAL_FD.lock(), b"Task not running, cannot kill").unwrap();
        return;
    }

    log("Attempting to kill task...");

    match kill(proc.pid) {
        Ok(f) => {
            let found = f != 0;
            write(
                *SERIAL_FD.lock(),
                format!("Task killed: {}", found).as_bytes(),
            )
            .unwrap();
        }
        Err(e) => {
            write(
                *SERIAL_FD.lock(),
                format!("Error killing task: {}", e).as_bytes(),
            )
            .unwrap();
        }
    }

    log("Sending kill message...");
    let fd = open("pipe:1/read", 0x2).unwrap();
    log("Writing to pipe...");
    let mut pid_buf = [0u8; 16];
    pid_buf[..8].copy_from_slice(&proc.pid.to_ne_bytes());
    write(
        fd,
        Message::new(jon_common::ipc::MessageType::Delete, pid_buf).to_bytes(),
    )
    .unwrap();
    let fd = open("pipe:1/write", 0x1).unwrap();
    let mut result = read(fd, &mut [0u8; 8]);

    while let Err(err) = result {
        if err == 11 {
            // EAGAIN: no data yet, try again
            result = read(fd, &mut [0u8; 8]);
            continue;
        }
    }
    log("Message sent.");
}
