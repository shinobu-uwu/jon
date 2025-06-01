#![no_std]
#![no_main]

mod allocator;
extern crate alloc;

use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};
use allocator::init;
use core::ffi::CStr;
use jon_common::{ExitCode, daemon::Daemon, ipc::Message};
use spinning_top::Spinlock;

static NAMES: Spinlock<BTreeMap<String, Vec<usize>>> = Spinlock::new(BTreeMap::new());

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();
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
            let mut name = String::new();
            name.push_str(daemon_name);

            match NAMES.lock().get(&name) {
                Some(pids) => {
                    if pids.is_empty() {
                        daemon.log(format_args!(
                            "Daemon {} has no registered PIDs",
                            daemon_name
                        ));
                        return Err(2); // ENOENT
                    }

                    let pid = pids.first().unwrap();
                    daemon.log(format_args!(
                        "Found daemon {} with pid {}",
                        daemon_name, pid
                    ));
                    Ok(*pid)
                }
                None => {
                    daemon.log(format_args!("Daemon {} not found", daemon_name));
                    Err(2) // ENOENT
                }
            }
        }
        jon_common::ipc::MessageType::Write => {
            daemon.log(format_args!("Registering daemon"));
            let daemon_name = CStr::from_bytes_until_nul(&message.data)
                .unwrap()
                .to_str()
                .unwrap();
            let mut name = String::new();
            name.push_str(daemon_name);

            let mut names = NAMES.lock();
            let pids = match names.get_mut(&name) {
                Some(pids) => pids,
                None => {
                    names.insert(name.clone(), Vec::new());
                    names.get_mut(&name).unwrap()
                }
            };
            pids.push(message.origin);
            daemon.log(format_args!(
                "Registered daemon: {}, pid: {}",
                daemon_name, message.origin
            ));

            Ok(0)
        }
        jon_common::ipc::MessageType::Delete => {
            let mut pid_buf = [0u8; 8];
            pid_buf.copy_from_slice(&message.data[..8]);
            let pid = usize::from_ne_bytes(pid_buf);
            let mut names = NAMES.lock();

            for (name, pids) in names.iter_mut() {
                if let Some(pos) = pids.iter().position(|&x| x == pid) {
                    pids.remove(pos);
                    daemon.log(format_args!("Unregistered daemon: {}, pid: {}", name, pid));

                    // if pids.is_empty() {
                    //     names.remove(name);
                    //     daemon.log(format_args!("Removed empty daemon: {}", name));
                    // }

                    return Ok(0);
                }
            }

            Ok(0)
        }
        // heartbeats are handled by the daemon upstream
        jon_common::ipc::MessageType::Heartbeat => unreachable!(),
    }
}
