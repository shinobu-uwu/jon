pub mod gdt;
pub mod idt;

pub fn init() {
    idt::init();
    gdt::init();
    x86_64::instructions::interrupts::enable();
}
