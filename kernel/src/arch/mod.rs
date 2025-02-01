use x86::{interrupts::LAPIC, structures::Registers};

#[cfg(target_arch = "x86_64")]
pub mod x86;

pub fn init() {
    #[cfg(target_arch = "x86_64")]
    x86::init();
}

pub unsafe fn end_of_interrupt() {
    #[cfg(target_arch = "x86_64")]
    LAPIC.lock().as_mut().unwrap().end_of_interrupt();
}

pub unsafe fn switch_to(
    prev_context: &mut Registers,
    next_context: &Registers,
    current_stack_frame: &Registers,
) {
    #[cfg(target_arch = "x86_64")]
    x86::switch_to(prev_context, next_context, current_stack_frame);
}
