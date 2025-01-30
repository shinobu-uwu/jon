#![no_std]
#![no_main]

use core::arch::asm;

use jon_common::{module_entrypoint, println, ExitCode};

module_entrypoint!("printmod", "A simple kernel module", "1.0.0", main);

fn main() -> Result<(), ExitCode> {
    println!("Hello, world!");

    loop {}

    Ok(())
}
