use alloc::collections::vec_deque::VecDeque;

use crate::sched::task::Task;

pub struct Scheduler {
    tasks: VecDeque<Task>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.tasks.push_back(task);
    }
}
