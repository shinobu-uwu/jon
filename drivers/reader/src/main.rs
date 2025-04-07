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

fn main(read_pipe: usize, _write_pipe: usize) -> Result<(), ExitCode> {
    let serial = open("serial:", 0x2).unwrap();

    loop {
        message(read_pipe);
        // match read(read_pipe, &mut buf) {
        //     Ok(_) => {
        //         if buf[0] == 69 {
        //             write(serial, b"Nice").unwrap();
        //         }
        //     }
        //     Err(e) => {
        //         write(serial, b"Error reading from pipe").unwrap();
        //     }
        // }
    }
}

fn message(read_pipe: usize) {
    let mut buf = [0u8; 1024];

    match read(read_pipe, &mut buf) {
        Ok(bytes_read) => {
            let _message = &buf[..bytes_read];
        }
        Err(_) => {}
    }
}
