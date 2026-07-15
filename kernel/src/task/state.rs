// XPARQ OS - Phase 6: Task State Machine
// Defines the valid states of a process/thread

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockReason {
    Input,
    Ipc,
    Event,
    Network,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Slot is available for allocation
    Unused,
    /// Task has been allocated but not yet scheduled
    Created,
    /// Task is in the Ready Queue, waiting for CPU time
    Ready,
    /// Task is currently executing on a CPU
    Running,
    /// Task is voluntarily sleeping for a duration
    Sleeping,
    /// Task is blocked waiting for an event/resource (e.g., IO, Mutex)
    Blocked(BlockReason),
    /// Task is waiting for a child process to exit (waitpid)
    Waiting,
    /// Task has exited but parent hasn't collected its status
    Zombie,
    /// Task has been fully cleaned up and is returning to Unused
    Dead,
}

impl TaskState {
    /// Checks if transitioning from `self` to `new_state` is valid
    pub fn can_transition_to(&self, new_state: TaskState) -> bool {
        match (self, new_state) {
            (TaskState::Unused, TaskState::Created) => true,
            (TaskState::Created, TaskState::Ready) => true,
            
            (TaskState::Ready, TaskState::Running) => true,
            (TaskState::Running, TaskState::Ready) => true, // Preempted
            (TaskState::Running, TaskState::Sleeping) => true, // Syscall sleep
            (TaskState::Running, TaskState::Blocked(_)) => true, // Syscall block
            (TaskState::Running, TaskState::Waiting) => true, // Syscall waitpid
            (TaskState::Running, TaskState::Zombie) => true, // Exited
            
            // Wakeup Paths
            (TaskState::Sleeping, TaskState::Ready) => true,
            (TaskState::Blocked(_), TaskState::Ready) => true,
            (TaskState::Waiting, TaskState::Ready) => true,
            
            (TaskState::Zombie, TaskState::Dead) => true,
            (TaskState::Dead, TaskState::Unused) => true,
            
            _ => false, // Invalid transition
        }
    }
}
