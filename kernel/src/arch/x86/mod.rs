pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod memory;
pub mod sched;

pub fn init() {
    gdt::init();
    idt::init();
    memory::allocator::init();
    interrupts::init();
}
