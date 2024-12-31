use log::debug;
use spinning_top::Spinlock;
use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder};

use crate::memory::paging::phys_to_virt;

pub static LAPIC: Spinlock<Option<LocalApic>> = Spinlock::new(None);
pub const TIMER_VECTOR: usize = 32;
pub const ERROR_VECTOR: usize = TIMER_VECTOR + 1;
pub const SPURIOUS_VECTOR: usize = ERROR_VECTOR + 1;

pub(super) fn init() {
    let phys_lapic = unsafe { xapic_base() };
    let virt_lapic = phys_to_virt(phys_lapic as usize);

    debug!(
        "Initializing LAPIC - Physical: {:#x}, Virtual: {:#x}",
        phys_lapic, virt_lapic
    );

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(TIMER_VECTOR)
        .error_vector(ERROR_VECTOR)
        .spurious_vector(SPURIOUS_VECTOR)
        .set_xapic_base(virt_lapic as u64)
        .build()
        .expect("Failed to build LAPIC");

    unsafe {
        debug!("Enabling LAPIC");
        lapic.enable();
        debug!("LAPIC enabled");
    }

    debug!("Storing LAPIC instance");
    *LAPIC.lock() = Some(lapic);
    debug!("LAPIC initialization complete");
}
