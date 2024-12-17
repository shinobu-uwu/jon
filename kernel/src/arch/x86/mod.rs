pub mod gdt;
pub mod idt;
pub mod mm;

pub fn init() {
    gdt::init();
    idt::init();
    x86_64::instructions::interrupts::enable();
}
