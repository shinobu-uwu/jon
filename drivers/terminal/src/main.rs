#![no_std]
#![no_main]

use core::arch::asm;

use jon_common::{ExitCode, module_entrypoint, syscall::fs::open};
use jon_common::{println, syscall};

module_entrypoint!(
    "terminal",
    "A simple terminal driver to serve as a point of interaction with the user.",
    "1.0.0",
    main
);

#[allow(named_asm_labels)]
fn main() -> Result<(), ExitCode> {
    let fd = open("vga:fb0");
    unsafe {
        asm!("lea rdi, [0x10a2 + rip]");
        asm!("debug_label:");
    }
    println!("{fd}");
    // unsafe {
    //     syscall(1, args.as_ptr() as usize, args.len(), 0, 0, 0, 0);
    // }
    loop {}

    Ok(())
}
