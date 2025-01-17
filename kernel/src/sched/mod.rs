use alloc::collections::btree_map::BTreeMap;
use bitmap_allocator::{BitAlloc, BitAlloc64K};
use lazy_static::lazy_static;
use log::debug;
use pid::Pid;
use spinning_top::Spinlock;
use task::Task;
use x86_64::structures::idt::InterruptStackFrame;

use crate::arch::end_of_interrupt;

pub mod pid;
pub mod task;

lazy_static! {
    pub static ref PID_ALLOCATOR: Spinlock<BitAlloc64K> = {
        let mut bitmap = BitAlloc64K::default();
        bitmap.insert(0..BitAlloc64K::CAP); // marks all bits as available
        bitmap.remove(0..1); // marks PID 0 as used, for the kernel

        Spinlock::new(bitmap)
    };
}

pub static mut TASKS: BTreeMap<Pid, Task> = BTreeMap::new();
pub static mut CURRENT_PID: Option<Pid> = None;
static mut CONTEXT_SWITCH_LOCK: bool = false;
static mut TICKS: usize = 0;

pub unsafe fn tick() {
    TICKS += 1;

    if TICKS % 10 != 0 {
        return;
    }

    if TASKS.is_empty() {
        debug!("No tasks to switch to");
        return;
    }

    match CURRENT_PID {
        Some(pid) => {
            if TASKS.len() < 2 {
                debug!("Only one task to switch to");
                return;
            }

            let current_task = TASKS.get(&pid).unwrap();
            let next_pid = TASKS.keys().find(|&&pid| pid != current_task.pid).unwrap();
            CURRENT_PID.replace(*next_pid);
            debug!("Switching tasks");
            switch_to(*next_pid);
        }
        None => {
            let next_pid = TASKS.keys().next().unwrap();
            let next_task = TASKS.get(next_pid).unwrap();
            CURRENT_PID.replace(*next_pid);
            next_task.restore();
        }
    }
}

pub unsafe fn switch_to(next_task: Pid) {
    CONTEXT_SWITCH_LOCK = true;
    if let Some(prev_pid) = CURRENT_PID {
        let prev_task = TASKS.get_mut(&prev_pid).unwrap();
        prev_task.save();
    }

    let next_task = TASKS.get(&next_task).unwrap();
    CONTEXT_SWITCH_LOCK = false;
    end_of_interrupt();
    next_task.restore();
}
