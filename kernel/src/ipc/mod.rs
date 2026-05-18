// XPARQ OS - Phase 01: OS & Kernel Foundations
// Inter-Process Communication (IPC) module
// Implements FIDL channel primitives for message passing

#![no_std]

pub mod channel;

// Re-export main types
pub use channel::{Channel, ChannelHandle, ChannelMessage, ChannelError};

/// Initialize the IPC system
pub fn init() {
    println!("Initializing IPC system...");
    
    // Initialize channel system
    channel::init();
    
    println!("IPC system initialized");
}

/// IPC errors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IpcError {
    /// Invalid channel handle
    InvalidChannel,
    /// Channel closed
    ChannelClosed,
    /// Buffer too small
    BufferTooSmall,
    /// No messages available
    NoMessages,
    /// Permission denied
    PermissionDenied,
    /// Resource exhausted
    ResourceExhausted,
}
