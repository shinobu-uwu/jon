#![no_std]
#![no_main]

use jon_common::{ExitCode, module_entrypoint};

module_entrypoint!("reincarnation", "Reincarnation server", "0.1.0", main);

static NAMES: [Option<(&str, usize)>; 64] = [None; 64];

fn main(_buf: &[u8]) -> Result<(), ExitCode> {
    Ok(())
}
