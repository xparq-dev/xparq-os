// XPARQ OS - Phase 11: IPC
// Inter-Process Communication

use crate::task::id::TaskId;
use crate::task::wait_queue::WaitQueue;
use arrayvec::ArrayVec;
use crate::input::CircularBuffer;
use spin::Mutex;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Message {
    pub sender: TaskId,
    pub type_: u32,
    pub data: [u8; 32],
}

impl Default for Message {
    fn default() -> Self {
        Self {
            sender: TaskId(0), // Valid task id normally starts at 1, but we use Default only for buffer initialization
            type_: 0,
            data: [0; 32],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcMode {
    Blocking,
    NonBlocking,
}

#[derive(Debug)]
pub struct IpcChannel {
    pub messages: CircularBuffer<Message, 32>,
    pub wait_queue: WaitQueue<16>,
    pub mode: IpcMode,
}

impl IpcChannel {
    pub const fn new() -> Self {
        Self {
            messages: CircularBuffer {
                data: [Message { sender: TaskId(0), type_: 0, data: [0; 32] }; 32],
                head: 0,
                tail: 0,
                count: 0,
            },
            wait_queue: WaitQueue::new(),
            mode: IpcMode::Blocking,
        }
    }
}
