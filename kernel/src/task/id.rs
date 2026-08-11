// XPARQ OS - Phase 6: Task Identifier
// Strong type for Task IDs to avoid confusing naked usizes

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TaskId(pub usize);

impl TaskId {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

// Special task ID for "None" in our intrusive linked lists (since we can't use Option<usize> in atomic arrays cleanly, though Option<TaskId> works.
// We will use Option<TaskId> for safety).
