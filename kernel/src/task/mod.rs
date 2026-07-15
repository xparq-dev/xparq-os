// XPARQ OS - Phase 6: Task Manager
// Manages process/thread lifecycle and scheduling

pub mod id;
pub mod state;
pub mod context;
pub mod task;
pub mod pool;
pub mod queue;
pub mod scheduler;
pub mod idle;
pub mod switch;
pub mod elf;
pub mod wait_queue;

use crate::sync::IrqSafeMutex;
pub use scheduler::TaskManager;

pub static TASK_MANAGER: IrqSafeMutex<TaskManager> = IrqSafeMutex::new(TaskManager::new());
