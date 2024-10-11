#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod memory;
mod output;

use core::panic::PanicInfo;

use arch::x86::{hlt_loop, memory::BootInfoFrameAllocator};
use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};
use log::error;
use output::console::init_console;
use x86_64::VirtAddr;

use crate::output::logger;

extern crate alloc;
extern crate core;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    logger::init().expect("Failed to init logger");

    if let Some(f) = boot_info.framebuffer.as_mut() {
        init_console(f.info(), f.buffer_mut());
    }

    #[cfg(target_arch = "x86_64")]
    arch::x86::init();

    let offset = boot_info.physical_memory_offset.into_option().unwrap();
    let phys_mem_offset = VirtAddr::new(offset);
    let mut mapper = unsafe { arch::x86::memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    crate::memory::allocator::init_heap(&mut mapper, &mut frame_allocator).unwrap();

    println!("Hello, world!");

    #[cfg(target_arch = "x86_64")]
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    #[cfg(target_arch = "x86_64")]
    hlt_loop();
}
