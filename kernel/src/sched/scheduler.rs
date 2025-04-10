use alloc::collections::{btree_map::BTreeMap, vec_deque::VecDeque};
use log::{debug, info};

use crate::arch::{self, x86::structures::Registers};

use super::{
    pid::Pid,
    task::{Priority, State, Task},
};

static mut TASKS: BTreeMap<Pid, Task> = BTreeMap::new();
static mut READY_QUEUE: VecDeque<Pid> = VecDeque::new();
static mut BLOCKED_QUEUE: VecDeque<Pid> = VecDeque::new();
static mut CURRENT_PID: Option<Pid> = None;
pub static mut IDLE_PID: Option<Pid> = None;

const QUANTUM_BASE: u64 = 10;
const HIGH_PRIORITY_BONUS: u64 = 15;
const LOW_PRIORITY_PENALTY: u64 = 5;

pub unsafe fn init() {
    debug!("Initializing scheduler");
    if IDLE_PID.is_none() {
        debug!("Creating idle task");
        let task = Task::idle();
        let pid = task.pid;
        TASKS.insert(pid, task);
        IDLE_PID = Some(pid);
    }
    debug!("Scheduler initialized");
}

pub unsafe fn schedule(stack_frame: &Registers) {
    debug!("Scheduling");

    if CURRENT_PID.is_none() && READY_QUEUE.is_empty() {
        debug!("No regular tasks to run, checking for idle task");

        if let Some(idle_pid) = IDLE_PID {
            if !READY_QUEUE.contains(&idle_pid) {
                debug!("Adding idle task to ready queue");
                READY_QUEUE.push_back(idle_pid);
            }
        } else {
            debug!("Creating new idle task");
            let idle_task = Task::idle();
            let pid = idle_task.pid;
            add_task(idle_task);
            IDLE_PID = Some(pid);
        }
    }

    let current_pid = CURRENT_PID;
    let next_pid = {
        let tasks = &mut TASKS;
        let queue = &mut READY_QUEUE;

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
        READY_QUEUE.push_back(pid);
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

        READY_QUEUE.retain(|&p| p != pid);

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

pub fn block_current_task() {
    unsafe {
        if let Some(pid) = CURRENT_PID {
            block_task(pid);
        }
    }
}

pub fn block_task(pid: Pid) {
    unsafe {
        info!("Blocking task {}", pid);

        if let Some(task) = TASKS.get_mut(&pid) {
            task.state = State::Blocked;
            READY_QUEUE.retain(|&p| p != pid);
            BLOCKED_QUEUE.push_back(pid);

            if let Some(current_pid) = CURRENT_PID {
                if current_pid == pid {
                    CURRENT_PID = None;
                }
            }

            schedule(&Registers::default());
        }
    }
}

pub fn unblock_task(pid: Pid) {
    unsafe {
        info!("Unblocking task {}", pid);

        if let Some(task) = TASKS.get_mut(&pid) {
            task.state = State::Running;
            BLOCKED_QUEUE.retain(|&p| p != pid);
            READY_QUEUE.push_back(pid);
        }
    }
}

pub fn unblock_current_task() {
    unsafe {
        if let Some(pid) = CURRENT_PID {
            unblock_task(pid);
        }
    }
}
