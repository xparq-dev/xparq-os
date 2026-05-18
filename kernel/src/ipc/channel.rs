// XPARQ OS - Phase 01: OS & Kernel Foundations
// FIDL channel implementation
// Provides bidirectional communication channels for IPC

#![no_std]

use super::IpcError;
use bitflags::bitflags;
use core::sync::atomic::{AtomicU64, Ordering};

/// Global channel counter for generating unique IDs
static NEXT_CHANNEL_ID: AtomicU64 = AtomicU64::new(1);

/// Channel handle - represents one endpoint of a channel
#[derive(Debug, Clone, Copy)]
pub struct ChannelHandle {
    /// Unique channel identifier
    pub id: u64,
    /// Channel rights
    pub rights: ChannelRights,
    /// Whether this is the read or write endpoint
    pub endpoint: ChannelEndpoint,
}

/// Channel rights
#[derive(Debug, Clone, Copy)]
pub struct ChannelRights {
    /// Right to read from the channel
    pub read: bool,
    /// Right to write to the channel
    pub write: bool,
    /// Right to duplicate the channel handle
    pub duplicate: bool,
    /// Right to transfer the channel handle
    pub transfer: bool,
}

/// Channel endpoint
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelEndpoint {
    Read,
    Write,
}

/// Channel message
#[derive(Debug, Clone)]
pub struct ChannelMessage {
    /// Message data
    pub data: arrayvec::ArrayVec<u8, 4096>,
    /// Message handles (for handle passing)
    pub handles: arrayvec::ArrayVec<ChannelHandle, 16>,
    /// Message flags
    pub flags: MessageFlags,
}

/// Message flags
#[derive(Debug, Clone, Copy)]
pub struct MessageFlags {
    /// Whether this message contains handles
    pub has_handles: bool,
    /// Whether this message is urgent
    pub urgent: bool,
    /// Whether this message requires acknowledgment
    pub require_ack: bool,
}

/// Channel - represents a bidirectional communication channel
#[derive(Debug)]
pub struct Channel {
    /// Unique channel identifier
    pub id: u64,
    /// Read endpoint
    pub read_endpoint: ChannelEndpointData,
    /// Write endpoint
    pub write_endpoint: ChannelEndpointData,
    /// Channel capacity
    pub capacity: usize,
    /// Channel flags
    pub flags: ChannelFlags,
}

/// Channel endpoint data
#[derive(Debug)]
pub struct ChannelEndpointData {
    /// Message queue
    pub message_queue: spin::Mutex<arrayvec::ArrayVec<ChannelMessage, 64>>,
    /// Whether this endpoint is closed
    pub closed: core::sync::atomic::AtomicBool,
    /// Number of waiting readers/writers
    pub waiters: core::sync::atomic::AtomicU32,
}

/// Channel flags
#[derive(Debug, Clone, Copy)]
pub struct ChannelFlags {
    /// Whether this channel is synchronous (blocking)
    pub synchronous: bool,
    /// Whether this channel allows handle passing
    pub allow_handle_passing: bool,
    /// Whether this channel is peer-to-peer
    pub peer_to_peer: bool,
}

/// Channel errors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelError {
    /// Invalid channel handle
    InvalidHandle,
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
    /// Invalid message format
    InvalidMessage,
}

/// Channel manager - manages all channels
pub struct ChannelManager {
    /// Global channel registry
    channels: spin::Mutex<ChannelRegistry>,
}

/// Channel registry - stores all channels
#[derive(Debug)]
struct ChannelRegistry {
    /// Map from channel ID to channel
    channels: arrayvec::ArrayVec<(u64, Channel), 1024>,
}

impl ChannelManager {
    /// Create a new channel manager
    pub fn new() -> Self {
        Self {
            channels: spin::Mutex::new(ChannelRegistry {
                channels: arrayvec::ArrayVec::new(),
            }),
        }
    }
    
