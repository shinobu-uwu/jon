use alloc::vec::Vec;
use log::debug;

use crate::sched::task::Task;

use super::pid::Pid;

pub struct Scheduler {
    current_index: usize,
    tasks: Vec<Task>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            current_index: 0,
            tasks: Vec::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn kill(&mut self, pid: Pid) {
        if let Some(index) = self.tasks.iter().position(|task| task.pid == pid) {
            let task = self.tasks.remove(index);
            debug!("Killed task with {}", task.pid);
        }

        debug!("Task with {} not found", pid);
    }

    pub fn tick(&mut self) {
        if self.tasks.is_empty() {
            debug!("No tasks to schedule");
            return;
        }

        let task = &self.tasks[self.current_index];
        debug!("Processing task with {}", task.pid);
        self.current_index = (self.current_index + 1) % self.tasks.len();
    }
}
