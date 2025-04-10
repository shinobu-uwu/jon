#![no_std]
#![no_main]

use jon_common::{ExitCode, module_entrypoint, syscall::fs::write};
module_entrypoint!(
    "reader",
    "A simple terminal driver to serve as a point of interaction with the user.",
    "1.0.0",
    main
);

fn main(read_pipe: usize, write_pipe: usize) -> Result<(), ExitCode> {
    write(write_pipe, b"Hello from the writer task!\n").unwrap();

    loop {}
}
