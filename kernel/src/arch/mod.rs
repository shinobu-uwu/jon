use x86::interrupts::LAPIC;

#[cfg(target_arch = "x86_64")]
pub mod x86;

#[cfg(target_arch = "x86_64")]
pub type InterruptStackFrame = x86_64::structures::idt::InterruptStackFrame;

pub fn init() {
    #[cfg(target_arch = "x86_64")]
    x86::init();
}

pub unsafe fn end_of_interrupt() {
    #[cfg(target_arch = "x86_64")]
    LAPIC.lock().as_mut().unwrap().end_of_interrupt();
}
