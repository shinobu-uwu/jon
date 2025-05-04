pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod memory;
pub mod sched;
pub mod structures;

pub fn init() {
    memory::allocator::init();
    cpu::init();
}
