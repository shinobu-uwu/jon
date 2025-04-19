#![no_std]
#![no_main]

use jon_common::{daemon::Daemon, ipc::Message};

pub extern "C" fn _start() -> ! {
    let daemon = Daemon::new(main);
    daemon.register("render");
    daemon.start();
}

pub fn main(daemon: &Daemon, message: Message) -> Result<usize, i32> {
    todo!()
}
