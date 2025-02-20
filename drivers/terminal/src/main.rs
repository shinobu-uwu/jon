#![no_std]
#![no_main]

use jon_common::{ExitCode, module_entrypoint};

module_entrypoint!(
    "terminal",
    "A simple terminal driver to serve as a point of interaction with the user.",
    "1.0.0",
    main
);

fn main() -> Result<(), ExitCode> {
    Ok(())
}
