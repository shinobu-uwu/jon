#![no_std]
#![no_main]

use core::ffi::CStr;

use heapless::FnvIndexMap;
use jon_common::{ExitCode, daemon::Daemon, ipc::Message};
use spinning_top::Spinlock;

static NAMES: Spinlock<FnvIndexMap<&str, usize, 8>> = Spinlock::new(FnvIndexMap::new());

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let daemon = Daemon::new(main);
    daemon.start();
}

fn main(daemon: &Daemon, message: Message) -> Result<usize, i32> {
    match message.message_type {
        jon_common::ipc::MessageType::Read => {
            let name_buffer = unsafe { core::slice::from_raw_parts(message.data as *const u8, 16) };
            let driver_name = CStr::from_bytes_until_nul(name_buffer)
                .unwrap()
                .to_str()
                .unwrap();

            if driver_name == "exit" {
                daemon.exit(ExitCode(1))
            }

            match NAMES.lock().get(driver_name) {
                Some(pid) => Ok(*pid),
                None => Err(-2),
            }
        }
        jon_common::ipc::MessageType::Write => {
            let name_buffer = unsafe { core::slice::from_raw_parts(message.data as *const u8, 16) };
            let driver_name = CStr::from_bytes_until_nul(name_buffer)
                .unwrap()
                .to_str()
                .unwrap();

            let mut names = NAMES.lock();

            if names.get(driver_name).is_some() {
                return Err(-2); // EEXIST
            }

            names.insert(driver_name, message.origin).unwrap();

            Ok(0)
        }
        jon_common::ipc::MessageType::Delete => todo!(),
        // heartbeats are handled by the daemon itself
        jon_common::ipc::MessageType::Heartbeat => unreachable!(),
    }
}
