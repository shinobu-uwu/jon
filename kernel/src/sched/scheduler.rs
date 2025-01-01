use alloc::vec::Vec;

use crate::sched::task::Task;

#[derive(Debug)]
pub struct Scheduler {
    tasks: Vec<Task>,
    current: Option<usize>,
    ticks: u64,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current: None,
            ticks: 0,
        }
    }

    pub fn tick(&mut self) {
        self.ticks += 1;

        // Switch tasks every N ticks (for example, every 10 ticks)
        if self.ticks % 10 == 0 {
            self.switch_next_task();
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn switch_next_task(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        if let Some(current) = self.current {
            unsafe {
                self.tasks[current].context.save();
            }
        }

        let next = match self.current {
            Some(current) => (current + 1) % self.tasks.len(),
            None => 0,
        };

        unsafe {
            self.tasks[next].context.restore();
        }

        self.current = Some(next);
    }
}
