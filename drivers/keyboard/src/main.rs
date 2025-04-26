#![no_std]
#![no_main]

use core::fmt::{Arguments, Write};

use heapless::String;
use jon_common::syscall::fs::{open, read, write};
use spinning_top::Spinlock;

static SERIAL_FD: Spinlock<usize> = Spinlock::new(0);

#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let fd = open("ps2:", 0).unwrap();
    *SERIAL_FD.lock() = open("serial:", 0).unwrap();
    let mut buf = [0u8; 1];
    loop {
        match read(fd, &mut buf) {
            Ok(n) => {
                log(format_args!("Read {} bytes: {:?}", n, buf));
            }
            Err(e) => {
                if e == 11 {
                    continue;
                }

                log(format_args!("Error reading from ps2: {}", e));
            }
        }
    }
}

fn log(args: Arguments) {
    let mut message = String::<128>::new();
    write!(message, "{}", args).unwrap();
    write(*SERIAL_FD.lock(), message.as_bytes()).unwrap();
}
