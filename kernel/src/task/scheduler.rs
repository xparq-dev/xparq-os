// XPARQ OS - Phase 12: Scheduler & Task Manager
// SMP-aware task management

use crate::task::id::TaskId;
use crate::task::pool::TaskPool;
use crate::task::queue::TaskQueue;
use crate::task::state::TaskState;
use crate::task::context::TaskContext;
use crate::cpu::{CPUS, MAX_CPUS};

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

pub struct TaskManager {
    pub pool: TaskPool,
    pub next_cpu_rr: usize,
}

impl TaskManager {
    pub const fn new() -> Self {
        Self {
            pool: TaskPool::new(),
            next_cpu_rr: 0,
        }
    }

    pub fn spawn_task(&mut self, entry: fn(), stack_base: u64, stack_size: u64) -> Result<TaskId, ()> {
        let id = self.pool.allocate_task().ok_or(())?;
        
        if let Some(task) = self.pool.get_task_mut(id) {
            let mut ctx = TaskContext::new();
            ctx.rip = entry as u64;
            
            let stack_ctx_ptr = (stack_base + stack_size - core::mem::size_of::<TaskContext>() as u64) as *mut TaskContext;
            
            // Stack Canary for overflow detection
            let canary_ptr = stack_base as *mut u64;
            unsafe {
                core::ptr::write(canary_ptr, 0xDEADC0DEDEADC0DE);
                core::ptr::write(stack_ctx_ptr, ctx);
            }
            
            task.stack_ptr = stack_ctx_ptr as u64;
            task.kernel_stack_top = stack_base + stack_size;
            task.pml4_addr = xparq_hal::x86_64::paging::get_cr3(); // Inherit current CR3
            task.state = TaskState::Ready;

            // Debug: verify entry and written RIP
            unsafe {
                crate::uart_puts(b"[spawn_task] entry=");
                crate::hal::x86_64::idt::df_print_hex(entry as u64);
                crate::uart_puts(b"[spawn_task] stack_ptr=");
                crate::hal::x86_64::idt::df_print_hex(task.stack_ptr);
                let written_rip = *((task.stack_ptr + 56) as *const u64);
                crate::uart_puts(b"[spawn_task] written_rip=");
                crate::hal::x86_64::idt::df_print_hex(written_rip);
            }

            // Assign to CPU using Round Robin
            let target_cpu = 0;
            self.next_cpu_rr = (self.next_cpu_rr + 1) % MAX_CPUS;
            
            let mut cpu = CPUS[target_cpu].lock();
            cpu.scheduler.ready_queue.push_back(&mut self.pool, id);
            
            Ok(id)
        } else {
            Err(())
        }
    }

    pub fn wake(&mut self, id: TaskId) {
        if let Some(task) = self.pool.get_task_mut(id) {
            if task.state.can_transition_to(TaskState::Ready) {
                task.state = TaskState::Ready;
                // For simplicity in Phase 12, wake to CPU 0, or derive from Task ID.
                // In a full implementation, the task should remember its affinity.
                let target_cpu = 0;
                let mut cpu = CPUS[target_cpu].lock();
                cpu.scheduler.ready_queue.push_back(&mut self.pool, id);
            }
        }
    }

    pub fn sleep_current_task(&mut self, ms: u64) {
        let cpu_id = crate::cpu::id::current_cpu_id();
        let mut cpu = CPUS[cpu_id].lock();
        
        if let Some(current_id) = cpu.current_task {
            if let Some(task) = self.pool.get_task_mut(current_id) {
                if task.state.can_transition_to(TaskState::Sleeping) {
                    task.state = TaskState::Sleeping;
                    task.sleep_until = crate::time::KERNEL_CLOCK.lock().ticks + ms; // Assuming KERNEL_CLOCK has ticks
                    cpu.scheduler.sleep_queue.push_back(&mut self.pool, current_id);
                }
            }
        }
    }

    pub fn tick(&mut self) {
        let current_ticks = crate::time::KERNEL_CLOCK.lock().ticks;
        // Check all sleep queues across CPUs and wake tasks
        for cpu_mutex in CPUS.iter() {
            let mut cpu = cpu_mutex.lock();
            let mut wake_list = arrayvec::ArrayVec::<TaskId, 32>::new();
            
            // Collect tasks that should wake
            let mut curr = cpu.scheduler.sleep_queue.head();
            while let Some(task_id) = curr {
                if let Some(task) = self.pool.get_task(task_id) {
                    if task.sleep_until <= current_ticks {
                        let _ = wake_list.try_push(task_id);
                    }
                    curr = task.next;
                } else {
                    break;
                }
            }
            
            // Remove from sleep queue and push to ready queue
            for task_id in wake_list {
                cpu.scheduler.sleep_queue.remove(&mut self.pool, task_id);
                if let Some(task) = self.pool.get_task_mut(task_id) {
                    task.state = TaskState::Ready;
                }
                cpu.scheduler.ready_queue.push_back(&mut self.pool, task_id);
            }
        }
    }

    // Helper method to safely schedule the next task for a specific CPU.
    // This expects the TASK_MANAGER lock to be held already.
    pub fn schedule_next_for_cpu(&mut self, cpu_id: usize) -> u64 {
        let mut cpu = CPUS[cpu_id].lock();
        
        if let Some(current_id) = cpu.current_task {
            if let Some(task) = self.pool.get_task_mut(current_id) {
                // If it was running and hasn't been put to sleep/blocked, preempt it
                if task.state == TaskState::Running {
                    if task.state.can_transition_to(TaskState::Ready) {
                        task.state = TaskState::Ready;
                        cpu.scheduler.ready_queue.push_back(&mut self.pool, current_id);
                    }
                }
            }
        }

        if let Some(next_id) = cpu.scheduler.ready_queue.pop_front(&mut self.pool) {
            if let Some(task) = self.pool.get_task_mut(next_id) {
                if task.state.can_transition_to(TaskState::Running) {
                    task.state = TaskState::Running;
                    cpu.current_task = Some(next_id);
                    if task.pml4_addr != 0 {
                        // Switch Page Tables for Memory Isolation
                        xparq_hal::x86_64::paging::set_cr3(task.pml4_addr);
                    }
                    unsafe {
                        crate::syscall::dispatcher::CPU_LOCAL.kernel_rsp = task.kernel_stack_top;
                        crate::hal::x86_64::gdt::set_kernel_stack(task.kernel_stack_top);
                    }
                    return task.stack_ptr;
                }
            }
        }

        // Switch to Idle Task if none are ready
        if let Some(idle_id) = cpu.scheduler.idle_task {
            if let Some(idle_task) = self.pool.get_task_mut(idle_id) {
                idle_task.state = TaskState::Running;
                cpu.current_task = Some(idle_id);
                if idle_task.pml4_addr != 0 {
                    xparq_hal::x86_64::paging::set_cr3(idle_task.pml4_addr);
                }
                unsafe {
                    crate::syscall::dispatcher::CPU_LOCAL.kernel_rsp = idle_task.kernel_stack_top;
                    crate::hal::x86_64::gdt::set_kernel_stack(idle_task.kernel_stack_top);
                }
                return idle_task.stack_ptr;
            }
        }

        panic!("CPU {} PANIC: No tasks ready and Idle task is missing!", cpu_id);
    }
}
