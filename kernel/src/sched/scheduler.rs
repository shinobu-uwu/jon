use alloc::vec::Vec;
use log::debug;

use super::task::Task;

pub struct Scheduler {
    tasks: Vec<Task>,
    current_task: Option<usize>,
    ticks: usize,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current_task: None,
            ticks: 0,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
        debug!("Added task to scheduler");
    }

    pub fn tick(&mut self) {
        self.ticks += 1;

        if self.ticks % 10 == 0 {
            self.switch_task();
            self.ticks = 0;
        }
    }

    fn switch_task(&mut self) {
        debug!("Switching tasks");

        if self.tasks.is_empty() {
            debug!("No tasks to switch");
            return;
        }

        if self.current_task.is_none() {
            self.current_task = Some(0);

            unsafe {
                self.tasks[0].restore();
            }
        }

        if self.tasks.len() < 2 {
            debug!("Not enough tasks to switch");
            return;
        }

        if let Some(current_task) = self.current_task {
            debug!("Switching from task {}", current_task);
            let next_task = (current_task + 1) % self.tasks.len();
            self.current_task = Some(next_task);

            unsafe {
                self.tasks[next_task].restore();
            }
        } else {
            debug!("Switching to first task");
            self.current_task = Some(0);
        }
    }
}
