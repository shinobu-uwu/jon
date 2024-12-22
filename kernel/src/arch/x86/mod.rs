pub mod gdt;
pub mod idt;
pub mod memory;

pub(super) fn init() {
    x86_64::instructions::interrupts::enable();
    gdt::init();
    idt::init();
    memory::init();
}
