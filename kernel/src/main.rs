#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod memory;
mod output;

use core::panic::PanicInfo;

use alloc::boxed::Box;
use arch::x86::{
    hlt_loop,
    memory::{active_level_4_table, translate_addr, BootInfoFrameAllocator},
};
use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};
use log::{debug, error, info};
use memory::allocator::HEAP_START;
use x86_64::{
    registers::debug,
    structures::paging::{PageTable, Translate},
    VirtAddr,
};

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
    #[cfg(target_arch = "x86_64")]
    arch::x86::init();

    let offset = match boot_info.physical_memory_offset {
        bootloader_api::info::Optional::Some(o) => o,
        bootloader_api::info::Optional::None => panic!("Memory not mapped"),
    };
    let phys_mem_offset = VirtAddr::new(offset);
    let mut mapper = unsafe { arch::x86::memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    crate::memory::allocator::init_heap(&mut mapper, &mut frame_allocator).unwrap();

    #[cfg(target_arch = "x86_64")]
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    #[cfg(target_arch = "x86_64")]
    hlt_loop();
}
