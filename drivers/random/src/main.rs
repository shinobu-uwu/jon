#![no_std]
#![no_main]

use jon_common::{ExitCode, daemon_entrypoint, ipc::Message};

daemon_entrypoint!("random", "Random number generator", "0.1.0", main);

fn main(_message: Message) -> Result<(), ExitCode> {
    todo!()
}
