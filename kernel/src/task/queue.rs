// XPARQ OS - Phase 6: Task Queue
// Intrusive array-backed linked list for task management

use crate::task::id::TaskId;
use crate::task::pool::TaskPool;

#[derive(Debug)]
pub struct TaskQueue {
    head: Option<TaskId>,
    tail: Option<TaskId>,
}

impl TaskQueue {
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub fn head(&self) -> Option<TaskId> {
        self.head
    }

    pub fn push_back(&mut self, pool: &mut TaskPool, id: TaskId) {
        if let Some(task) = pool.get_task_mut(id) {
            task.next = None;
            task.prev = self.tail;
        }

        if let Some(tail_id) = self.tail {
            if let Some(tail_task) = pool.get_task_mut(tail_id) {
                tail_task.next = Some(id);
            }
        } else {
            self.head = Some(id); // Queue was empty
        }
        
        self.tail = Some(id);
    }

    pub fn pop_front(&mut self, pool: &mut TaskPool) -> Option<TaskId> {
        let head_id = self.head?;
        
        let next_id = if let Some(head_task) = pool.get_task(head_id) {
            head_task.next
        } else {
            return None;
        };
        
        self.head = next_id;
        
        if let Some(next_id) = next_id {
            if let Some(next_task) = pool.get_task_mut(next_id) {
                next_task.prev = None;
            }
        } else {
            self.tail = None; // Queue is now empty
        }
        
        if let Some(head_task) = pool.get_task_mut(head_id) {
            head_task.next = None;
            head_task.prev = None;
        }
        
        Some(head_id)
    }

    pub fn remove(&mut self, pool: &mut TaskPool, id: TaskId) {
        let (prev_id, next_id) = if let Some(task) = pool.get_task(id) {
            (task.prev, task.next)
        } else {
            return;
        };

        if let Some(prev) = prev_id {
            if let Some(prev_task) = pool.get_task_mut(prev) {
                prev_task.next = next_id;
            }
        } else {
            self.head = next_id;
        }

        if let Some(next) = next_id {
            if let Some(next_task) = pool.get_task_mut(next) {
                next_task.prev = prev_id;
            }
        } else {
            self.tail = prev_id;
        }

        if let Some(task) = pool.get_task_mut(id) {
            task.next = None;
            task.prev = None;
        }
    }
}
