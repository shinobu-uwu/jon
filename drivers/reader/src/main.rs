#![no_std]
#![no_main]

use jon_common::{
    ExitCode, module_entrypoint,
    syscall::fs::{open, read, write},
    usize_to_str,
};
module_entrypoint!(
    "reader",
    "A simple terminal driver to serve as a point of interaction with the user.",
    "1.0.0",
    main
);

fn main(_read_pipe: usize, _write_pipe: usize) -> Result<(), ExitCode> {
    let writer = open("pipe:writer/write", 0).unwrap();
    let serial = open("serial:", 0).unwrap();
    let mut buf = [0u8; 1024];
    read(writer, &mut buf).unwrap();
    write(serial, &buf).unwrap();

    loop {}
}
