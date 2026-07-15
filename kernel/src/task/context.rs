// XPARQ OS - Phase 6: Task Context
// Architecture-specific state for a task (x86_64)

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TaskContext {
    pub rflags: u64,
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub rip: u64,
}

impl TaskContext {
    pub const fn new() -> Self {
        Self {
            rflags: 0x202,
            r15: 0, r14: 0, r13: 0, r12: 0, rbx: 0, rbp: 0,
            rip: 0,
        }
    }
}
