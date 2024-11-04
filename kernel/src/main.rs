#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod output;

use core::arch::asm;

use arch::x86::gdt::GDT;
use arch::x86::idt::IDT;
use limine::request::{
    FramebufferRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker, RsdpRequest,
    SmpRequest,
};
use limine::BaseRevision;
use log::{debug, error};
use output::logger;
use x86_64::instructions::interrupts::int3;
use x86_64::structures::DescriptorTablePointer;
use x86_64::VirtAddr;

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

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[link_section = ".requests"]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

#[used]
#[link_section = ".requests"]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

/// Define the stand and end markers for Limine requests.
#[used]
#[link_section = ".requests_start_marker"]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[link_section = ".requests_end_marker"]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[no_mangle]
unsafe extern "C" fn kmain() -> ! {
    assert!(BASE_REVISION.is_supported());
    logger::init().unwrap();
    arch::init();
    debug!("{:#?}", GDT.0);
    debug!("{:#?}", read_idt());
    int3();

    hcf();
}

pub fn read_idt() -> DescriptorTablePointer {
    let mut idt: DescriptorTablePointer = DescriptorTablePointer {
        base: VirtAddr::new(0),
        limit: 0,
    };

    unsafe {
        asm!("sidt [{}]", in(reg) &mut idt);
    }

    idt
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
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
