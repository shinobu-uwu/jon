use bitmap_allocator::BitAlloc;

use super::{pid::Pid, PID_ALLOCATOR};

pub struct Task {
    pub pid: Pid,
    pub state: TaskState,
    pub parent: Option<Pid>,
}

pub enum TaskState {
    Running,
    Sleeping,
    Stopped,
    Zombie,
}

impl Task {
    pub fn new(parent: Option<Pid>) -> Self {
        let pid = Pid::new(PID_ALLOCATOR.lock().alloc().expect("Failed to alloc pid"));
        Self {
            pid,
            state: TaskState::Running,
            parent,
        }
    }
}
