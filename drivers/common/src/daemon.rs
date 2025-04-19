use core::{
    arch::asm,
    fmt::{Arguments, Write},
};
use heapless::String;

use crate::{
    exit,
    ipc::{Message, MessageType},
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
        self.log(format_args!("Registering daemon {}", name));
        let mut name_buf = [0u8; 16];
        let bytes = name.as_bytes();
        let len = bytes.len().min(15);
        name_buf[..len].copy_from_slice(bytes);
        name_buf[len] = 0;
        let reincarnation_pipe = syscall::fs::open("pipe:2/read", 0x1).unwrap();
        let message = Message::new(MessageType::Write, name_buf);
        syscall::fs::write(reincarnation_pipe, message.to_bytes()).unwrap();
        self.log(format_args!("Registered daemon {}", name));
    }

    pub fn start(&self) -> ! {
        loop {
            let mut buf = [0u8; 1024];
            match read(self.read_pipe, &mut buf) {
                Ok(bytes_read) => {
                    self.log(format_args!("Received message"));
                    let result_buffer = &buf[..bytes_read];
                    self.log(format_args!("Parsing message"));
                    let message = Message::from_bytes(result_buffer);
                    self.log(format_args!("Parsed message"));

                    if let MessageType::Heartbeat = message.message_type {
                        self.log(format_args!("Heartbeat received"));
                        syscall::fs::write(self.write_pipe, &[0x44]).unwrap();
                        continue;
                    }

                    self.log(format_args!("Handling message: {:?}", message));
                    match (self.callback)(self, message) {
                        Ok(n) => {
                            self.log(format_args!("Message handled"));
                            syscall::fs::write(self.write_pipe, &n.to_ne_bytes()).unwrap();
                        }
                        Err(_) => todo!(),
                    }
                }
                Err(errno) => {
                    if errno == 11 {
                        // EAGAIN
                        // self.log(format_args!("No data available, retrying"));
                        continue;
                    }
                }
            }
        }
    }

    pub fn exit(&self, code: ExitCode) -> ! {
        exit(code)
    }

    pub fn log(&self, args: Arguments) {
        let mut message = String::<128>::new();
        write!(message, "{}", args).unwrap();
        syscall::fs::write(self.serial, message.as_bytes()).unwrap();
    }
}
