#![no_std]
#![no_main]

use core::ffi::CStr;

use heapless::{FnvIndexMap, String};
use jon_common::{ExitCode, daemon::Daemon, ipc::Message};
use spinning_top::Spinlock;

static NAMES: Spinlock<FnvIndexMap<String<16>, usize, 8>> = Spinlock::new(FnvIndexMap::new());

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let daemon = Daemon::new(main);
    daemon.start();
}

fn main(daemon: &Daemon, message: Message) -> Result<usize, i32> {
    match message.message_type {
        jon_common::ipc::MessageType::Read => {
            let daemon_name = CStr::from_bytes_until_nul(&message.data)
                .unwrap()
                .to_str()
                .unwrap();
            daemon.log(format_args!("Getting daemon {}", daemon_name));

            if daemon_name == "exit" {
                daemon.exit(ExitCode(1))
            }
            let mut name = String::<16>::new();
            name.push_str(daemon_name).unwrap();

            match NAMES.lock().get(&name) {
                Some(pid) => {
                    daemon.log(format_args!(
                        "Found daemon {} with pid {}",
                        daemon_name, pid
                    ));
                    Ok(*pid)
                }
                None => {
                    daemon.log(format_args!("Daemon {} not found", daemon_name));
                    Err(-2) // ENOENT
                }
            }
        }
        jon_common::ipc::MessageType::Write => {
            daemon.log(format_args!("Registering daemon"));
            let daemon_name = CStr::from_bytes_until_nul(&message.data)
                .unwrap()
                .to_str()
                .unwrap();
            let mut name = String::<16>::new();
            name.push_str(daemon_name).unwrap();

            let mut names = NAMES.lock();

            if names.get(&name).is_some() {
                daemon.log(format_args!("Daemon {} already registered", daemon_name));
                return Err(-2); // EEXIST
            }

            names.insert(name, message.origin).unwrap();
            daemon.log(format_args!("Registered daemon: {}", daemon_name));

            Ok(0)
        }
        jon_common::ipc::MessageType::Delete => {
            let daemon_name = CStr::from_bytes_until_nul(&message.data)
                .unwrap()
                .to_str()
                .unwrap();
            let mut name = String::<16>::new();
            name.push_str(daemon_name).unwrap();

            let mut names = NAMES.lock();

            if names.remove(&name).is_none() {
                daemon.log(format_args!("Daemon {} not registered", daemon_name));
                return Err(-2); // ENOENT
            }

            daemon.log(format_args!("Unregistered daemon: {}", daemon_name));

            Ok(0)
        }
        // heartbeats are handled by the daemon upstream
        jon_common::ipc::MessageType::Heartbeat => unreachable!(),
    }
}
