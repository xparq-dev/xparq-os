// XPARQ OS - Phase 20.5: Unified Kernel Object Model
// A no_alloc capability-based object model using Enum variants

use crate::sync::IrqSafeMutex;
use bitflags::bitflags;
use core::sync::atomic::{AtomicU32, Ordering};
use crate::fs::vfs::VNodeInfo;

/// Object handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Handle {
    value: u32,
}

impl Handle {
    pub const fn new(object_id: u32, rights: HandleRights) -> Self {
        let handle_value = (object_id << 16) | rights.bits();
        Self { value: handle_value }
    }
    
    pub fn object_id(&self) -> u32 {
        self.value >> 16
    }
    
    pub fn rights(&self) -> HandleRights {
        HandleRights::from_bits_truncate(self.value & 0xFFFF)
    }
    
    pub const INVALID: Self = Self { value: 0 };
    
    pub fn is_valid(&self) -> bool {
        self.value != 0
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct HandleRights: u32 {
        const READ = 0x0001;
        const WRITE = 0x0002;
        const EXECUTE = 0x0004;
        const DUPLICATE = 0x0008;
        const WAIT = 0x0020;
        const SIGNAL = 0x0040;
        const BASIC = Self::READ.bits() | Self::WRITE.bits();
        const ALL = 0xFFFF;
    }
}

/// Unified Object Variants
#[derive(Debug, Clone)]
pub enum ObjectVariant {
    None,
    File(VNodeInfo),
    UdpSocket(usize),
    TcpSocket(usize),
    // Future: Thread, Process, Event, Timer, etc.
}

/// A slot in the object pool
pub struct ObjectSlot {
    pub ref_count: AtomicU32,
    pub variant: ObjectVariant,
}

impl ObjectSlot {
    pub const fn empty() -> Self {
        Self {
            ref_count: AtomicU32::new(0),
            variant: ObjectVariant::None,
        }
    }
}

pub struct ObjectPool {
    pub slots: [ObjectSlot; 256],
    pub next_id: u32,
}

impl ObjectPool {
    pub const fn new() -> Self {
        Self {
            slots: [const { ObjectSlot::empty() }; 256],
            next_id: 1,
        }
    }

    /// Allocates an object, returning its ID
    pub fn allocate(&mut self, variant: ObjectVariant) -> Option<u32> {
        for i in 1..256 {
            if matches!(self.slots[i].variant, ObjectVariant::None) {
                self.slots[i].variant = variant;
                self.slots[i].ref_count.store(1, Ordering::SeqCst);
                let id = self.next_id;
                self.next_id = self.next_id.wrapping_add(1);
                // Store actual index in the upper bits of ID? 
                // Or just use index as ID for now.
                return Some(i as u32);
            }
        }
        None
    }

    /// Increments reference count
    pub fn add_ref(&self, id: u32) {
        let idx = id as usize;
        if idx < 256 {
            self.slots[idx].ref_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Decrements reference count and drops object if 0
    pub fn release(&mut self, id: u32) {
        let idx = id as usize;
        if idx < 256 {
            let prev = self.slots[idx].ref_count.fetch_sub(1, Ordering::SeqCst);
            if prev == 1 {
                self.slots[idx].variant = ObjectVariant::None;
            }
        }
    }

    /// Gets a cloned variant if valid
    pub fn get_variant(&self, id: u32) -> Option<ObjectVariant> {
        let idx = id as usize;
        if idx < 256 && self.slots[idx].ref_count.load(Ordering::SeqCst) > 0 {
            Some(self.slots[idx].variant.clone())
        } else {
            None
        }
    }
}

pub static OBJECT_POOL: IrqSafeMutex<ObjectPool> = IrqSafeMutex::new(ObjectPool::new());
