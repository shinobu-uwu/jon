#![no_std]
#![no_main]

use jon_common::{
    ExitCode, module_entrypoint, println,
    syscall::fs::{open, read, write},
};
module_entrypoint!(
    "terminal",
    "A simple terminal driver to serve as a point of interaction with the user.",
    "1.0.0",
    main
);

fn main() -> Result<(), ExitCode> {
    let write_fd = open("pipe:abc", 1);
    let mut buffer = [137u8; 128];
    write(write_fd, &buffer);
    buffer = [0u8; 128];
    let read_fd = open("pipe:abc", 0);
    read(read_fd, &mut buffer);

    let serial_fd = open("serial:", 1);

    loop {
        write(serial_fd, "READ BYTES".as_bytes());
        write(serial_fd, &buffer);
    }

    Ok(())
}
