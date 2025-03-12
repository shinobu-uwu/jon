use alloc::collections::{btree_map::BTreeMap, vec_deque::VecDeque};
use log::debug;

use crate::arch::{self, x86::structures::Registers};

use super::{
    pid::Pid,
    task::{Priority, State, Task},
};

static mut TASKS: BTreeMap<Pid, Task> = BTreeMap::new();
static mut QUEUE: VecDeque<Pid> = VecDeque::new();
static mut CURRENT_PID: Option<Pid> = None;

const QUANTUM_BASE: u64 = 10;
const HIGH_PRIORITY_BONUS: u64 = 15;
const LOW_PRIORITY_PENALTY: u64 = 5;

pub unsafe fn tick(stack_frame: &Registers) {
    if CURRENT_PID.is_none() && TASKS.is_empty() {
        debug!("No tasks to run, halting");
        return;
    }

    let current_pid = CURRENT_PID;
    let next_pid = {
        let tasks = &mut TASKS;
        let queue = &mut QUEUE;

        match current_pid {
            Some(pid) => {
                let current_task = tasks.get_mut(&pid).unwrap();
                current_task.quantum += 1;

                let quantum_limit = match current_task.priority {
                    Priority::High => QUANTUM_BASE + HIGH_PRIORITY_BONUS,
                    Priority::Normal => QUANTUM_BASE,
                    Priority::Low => QUANTUM_BASE - LOW_PRIORITY_PENALTY,
                };

                if current_task.quantum >= quantum_limit {
                    current_task.quantum = 0;
                    if let State::Running = current_task.state {
                        queue.push_back(pid);
                    }
                    queue.pop_front()
                } else {
                    None
                }
            }
            None => queue.pop_front(),
        }
    };

    match (next_pid, current_pid) {
        (Some(next), Some(current)) => {
            let prev_task = TASKS.get_mut(&current).unwrap();
            let next_task = TASKS.get_mut(&next).unwrap();
            debug!("Switching from task {} to task {}", current, next);

            prev_task.state = State::Waiting;
            next_task.state = State::Running;

            CURRENT_PID = Some(next);
            arch::switch_to(Some(prev_task), &next_task, stack_frame);
        }
        (Some(next), None) => {
            let next_task = TASKS.get_mut(&next).unwrap();
            debug!("Switching to task {}", next);

            next_task.state = State::Running;

            CURRENT_PID = Some(next);
            arch::switch_to(None, &next_task, stack_frame);
        }
        _ => {}
    }
}

pub fn add_task(task: Task) {
    unsafe {
        let pid = task.pid;
        debug!("Adding task {}", pid);
        QUEUE.push_back(pid);
        TASKS.insert(pid, task);
    }
}

pub fn remove_current_task() {
    unsafe {
        if let Some(pid) = CURRENT_PID {
            remove_task(pid);
        }
    }
}

pub fn remove_task(pid: Pid) {
    unsafe {
        debug!("Removing task {}", pid);

        if TASKS.remove(&pid).is_none() {
            debug!("Task {} not found", pid);
            return;
        }

        QUEUE.retain(|&p| p != pid);

        if CURRENT_PID == Some(pid) {
            CURRENT_PID = None;
        }
    }
}

pub fn current_pid() -> Option<Pid> {
    unsafe { CURRENT_PID }
}

pub fn current_task() -> Option<&'static Task> {
    unsafe { TASKS.get(&CURRENT_PID?) }
}

pub fn get_task(pid: Pid) -> Option<&'static Task> {
    unsafe { TASKS.get(&pid) }
}

pub fn get_task_mut(pid: Pid) -> Option<&'static mut Task> {
    unsafe { TASKS.get_mut(&pid) }
}
