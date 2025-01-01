#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod memory;
mod output;
mod sched;

use core::arch::asm;

use arch::x86::interrupts::LAPIC;
use limine::request::{RequestsEndMarker, RequestsStartMarker};
use limine::BaseRevision;
use log::{debug, error};
use output::logger;
use sched::task::Task;
use sched::SCHEDULER;

/// Sets the base revision to the latest revision supported by the crate.
/// See specification for further info.
/// Be sure to mark all limine requests with #[used], otherwise they may be removed by the compiler.
#[used]
// The .requests section allows limine to find the requests faster and more safely.
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

/// Define the stand and end markers for Limine requests.
#[used]
#[link_section = ".requests_start_marker"]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[link_section = ".requests_end_marker"]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

extern crate alloc;

#[no_mangle]
unsafe extern "C" fn kmain() -> ! {
    assert!(BASE_REVISION.is_supported());
    logger::init().unwrap();
    arch::init();
    let mut scheduler = SCHEDULER.lock();
    let task1 = Task::new(None, || {
        debug!("Task 1 first time");
        debug!("Task 1 second time");
        debug!("Task 1 third");
    });
    let task2 = Task::new(None, || {
        debug!("Task 2 first time");
        debug!("Task 2 second time");
        debug!("Task 2 third");
    });
    let task3 = Task::new(None, || {
        debug!("Task 3 first time");
        debug!("Task 3 second time");
        debug!("Task 3 third");
    });
    scheduler.add_task(task1);
    scheduler.add_task(task2);
    scheduler.add_task(task3);
    drop(scheduler);

    hcf();
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    // TODO Move this to x86 specific panic_impl
    unsafe {
        LAPIC.lock().as_mut().unwrap().disable();
    }
    hcf();
}

pub fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            asm!("wfi");
            #[cfg(target_arch = "loongarch64")]
            asm!("idle 0");
        }
    }
}
