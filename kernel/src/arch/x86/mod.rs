pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod memory;

pub fn init() {
    gdt::init();
    idt::init();
    memory::allocator::init();
    x86_64::instructions::interrupts::enable();
    interrupts::init();
}
