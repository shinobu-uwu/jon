#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, naked_functions, fn_align)]

mod arch;
mod memory;
mod output;
mod sched;
mod syscall;

use core::arch::asm;

use arch::x86::interrupts::LAPIC;
use limine::request::{RequestsEndMarker, RequestsStartMarker, SmpRequest};
use limine::BaseRevision;
use log::{debug, error, info};
use output::logger;
use sched::task::Task;
use sched::TASKS;
use x86_64::instructions::interrupts::{disable, enable, software_interrupt};

/// Sets the base revision to the latest revision supported by the crate.
/// See specification for further info.
/// Be sure to mark all limine requests with #[used], otherwise they may be removed by the compiler.
#[used]
// The .requests section allows limine to find the requests faster and more safely.
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[link_section = ".requests"]
static SMP_REQUEST: SmpRequest = SmpRequest::new();

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
    syscall::init();

    disable();

    let task = Task::new(include_bytes!("./bin/test"));
    TASKS.insert(task.pid, task);
    let task = Task::new(include_bytes!("./bin/test"));
    TASKS.insert(task.pid, task);

    enable();

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
