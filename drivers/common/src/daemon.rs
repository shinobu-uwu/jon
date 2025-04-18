use core::ffi::CStr;

use crate::{
    exit,
    ipc::Message,
    syscall::{self, fs::read},
    ExitCode,
};

pub struct Daemon {
    serial: usize,
    read_pipe: usize,
    write_pipe: usize,
    callback: fn(&Self, Message) -> Result<usize, i32>,
}

impl Daemon {
    pub fn new(callback: fn(&Self, Message) -> Result<usize, i32>) -> Self {
        let serial = syscall::fs::open("serial:", 0x0).unwrap();
        syscall::fs::write(serial, b"Creating daemon").unwrap();
        let read_pipe = syscall::fs::open("pipe:read", 0o100 | 0x1).unwrap();
        let write_pipe = syscall::fs::open("pipe:write", 0o100 | 0x2).unwrap();
        syscall::fs::write(serial, b"Daemon created").unwrap();

        Self {
            serial,
            read_pipe,
            write_pipe,
            callback,
        }
    }

    pub fn run_once<F2: FnOnce()>(&self, callback: F2) {
        callback();
    }

    /// XXX: This should be at max 15 characters long, as reincarnation will use a 16 bytes
    /// buffer and will expect the last byte to be null
    pub fn register(&self, name: &str) {
        let mut name_buf = [0u8; 16];
        let bytes = name.as_bytes();
        let len = bytes.len().min(15);
        name_buf[..len].copy_from_slice(bytes);
        name_buf[len] = 0;
        let reincarnation_pipe = syscall::fs::open("pipe:2/read", 0x1).unwrap();
        syscall::fs::write(reincarnation_pipe, &name_buf).unwrap();
    }

    pub fn start(&self) -> ! {
        loop {
            let mut buf = [0u8; 1024];
            match read(self.read_pipe, &mut buf) {
                Ok(bytes_read) => {
                    syscall::fs::write(self.serial, b"Received message\n").unwrap();
                    let result_buffer = &buf[..bytes_read];
                    let message = Message::from_bytes(result_buffer);
                    match (self.callback)(self, message) {
                        Ok(_) => todo!(),
                        Err(_) => todo!(),
                    }
                }
                Err(errno) => {
                    if errno == 11 {
                        // EAGAIN
                        syscall::fs::write(self.serial, b"No messages, trying again\n").unwrap();
                        continue;
                    }
                }
            }
        }
    }

    pub fn exit(&self, code: ExitCode) -> ! {
        exit(code)
    }

    pub fn log(&self, message: &str) {
        syscall::fs::write(self.serial, message.as_bytes()).unwrap();
    }
}

fn str_to_cstr<'a>(s: &str, buf: &'a mut [u8; 16]) -> &'a CStr {
    let bytes = s.as_bytes();
    let len = bytes.len().min(15);
    buf[..len].copy_from_slice(bytes);
    buf[len] = 0;

    unsafe { CStr::from_bytes_with_nul_unchecked(&buf[..=len]) }
}
