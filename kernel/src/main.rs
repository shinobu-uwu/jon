#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod memory;
mod output;
mod paging;

use core::panic::PanicInfo;

use alloc::boxed::Box;
use arch::x86::hlt_loop;
use bootloader_api::BootInfo;
use output::console::init_console;

extern crate alloc;
extern crate core;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(f) = boot_info.framebuffer.as_mut() {
        init_console(f.info(), f.buffer_mut())
    }
    #[cfg(target_arch = "x86_64")]
    arch::x86::init();

    let a = Box::new(41);
    println!("It did not crash!, a = {}", a);

    #[cfg(target_arch = "x86_64")]
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
