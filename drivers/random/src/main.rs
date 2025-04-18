#![no_std]
#![no_main]

use jon_common::{daemon::Daemon, ipc::Message};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let daemon = Daemon::new(main);
    daemon.register("random");
    daemon.start();
}

fn main(_daemon: &Daemon, message: Message) -> Result<usize, i32> {
    match message {
        _ => Ok(10),
    }
}
