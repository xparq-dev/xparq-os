// XPARQ OS - Phase 6: Task struct
// Defines a process/thread and its linked-list pointers

use crate::task::id::TaskId;
use crate::task::state::TaskState;

#[derive(Debug)]
pub struct Task {
    pub id: TaskId,
    pub state: TaskState,
    pub stack_ptr: u64, // kernel rsp
    pub kernel_stack_top: u64,
    pub pml4_addr: u64, // cr3
    pub sleep_until: u64, // kernel ticks
    pub fd_table: [crate::objects::Handle; 16], // Unified Handle Table
    
    // Per-task saved user context (used by sysretq after blocking)
    pub saved_user_rip: u64,
    pub saved_user_rsp: u64,
    pub saved_user_rflags: u64,
    
    pub ipc_mailbox: spin::Mutex<crate::ipc::IpcChannel>,

    // Intrusive queue pointers
    pub next: Option<TaskId>,
    pub prev: Option<TaskId>,
}

impl Task {
    pub const fn new() -> Self {
        Self {
            id: TaskId::new(0),
            state: TaskState::Unused,
            stack_ptr: 0,
            kernel_stack_top: 0,
            pml4_addr: 0,
            sleep_until: 0,
            fd_table: [crate::objects::Handle::INVALID; 16],
            saved_user_rip: 0,
            saved_user_rsp: 0,
            saved_user_rflags: 0,
            ipc_mailbox: spin::Mutex::new(crate::ipc::IpcChannel::new()),
            next: None,
            prev: None,
        }
    }
}
