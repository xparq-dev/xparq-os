// XPARQ OS - Phase 12: Scheduler Context
// Per-CPU scheduler context

use crate::task::id::TaskId;
use crate::task::queue::TaskQueue;

#[derive(Debug)]
pub struct SchedulerContext {
    pub ready_queue: TaskQueue,
    pub sleep_queue: TaskQueue,
    pub idle_task: Option<TaskId>,
}

impl SchedulerContext {
    pub const fn new() -> Self {
        Self {
            ready_queue: TaskQueue::new(),
            sleep_queue: TaskQueue::new(),
            idle_task: None,
        }
    }
}
