use core::arch::asm;

use interrupt::PICS;
use x86_64::instructions::hlt;

pub mod gdt;
pub mod idt;
pub mod interrupt;
pub mod memory;

pub fn init() {
    idt::init();
    gdt::init();
    let mut pics = PICS.lock();
    unsafe { pics.initialize() };
    x86_64::instructions::interrupts::enable();
    unsafe { asm!("sti") }
}

pub fn hlt_loop() -> ! {
    loop {
        hlt();
    }
}
