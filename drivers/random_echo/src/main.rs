#![no_std]
#![no_main]
#![feature(let_chains)]
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
    let (mut random_read_fd, mut random_write_fd, serial_fd) = init();
    loop {
        if count <= 100000000 {
            count += 1;
            continue;
        }
        count = 0;
        log(serial_fd, format_args!("Getting random number"));
        let n = match get_random_number(random_read_fd, random_write_fd, serial_fd) {
            Ok(n) => n,
            Err(RandomError::DaemonNotAvailable) => {
                log(
                    serial_fd,
                    format_args!("Daemon not available, reconnecting..."),
                );
                let random_pid = get_random_pid(serial_fd);
                let mut random_path: String<16> = String::new();
                write!(random_path, "pipe:{}/read", random_pid).unwrap();
                log(serial_fd, format_args!("Random read path: {}", random_path));
                random_read_fd = open(&random_path, 0x2).unwrap();
                let mut random_path: String<16> = String::new();
                write!(random_path, "pipe:{}/write", random_pid).unwrap();
                random_write_fd = open(&random_path, 0x1).unwrap();
                log(serial_fd, format_args!("Reconnected to new daemon"));
                continue;
            }
        };
        let mut message = String::<512>::new();
        write!(message, "Random number: {}\n", n).unwrap();
        write(serial_fd, message.as_bytes()).unwrap();
    }
}
fn init() -> (usize, usize, usize) {
    let serial_fd = open("serial:", 0x1).unwrap();
    let random_pid = get_random_pid(serial_fd);
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
fn get_random_pid(serial_fd: usize) -> usize {
    let daemon = Daemon::new(|_daemon, _message| Ok(0));
    let mut pid = daemon.get_daemon_pid("random");
    log(serial_fd, format_args!("Daemon PID: {:?}", pid));

    while pid.is_none() {
        let mut delay_counter = 0;
        while delay_counter < 100000 {
            delay_counter += 1;
        }
        pid = daemon.get_daemon_pid("random");
    }

    pid.unwrap()
}

fn get_random_number(
    random_read_fd: usize,
    random_write_fd: usize,
    serial_fd: usize,
) -> Result<usize, RandomError> {
    let msg = Message::new(jon_common::ipc::MessageType::Read, [0; 16]);
    let mut buf = [0; size_of::<usize>()];

    // Try to write the message
    if let Err(e) = write(random_read_fd, msg.to_bytes()) {
        log(serial_fd, format_args!("Write failed with error: {}", e));
        if e == 9 {
            return Err(RandomError::DaemonNotAvailable);
        }
    }

    // Read with daemon death detection
    let mut result = read(random_write_fd, &mut buf);
    let mut read_attempts = 0;

    while let Err(err) = result {
        if err == 11 {
            // EAGAIN: no data yet, try again
            read_attempts += 1;

            // Periodically check if daemon is still alive by trying a small write
            if read_attempts % 50000 == 0 {
                log(
                    serial_fd,
                    format_args!("Still waiting for response, attempt: {}", read_attempts),
                );
                // Try to detect if daemon died by attempting another write
                if let Err(write_err) = write(random_read_fd, msg.to_bytes()) {
                    if write_err == 9 {
                        log(serial_fd, format_args!("Daemon died while reading"));
                        return Err(RandomError::DaemonNotAvailable);
                    }
                }
            }

            result = read(random_write_fd, &mut buf);
            continue;
        }

        if err == 9 {
            // EBADF: bad file descriptor
            log(serial_fd, format_args!("Read failed with EBADF"));
            return Err(RandomError::DaemonNotAvailable);
        }
        // Handle other errors
        log(
            serial_fd,
            format_args!("Read failed with unexpected error: {}", err),
        );
        return Err(RandomError::DaemonNotAvailable);
    }

    log(serial_fd, format_args!("Successfully read random number"));
    Ok(usize::from_ne_bytes(buf))
}

fn log(serial_fd: usize, args: Arguments) {
    let mut message = String::<128>::new();
    write!(message, "{}", args).unwrap();
    write(serial_fd, message.as_bytes()).unwrap();
}

enum RandomError {
    DaemonNotAvailable,
}
