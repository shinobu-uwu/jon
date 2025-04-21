#![no_std]
#![no_main]

mod shift;

use jon_common::{daemon::Daemon, ipc::Message};
use shift::XorShift64;
use spinning_top::Spinlock;

static RNG: Spinlock<XorShift64> = Spinlock::new(XorShift64::new());

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let daemon = Daemon::new(main);
    daemon.register("random").unwrap();
    daemon.start();
}

fn main(_daemon: &Daemon, message: Message) -> Result<usize, i32> {
    match message {
        _ => Ok(RNG.lock().next_u64() as usize),
    }
}
