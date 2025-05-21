#![no_std]
#![no_main]

use core::fmt::{Arguments, Write};
use core::mem::size_of;
use heapless::String;
use jon_common::{
    daemon::Daemon,
    ipc::Message,
    syscall::fs::{open, read, write},
};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut count = 0;
    let (random_read_fd, random_write_fd, serial_fd) = init();

    loop {
        if count <= 100000000 {
            count += 1;
            continue;
        }

        count = 0;
        log(serial_fd, format_args!("Getting random number"));
        let n = get_random_number(random_read_fd, random_write_fd);
        let mut message = String::<512>::new();
        write!(message, "Random number: {}\n", n).unwrap();
        write(serial_fd, message.as_bytes()).unwrap();
    }
}

fn init() -> (usize, usize, usize) {
    let serial_fd = open("serial:", 0x0).unwrap();
    let random_pid = {
        let daemon = Daemon::new(|_daemon, _message| Ok(0));
        daemon.get_daemon_pid("random").unwrap()
    };
    log(serial_fd, format_args!("Random PID: {}", random_pid));
    let mut random_path: String<16> = String::new();
    write!(random_path, "pipe:{}/read", random_pid).unwrap();
    log(serial_fd, format_args!("Random read path: {}", random_path));
    let random_read_fd = open(&random_path, 0x2).unwrap();
    let mut random_path: String<16> = String::new();
    write!(random_path, "pipe:{}/write", random_pid).unwrap();
    log(
        serial_fd,
        format_args!("Random write path: {}", random_path),
    );
    let random_write_fd = open(&random_path, 0x1).unwrap();

    (random_read_fd, random_write_fd, serial_fd)
}

fn get_random_number(random_read_fd: usize, random_write_fd: usize) -> usize {
    let msg = Message::new(jon_common::ipc::MessageType::Read, [0; 16]);
    let mut buf = [0; size_of::<usize>()];
    write(random_read_fd, msg.to_bytes()).unwrap();
    let mut result = read(random_write_fd, &mut buf);

    while let Err(err) = result {
        if err == 11 {
            // EAGAIN: no data yet, try again
            result = read(random_write_fd, &mut buf);
            continue;
        }
    }

    usize::from_ne_bytes(buf)
}

fn log(serial_fd: usize, args: Arguments) {
    let mut message = String::<128>::new();
    write!(message, "{}", args).unwrap();
    write(serial_fd, message.as_bytes()).unwrap();
}
