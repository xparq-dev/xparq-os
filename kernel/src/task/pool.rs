// XPARQ OS - Phase 6: Task Pool
// Static array storage for all tasks

use crate::task::task::Task;
use crate::task::id::TaskId;
use crate::task::state::TaskState;

pub const MAX_TASKS: usize = 64;

pub struct TaskPool {
    tasks: [Task; MAX_TASKS],
}

impl TaskPool {
    pub const fn new() -> Self {
        Self {
            tasks: [const { Task::new() }; MAX_TASKS],
        }
    }

    pub fn get_task_mut(&mut self, id: TaskId) -> Option<&mut Task> {
        if id.as_usize() < MAX_TASKS {
            Some(&mut self.tasks[id.as_usize()])
        } else {
            None
        }
    }

    pub fn get_task(&self, id: TaskId) -> Option<&Task> {
        if id.as_usize() < MAX_TASKS {
            Some(&self.tasks[id.as_usize()])
        } else {
            None
        }
    }

    pub fn allocate_task(&mut self) -> Option<TaskId> {
        for i in 0..MAX_TASKS {
            if self.tasks[i].state == TaskState::Unused {
                self.tasks[i].id = TaskId::new(i);
                self.tasks[i].state = TaskState::Created;
                self.tasks[i].next = None;
                self.tasks[i].prev = None;
                return Some(TaskId::new(i));
            }
        }
        None
    }

    pub fn free_task(&mut self, id: TaskId) {
        if let Some(task) = self.get_task_mut(id) {
            task.state = TaskState::Unused;
            task.next = None;
            task.prev = None;
        }
    }
}
