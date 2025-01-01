use bitmap_allocator::{BitAlloc, BitAlloc64K};
use lazy_static::lazy_static;
use scheduler::Scheduler;
use spinning_top::Spinlock;

pub mod pid;
pub mod scheduler;
pub mod task;

pub static SCHEDULER: Spinlock<Scheduler> = Spinlock::new(Scheduler::new());
lazy_static! {
    pub static ref PID_ALLOCATOR: Spinlock<BitAlloc64K> = {
        let mut bitmap = BitAlloc64K::default();
        bitmap.insert(0..BitAlloc64K::CAP); // marks all bits as available
        bitmap.remove(0..1); // marks PID 0 as used, for the kernel

        Spinlock::new(bitmap)
    };
}
