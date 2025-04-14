#![no_std]
#![no_main]

use jon_common::{ExitCode, daemon_entrypoint};

daemon_entrypoint!("reincarnation", "Reincarnation server", "0.1.0", main);

static NAMES: [Option<(&str, usize)>; 8] = [None; 8];

fn main() -> Result<&'static str, ExitCode> {
    Ok("Hello, from reincarnation!")
}
