#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod output;
mod paging;

use core::panic::PanicInfo;

use arch::x86::hlt_loop;
use bootloader_api::BootInfo;
use output::console::init_console;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(f) = boot_info.framebuffer.as_mut() {
        init_console(f.info(), f.buffer_mut())
    }
    #[cfg(target_arch = "x86_64")]
    arch::x86::init();

    fn stack_overflow() -> ! {
        stack_overflow();
    }

    stack_overflow();

    println!("It did not crash!");

    #[cfg(target_arch = "x86_64")]
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
}
