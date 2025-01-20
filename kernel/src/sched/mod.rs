use alloc::collections::btree_map::BTreeMap;
use bitmap_allocator::{BitAlloc, BitAlloc64K};
use lazy_static::lazy_static;
use log::debug;
use pid::Pid;
use spinning_top::Spinlock;
use task::Task;

use crate::arch::{end_of_interrupt, InterruptStackFrame};

pub mod memory;
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

pub unsafe fn tick(stack_frame: &InterruptStackFrame) {
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

            let current_task = TASKS
                .get(&pid)
                .expect("Current task should exist in the task list");
            let next_pid = TASKS
                .keys()
                .filter(|&&next_pid| next_pid != current_task.pid)
                .next() // Get the next available PID, skipping the current one
                .expect("There should be another task to switch to");

            debug!(
                "Switching from task {} to task {}",
                current_task.pid, *next_pid
            );

            switch_to(*next_pid, stack_frame);
        }
        None => {
            if let Some(next_pid) = TASKS.keys().next() {
                CURRENT_PID.replace(*next_pid);
                let next_task = TASKS
                    .get(next_pid)
                    .expect("Next task should exist in the task list");
                debug!("Starting the first task: {}", next_pid);
                next_task.restore();
            } else {
                debug!("No task available to start");
            }
        }
    }
}

pub unsafe fn switch_to(next_pid: Pid, stack_frame: &InterruptStackFrame) {
    CONTEXT_SWITCH_LOCK = true;
    if let Some(prev_pid) = CURRENT_PID {
        let prev_task = TASKS.get_mut(&prev_pid).unwrap();
        prev_task.save(stack_frame);
    }

    let next_task = TASKS.get(&next_pid).unwrap();
    CONTEXT_SWITCH_LOCK = false;

    CURRENT_PID.replace(next_pid);
    end_of_interrupt();
    next_task.restore();
}
