// XPARQ OS - Phase 12: CPU Subsystem
// Represents the root execution identity for each core.

use crate::task::id::TaskId;
use crate::task::scheduler::SchedulerContext;
use spin::Mutex;
use arrayvec::ArrayVec;

pub mod id;

pub const MAX_CPUS: usize = 8;

pub struct Cpu {
    pub id: usize,
    pub current_task: Option<TaskId>,
    pub scheduler: SchedulerContext,
    pub kernel_stack: *mut u8,
}

// We need to implement Send/Sync safely because we will store this in a static array.
unsafe impl Send for Cpu {}
unsafe impl Sync for Cpu {}

pub static CPUS: [Mutex<Cpu>; MAX_CPUS] = [const { Mutex::new(Cpu::new(0)) }; MAX_CPUS];

impl Cpu {
    pub const fn new(id: usize) -> Self {
        Self {
            id,
            current_task: None,
            scheduler: SchedulerContext::new(),
            kernel_stack: core::ptr::null_mut(),
        }
    }
}