    /// Create a new channel
    pub fn create_channel(&self, capacity: usize, flags: ChannelFlags) -> Result<(ChannelHandle, ChannelHandle), ChannelError> {
        let mut registry = self.channels.lock();
        
        if registry.channels.is_full() {
            return Err(ChannelError::ResourceExhausted);
        }
        
        // Create channel
        let channel_id = NEXT_CHANNEL_ID.fetch_add(1, Ordering::SeqCst);
        
        let channel = Channel {
            id: channel_id,
            read_endpoint: ChannelEndpointData {
                message_queue: spin::Mutex::new(arrayvec::ArrayVec::new()),
                closed: core::sync::atomic::AtomicBool::new(false),
                waiters: core::sync::atomic::AtomicU32::new(0),
            },
            write_endpoint: ChannelEndpointData {
                message_queue: spin::Mutex::new(arrayvec::ArrayVec::new()),
                closed: core::sync::atomic::AtomicBool::new(false),
                waiters: core::sync::atomic::AtomicU32::new(0),
            },
            capacity,
            flags,
        };
        
        // Create handles
        let read_handle = ChannelHandle {
            id: channel_id,
            rights: ChannelRights {
                read: true,
                write: false,
                duplicate: true,
                transfer: true,
            },
            endpoint: ChannelEndpoint::Read,
        };
        
        let write_handle = ChannelHandle {
            id: channel_id,
            rights: ChannelRights {
                read: false,
                write: true,
                duplicate: true,
                transfer: true,
            },
            endpoint: ChannelEndpoint::Write,
        };
        
        // Register channel
        registry.channels.push((channel_id, channel));
        
        println!("Created channel {} with capacity {} bytes", channel_id, capacity);
        
        Ok((read_handle, write_handle))
    }
    
    /// Write a message to a channel
    pub fn write_message(&self, handle: ChannelHandle, message: ChannelMessage) -> Result<(), ChannelError> {
        let mut registry = self.channels.lock();
        
        // Find channel
        let channel = registry.channels.iter_mut()
            .find(|(id, _)| *id == handle.id)
            .map(|(_, channel)| channel)
            .ok_or(ChannelError::InvalidHandle)?;
        
        // Check rights
        if !handle.rights.write {
            return Err(ChannelError::PermissionDenied);
        }
        
        // Check endpoint
        if handle.endpoint != ChannelEndpoint::Write {
            return Err(ChannelError::PermissionDenied);
        }
        
        // Check if write endpoint is closed
        if channel.write_endpoint.closed.load(Ordering::SeqCst) {
            return Err(ChannelError::ChannelClosed);
        }
        
        // Check if read endpoint is closed (synchronous channel)
        if channel.flags.synchronous && channel.read_endpoint.closed.load(Ordering::SeqCst) {
            return Err(ChannelError::ChannelClosed);
        }
        
        // Check queue capacity
        let mut read_queue = channel.read_endpoint.message_queue.lock();
        if read_queue.is_full() {
            return Err(ChannelError::ResourceExhausted);
        }
        
        // Add message to read queue
        read_queue.push(message);
        
        println!("Wrote message to channel {}", handle.id);
        
        Ok(())
    }
    
    /// Read a message from a channel
    pub fn read_message(&self, handle: ChannelHandle) -> Result<ChannelMessage, ChannelError> {
        let mut registry = self.channels.lock();
        
        // Find channel
        let channel = registry.channels.iter_mut()
            .find(|(id, _)| *id == handle.id)
            .map(|(_, channel)| channel)
            .ok_or(ChannelError::InvalidHandle)?;
        
        // Check rights
        if !handle.rights.read {
            return Err(ChannelError::PermissionDenied);
        }
        
        // Check endpoint
        if handle.endpoint != ChannelEndpoint::Read {
            return Err(ChannelError::PermissionDenied);
        }
        
        // Check if read endpoint is closed
        if channel.read_endpoint.closed.load(Ordering::SeqCst) {
            return Err(ChannelError::ChannelClosed);
        }
        
        // Get message from queue
        let mut read_queue = channel.read_endpoint.message_queue.lock();
        if read_queue.is_empty() {
            return Err(ChannelError::NoMessages);
        }
        
        let message = read_queue.remove(0);
        
        println!("Read message from channel {}", handle.id);
        
        Ok(message)
    }
    
    /// Close a channel endpoint
    pub fn close_endpoint(&self, handle: ChannelHandle) -> Result<(), ChannelError> {
        let mut registry = self.channels.lock();
        
        // Find channel
        let channel = registry.channels.iter_mut()
            .find(|(id, _)| *id == handle.id)
            .map(|(_, channel)| channel)
            .ok_or(ChannelError::InvalidHandle)?;
        
        // Close appropriate endpoint
        match handle.endpoint {
            ChannelEndpoint::Read => {
                channel.read_endpoint.closed.store(true, Ordering::SeqCst);
                println!("Closed read endpoint of channel {}", handle.id);
            }
            ChannelEndpoint::Write => {
                channel.write_endpoint.closed.store(true, Ordering::SeqCst);
                println!("Closed write endpoint of channel {}", handle.id);
            }
        }
        
        // If both endpoints are closed, remove channel
        if channel.read_endpoint.closed.load(Ordering::SeqCst) && 
           channel.write_endpoint.closed.load(Ordering::SeqCst) {
            // Remove channel from registry
            registry.channels.retain(|(id, _)| *id != handle.id);
            println!("Removed channel {} (both endpoints closed)", handle.id);
        }
        
        Ok(())
    }
}

