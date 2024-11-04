pub mod gdt;
pub mod idt;

pub fn init() {
    gdt::init();
    idt::init();
    x86_64::instructions::interrupts::enable();
}
