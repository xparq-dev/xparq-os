// XPARQ OS - Phase 11: Wait Queues
// Allows tasks to block securely until an event occurs

use crate::task::{id::TaskId, state::{TaskState, BlockReason}, TASK_MANAGER};
use arrayvec::ArrayVec;

#[derive(Debug)]
pub struct WaitQueue<const N: usize> {
    pub tasks: ArrayVec<TaskId, N>,
}

impl<const N: usize> WaitQueue<N> {
    pub const fn new() -> Self {
        Self { tasks: ArrayVec::new_const() }
    }

    pub fn block_current(&mut self, reason: BlockReason) {
        unsafe { crate::uart_puts(b"    -> [block_current] getting current cpu task\n"); }
        let cpu_id = crate::cpu::id::current_cpu_id();
        let cpu = crate::cpu::CPUS[cpu_id].lock();
        let current_task_opt = cpu.current_task;
        drop(cpu);

        unsafe { crate::uart_puts(b"    -> [block_current] locking TASK_MANAGER\n"); }
        if let Some(current_id) = current_task_opt {
            let mut manager = TASK_MANAGER.lock();
            unsafe { crate::uart_puts(b"    -> [block_current] getting task_mut\n"); }
            if let Some(task) = manager.pool.get_task_mut(current_id) {
                if task.state.can_transition_to(TaskState::Blocked(reason)) {
                    task.state = TaskState::Blocked(reason);
                    let _ = self.tasks.try_push(current_id);
                }
            }
            // CRITICAL: drop the lock BEFORE firing int 32.
            // The timer_interrupt_handler also needs TASK_MANAGER.
            // Holding it across the interrupt causes a deadlock on a single-CPU spinlock system.
            drop(manager);
        }
        
        unsafe { crate::uart_puts(b"    -> [block_current] executing int 32\n"); }
        // Now trigger reschedule — lock is already released.
        unsafe { core::arch::asm!("int 32"); }
        unsafe { crate::uart_puts(b"    -> [block_current] returned from int 32\n"); }
    }

    /// Wakes the first task in the wait queue, changing its state to Ready.
    pub fn wake_one(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        let task_id = self.tasks.remove(0); // Pop front
        let mut manager = TASK_MANAGER.lock();
        manager.wake(task_id);
    }

    /// Wakes all tasks in the wait queue, changing their states to Ready.
    pub fn wake_all(&mut self) {
        let mut manager = TASK_MANAGER.lock();
        for task_id in self.tasks.drain(..) {
            manager.wake(task_id);
        }
    }
}