/// Initialize the channel system
pub fn init() {
    println!("Initializing channel system...");
    
    // Phase 1: Create global channel manager
    // Phase 2: Initialize with proper configuration
    
    println!("Channel system initialized");
}

/// Create a new channel
pub fn create_channel() -> Result<(ChannelHandle, ChannelHandle), ChannelError> {
    let flags = ChannelFlags {
        synchronous: false,
        allow_handle_passing: true,
        peer_to_peer: true,
    };
    
    // Phase 1: Use dummy channel manager
    // Phase 2: Use actual channel manager
    
    let channel_id = NEXT_CHANNEL_ID.fetch_add(1, Ordering::SeqCst);
    
    let read_handle = ChannelHandle {
        id: channel_id,
        rights: ChannelRights {
            read: true,
            write: false,
            duplicate: true,
            transfer: true,
        },
        endpoint: ChannelEndpoint::Read,
    };
    
    let write_handle = ChannelHandle {
        id: channel_id,
        rights: ChannelRights {
            read: false,
            write: true,
            duplicate: true,
            transfer: true,
        },
        endpoint: ChannelEndpoint::Write,
    };
    
    println!("Created channel {} (dummy implementation)", channel_id);
    
    Ok((read_handle, write_handle))
}

/// Write a message to a channel
pub fn write_message(handle: ChannelHandle, data: &[u8]) -> Result<(), ChannelError> {
    if !handle.rights.write {
        return Err(ChannelError::PermissionDenied);
    }
    
    if handle.endpoint != ChannelEndpoint::Write {
        return Err(ChannelError::PermissionDenied);
    }
    
    // Create message
    let mut message_data = arrayvec::ArrayVec::new();
    for &byte in data.iter().take(4096) {
        if message_data.is_full() {
            break;
        }
        message_data.push(byte);
    }
    
    let message = ChannelMessage {
        data: message_data,
        handles: arrayvec::ArrayVec::new(),
        flags: MessageFlags {
            has_handles: false,
            urgent: false,
            require_ack: false,
        },
    };
    
    println!("Wrote {} bytes to channel {}", data.len(), handle.id);
    
    // Phase 1: Dummy implementation
    // Phase 2: Use actual channel manager
    
    Ok(())
}

/// Read a message from a channel
pub fn read_message(handle: ChannelHandle, buffer: &mut [u8]) -> Result<usize, ChannelError> {
    if !handle.rights.read {
        return Err(ChannelError::PermissionDenied);
    }
    
    if handle.endpoint != ChannelEndpoint::Read {
        return Err(ChannelError::PermissionDenied);
    }
    
    // Phase 1: Return dummy data
    // Phase 2: Use actual channel manager
    
    let dummy_data = b"Hello from XPARQ OS!";
    let bytes_to_copy = core::cmp::min(buffer.len(), dummy_data.len());
    
    for (i, &byte) in dummy_data.iter().take(bytes_to_copy).enumerate() {
        buffer[i] = byte;
    }
    
    println!("Read {} bytes from channel {} (dummy implementation)", bytes_to_copy, handle.id);
    
    Ok(bytes_to_copy)
}

/// Close a channel endpoint
pub fn close_endpoint(handle: ChannelHandle) -> Result<(), ChannelError> {
    println!("Closed endpoint of channel {} (dummy implementation)", handle.id);
    
    // Phase 1: Dummy implementation
    // Phase 2: Use actual channel manager
    
    Ok(())
}

impl Default for ChannelRights {
    fn default() -> Self {
        Self {
            read: true,
            write: true,
            duplicate: true,
            transfer: true,
        }
    }
}

impl Default for ChannelFlags {
    fn default() -> Self {
        Self {
            synchronous: false,
            allow_handle_passing: true,
            peer_to_peer: true,
        }
    }
}

impl Default for MessageFlags {
    fn default() -> Self {
        Self {
            has_handles: false,
            urgent: false,
            require_ack: false,
        }
    }
}
