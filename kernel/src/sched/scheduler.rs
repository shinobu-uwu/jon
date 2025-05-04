use alloc::{
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    vec::Vec,
};
use log::{debug, info};
use spinning_top::{RwSpinlock, Spinlock};

use crate::arch::x86::{cpu::current_pcr_mut, structures::Registers};

use super::{
    pid::Pid,
    task::{Priority, State, Task},
};

pub static TASKS: RwSpinlock<BTreeMap<Pid, Task>> = RwSpinlock::new(BTreeMap::new());
pub static READY_QUEUE: Spinlock<VecDeque<Pid>> = Spinlock::new(VecDeque::new());
pub static BLOCKED_QUEUE: Spinlock<VecDeque<Pid>> = Spinlock::new(VecDeque::new());
//
// Constants for scheduler tuning
const QUANTUM_BASE: u64 = 8;
const HIGH_PRIORITY_BONUS: u64 = 24;
const LOW_PRIORITY_PENALTY: u64 = 6;

pub unsafe fn schedule(stack_frame: &Registers) {}

fn balance_load() {
    todo!()
}

pub fn add_task(task: Task) {
    let pid = task.pid;
    debug!("Adding task {}", pid);
    READY_QUEUE.lock().push_back(pid);
    TASKS.write().insert(pid, task);
}

pub fn remove_current_task() {
    if let Some(pid) = *CURRENT_PID.lock() {
        remove_task(pid);
    }
}

pub fn remove_task(pid: Pid) {
    debug!("Removing task {}", pid);
    let mut tasks = TASKS.write();
    let mut ready_queue = READY_QUEUE.lock();
    let mut current_pid = CURRENT_PID.lock();

    match tasks.get_mut(&pid) {
        Some(t) => {
            info!("Found task {:#?}", t);
            t.state = State::Stopped;

            ready_queue.retain(|&p| p != pid);

            if *current_pid == Some(pid) {
                *current_pid = None;
            }
        }
        None => {
            debug!("Task {} not found", pid);
            return;
        }
    }

    // if tasks.remove(&pid).is_none() {
    //     debug!("Task {} not found", pid);
    //     return;
    // }
}

pub fn current_pid() -> Option<Pid> {
    *CURRENT_PID.lock()
}

pub fn get_tasks() -> Vec<&'static Task> {
    let tasks = TASKS.read();

    unsafe { tasks.values().map(|task| &*(task as *const Task)).collect() }
}

pub fn current_task() -> Option<&'static Task> {
    match *CURRENT_PID.lock() {
        Some(pid) => {
            let tasks = TASKS.read();
            tasks
                .get(&pid)
                .map(|task| unsafe { &*(task as *const Task) })
        }
        None => None,
    }
}

pub fn current_task_mut() -> Option<&'static mut Task> {
    match *CURRENT_PID.lock() {
        Some(pid) => {
            let mut tasks = TASKS.write();
            tasks
                .get_mut(&pid)
                .map(|task| unsafe { &mut *(task as *mut Task) })
        }
        None => None,
    }
}

pub fn get_task(pid: Pid) -> Option<&'static Task> {
    let tasks = TASKS.read();

    // this is safe because TASKS' inner is static
    unsafe { tasks.get(&pid).map(|task| &*(task as *const Task)) }
}

pub fn get_task_mut(pid: Pid) -> Option<&'static mut Task> {
    let mut tasks = TASKS.write();

    // this is safe because TASKS' inner is static
    unsafe { tasks.get_mut(&pid).map(|task| &mut *(task as *mut Task)) }
}

pub fn block_current_task() {
    let current_pid = CURRENT_PID.lock();

    if let Some(pid) = *current_pid {
        block_task(pid);
    }
}

pub fn block_task(pid: Pid) {
    info!("Blocking task {}", pid);
    let mut tasks = TASKS.write();
    let mut blocked_queue = BLOCKED_QUEUE.lock();
    let mut ready_queue = READY_QUEUE.lock();
    let mut current_pid = CURRENT_PID.lock();

    if let Some(task) = tasks.get_mut(&pid) {
        task.state = State::Blocked;
        ready_queue.retain(|&p| p != pid);
        blocked_queue.push_back(pid);

        if let Some(p) = *current_pid {
            if p == pid {
                *current_pid = None;
            }
        }

        unsafe {
            schedule(&Registers::default());
        }
    }
}

pub fn unblock_task(pid: Pid) {
    debug!("Unblocking task {}", pid);
    let mut tasks = TASKS.write();
    let mut blocked_queue = BLOCKED_QUEUE.lock();
    let mut ready_queue = READY_QUEUE.lock();
    let current_pid = *CURRENT_PID.lock();

    if let Some(task) = tasks.get_mut(&pid) {
        if current_pid == Some(pid) {
            task.state = State::Running;
        } else {
            task.state = State::Waiting;
        }

        blocked_queue.retain(|&p| p != pid);
        ready_queue.push_back(pid);
    }
}

pub fn unblock_current_task() {
    if let Some(pid) = *CURRENT_PID.lock() {
        unblock_task(pid);
    }
}
