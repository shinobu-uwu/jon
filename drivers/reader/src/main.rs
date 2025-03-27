#![no_std]
#![no_main]

use jon_common::{
    ExitCode, module_entrypoint,
    syscall::fs::{open, read, write},
};
module_entrypoint!(
    "reader",
    "A simple terminal driver to serve as a point of interaction with the user.",
    "1.0.0",
    main
);

fn main() -> Result<(), ExitCode> {
    let fd = open("pipe:abc", 1);
    let mut buffer = [0; 128];
    read(fd, &mut buffer);

    let fd = open("serial:", 0);
    write(fd, &buffer);
    loop {}

    Ok(())
}
