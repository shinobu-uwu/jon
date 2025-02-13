use alloc::collections::{btree_map::BTreeMap, vec_deque::VecDeque};
use log::debug;

use crate::arch::{self, x86::structures::Registers};

use super::{
    pid::Pid,
    task::{Priority, State, Task},
};

static mut TASKS: BTreeMap<Pid, Task> = BTreeMap::new();
static mut QUEUES: PriorityQueues = PriorityQueues::new();
static mut CURRENT_PID: Option<Pid> = None;

const QUANTUM_BASE: u64 = 10;
const HIGH_PRIORITY_BONUS: u64 = 15;
const LOW_PRIORITY_PENALTY: u64 = 5;

#[derive(Debug)]
pub struct PriorityQueues {
    high: VecDeque<Pid>,
    normal: VecDeque<Pid>,
    low: VecDeque<Pid>,
}

impl PriorityQueues {
    pub const fn new() -> Self {
        Self {
            high: VecDeque::new(),
            normal: VecDeque::new(),
            low: VecDeque::new(),
        }
    }

    fn get_next_task(&mut self) -> Option<Pid> {
        self.high
            .pop_front()
            .or_else(|| self.normal.pop_front())
            .or_else(|| self.low.pop_front())
    }

    pub fn add_task(&mut self, pid: Pid, priority: Priority) {
        match priority {
            Priority::High => self.high.push_back(pid),
            Priority::Normal => self.normal.push_back(pid),
            Priority::Low => self.low.push_back(pid),
        }
    }
}

pub unsafe fn tick(stack_frame: &Registers) {
    if CURRENT_PID.is_none() && TASKS.is_empty() {
        debug!("No tasks to run, halting");
        return;
    }

    let current_pid = CURRENT_PID;
    let next_pid = {
        let tasks = &mut TASKS;
        let queues = &mut QUEUES;

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
                        queues.add_task(pid, current_task.priority);
                    }
                    queues.get_next_task()
                } else {
                    None
                }
            }
            None => queues.get_next_task(),
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
            arch::switch_to(&mut prev_task.context, &next_task.context, stack_frame);
        }
        (Some(next), None) => {
            let next_task = TASKS.get_mut(&next).unwrap();
            debug!("Switching to task {}", next);

            next_task.state = State::Running;

            CURRENT_PID = Some(next);
            arch::switch_to(&mut Registers::default(), &next_task.context, stack_frame);
        }
        _ => {}
    }
}

pub fn add_task(task: Task) {
    unsafe {
        let pid = task.pid;
        debug!("Adding task {}", pid);
        QUEUES.add_task(pid, task.priority);
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
        let task = match (TASKS.remove(&pid)) {
            Some(task) => task,
            None => {
                debug!("Task {} not found", pid);
                return;
            }
        };

        match task.priority {
            Priority::High => QUEUES.high.retain(|&p| p != pid),
            Priority::Normal => QUEUES.normal.retain(|&p| p != pid),
            Priority::Low => QUEUES.low.retain(|&p| p != pid),
        }

        if CURRENT_PID == Some(pid) {
            CURRENT_PID = None;
        }
    }
}
