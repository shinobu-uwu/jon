use super::pid::Pid;

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
