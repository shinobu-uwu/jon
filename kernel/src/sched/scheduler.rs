use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use spinning_top::RwSpinlock;

use crate::arch::{
    switch_to,
    x86::{
        cpu::{current_pcr, current_pcr_mut, get_pcr_mut, PCRS},
        structures::Registers,
    },
};

use super::{
    pid::Pid,
    task::{Priority, State, Task},
};

pub static TASKS: RwSpinlock<BTreeMap<Pid, Task>> = RwSpinlock::new(BTreeMap::new());

const QUANTUM_BASE: u64 = 8;
const HIGH_PRIORITY_BONUS: u64 = 24;
const LOW_PRIORITY_PENALTY: u64 = 6;

pub unsafe fn schedule(stack_frame: &Registers) {
    let pcr = current_pcr_mut();

    if pcr.sched.current_pid.is_none() && pcr.sched.run_queue.is_empty() {
        let idle_pid = pcr.idle_task();
        pcr.sched.current_pid = Some(idle_pid);
        let task = get_task_mut(idle_pid).unwrap();
        task.state = State::Running;

        return switch_to(None, task, stack_frame);
    }

    let next_pid = match pcr.sched.current_pid {
        Some(pid) => {
            let current_task = get_task_mut(pid).unwrap();
            current_task.quantum += 1;

            let quantum_limit = match current_task.priority {
                Priority::High => QUANTUM_BASE + HIGH_PRIORITY_BONUS,
                Priority::Normal => QUANTUM_BASE,
                Priority::Low => QUANTUM_BASE - LOW_PRIORITY_PENALTY,
            };

            if current_task.quantum >= quantum_limit {
                current_task.quantum = 0;
                if matches!(current_task.state, State::Running) {
                    pcr.sched.run_queue.push_back(pid);
                }
                pcr.sched.run_queue.pop_front()
            } else {
                None
            }
        }
        None => pcr.sched.run_queue.pop_front(),
    };

    match (next_pid, pcr.sched.current_pid) {
        (Some(next), Some(current)) => {
            {
                let prev_task = get_task_mut(current).unwrap();
                prev_task.state = State::Waiting;
                prev_task.quantum = 0; // Reset quantum when switching away

                let next_task = get_task_mut(next).unwrap();
                next_task.state = State::Running;
            }

            pcr.sched.current_pid = Some(next);

            let prev_task_ptr = get_task_mut(current).unwrap() as *mut Task;
            let next_task_ptr = get_task_mut(next).unwrap() as *mut Task;

            let prev_task_ref = &mut *prev_task_ptr;
            let next_task_ref = &*next_task_ptr;

            switch_to(Some(prev_task_ref), next_task_ref, stack_frame);
        }
        (Some(next), None) => {
            {
                let mut tasks_guard = TASKS.write();
                let next_task = tasks_guard.get_mut(&next).unwrap();
                next_task.state = State::Running;
            }

            pcr.sched.current_pid = Some(next);
            let next_task_ptr = get_task_mut(next).unwrap() as *mut Task;
            let next_task_ref = &*next_task_ptr;
            switch_to(None, next_task_ref, stack_frame);
        }
        _ => {}
    }
}

pub fn current_pid() -> Option<Pid> {
    current_pcr().sched.current_pid
}

pub fn get_tasks() -> Vec<&'static Task> {
    let tasks = TASKS.read();

    unsafe { tasks.values().map(|task| &*(task as *const Task)).collect() }
}

pub fn current_task() -> Option<&'static Task> {
    match current_pcr().sched.current_pid {
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
    match current_pcr().sched.current_pid {
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

pub fn remove_current_task() -> Option<Pid> {
    let pcr = current_pcr_mut();

    if let Some(pid) = pcr.sched.current_pid {
        remove_task(pid);

        return pcr.sched.current_pid;
    }

    None
}

pub fn remove_task(pid: Pid) -> bool {
    let pcrs = unsafe { &mut PCRS };

    for pcr in pcrs {
        pcr.sched.run_queue.retain(|&p| p != pid);

        if pcr.sched.current_pid == Some(pid) {
            pcr.sched.current_pid = None;
        }
    }

    let mut tasks = TASKS.write();
    if let Some(task) = tasks.get_mut(&pid) {
        task.state = State::Stopped;
        return true;
    }

    false
}

pub fn add_task(task: Task, cpu_affinity: Option<u64>) {
    let pcr = match cpu_affinity {
        Some(cpu_id) => get_pcr_mut(cpu_id),
        None => current_pcr_mut(),
    };
    let pid = task.pid;
    TASKS.write().insert(pid, task);
    pcr.sched.run_queue.push_back(pid);
}
