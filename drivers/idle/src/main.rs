#![no_std]
#![no_main]

use jon_common::{
    ExitCode, module_entrypoint,
    syscall::fs::{open, read, write},
};
module_entrypoint!("idle", "idle task", "1.0.0", main);

fn main() -> Result<(), ExitCode> {
    loop {}

    Ok(())
}
