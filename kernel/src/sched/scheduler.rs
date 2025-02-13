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
        debug!("No current task");
        return;
    }

    let current_pid = CURRENT_PID;
    let next_pid = {
        let tasks = &mut TASKS;
        let current_pid = CURRENT_PID;
        let queues = &mut QUEUES;

        if let Some(pid) = current_pid {
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

                let next_pid = queues.get_next_task();
                next_pid
            } else {
                Some(pid)
            }
        } else {
            let next_pid = queues.get_next_task();
            next_pid
        }
    };

    match next_pid {
        Some(p) => unsafe {
            let prev_context = &mut TASKS.get_mut(&current_pid.unwrap()).unwrap().context;
            let next_context = &TASKS.get(&p).unwrap().context;
            debug!("Switching from task {} to task {}", current_pid.unwrap(), p);
            debug!("Queues len: {}", QUEUES.normal.len());
            arch::switch_to(prev_context, &next_context, stack_frame);
        },
        None => return,
    }
}

pub fn add_task(task: Task) {
    unsafe {
        let pid = task.pid;
        QUEUES.add_task(pid, task.priority);
        TASKS.insert(pid, task);

        if CURRENT_PID.is_none() {
            CURRENT_PID = Some(pid);
        }
    }
}

pub fn remove_current_task() {
    unsafe {
        if let Some(pid) = CURRENT_PID {
            TASKS.remove(&pid).unwrap();
        }

        CURRENT_PID = None;
    }
}
