#![no_std]
#![no_main]

use core::arch::asm;

use jon_common::{ExitCode, module_entrypoint, println};

module_entrypoint!(
    "terminal",
    "A simple terminal driver to serve as a point of interaction with the user.",
    "1.0.0",
    main
);

fn main() -> Result<(), ExitCode> {
    // let path = "vga:fb0";

    loop {
        println!("{:#x?}", read_rdi());
    }
    Ok(())
}

fn read_rdi() -> usize {
    let rdi_value: usize;
    unsafe {
        asm!(
            "mov {}, rdi", // Move the value of rdi into the rdi_value variable
            out(reg) rdi_value, // Output the value of rdi to the rdi_value variable
        );
    }
    rdi_value
}
