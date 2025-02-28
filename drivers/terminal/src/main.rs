#![no_std]
#![no_main]

use jon_common::{ExitCode, module_entrypoint, println, syscall};

module_entrypoint!(
    "terminal",
    "A simple terminal driver to serve as a point of interaction with the user.",
    "1.0.0",
    main
);

fn main() -> Result<(), ExitCode> {
    // let path = "vga:fb0";
    // println!("Opening: {:?}", path);
    // let fd = unsafe { syscall(2, path.as_ptr() as usize, path.len(), 0, 0, 0, 0) };
    // println!("Opened: {:?}", fd);
    // let buf = [2u8; 1024];
    // unsafe {
    //     syscall(3, fd as usize, buf.as_ptr() as usize, buf.len(), 0, 0, 0);
    // }
    loop {
        println!("Read");
    }
    Ok(())
}
