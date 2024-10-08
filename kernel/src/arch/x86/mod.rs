use x86_64::instructions::hlt;

pub mod gdt;
pub mod idt;
pub mod memory;

pub fn init() {
    idt::init();
    gdt::init();
}

pub fn hlt_loop() -> ! {
    loop {
        hlt();
    }
}
