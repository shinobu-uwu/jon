use x86::{interrupts::LAPIC, structures::Registers};
use x86_64::instructions::interrupts::disable;

use crate::sched::task::Task;

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

pub unsafe fn switch_to(prev: Option<&mut Task>, next: &Task, current_stack_frame: &Registers) {
    #[cfg(target_arch = "x86_64")]
    x86::switch_to(prev, next, current_stack_frame);
}

pub fn panic(_info: &core::panic::PanicInfo) {
    disable();
}
