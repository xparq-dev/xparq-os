// XPARQ OS - Phase 01: OS & Kernel Foundations
// Virtual Memory Object (VMO) implementation
// Provides memory allocation and sharing capabilities

#![no_std]

use super::MemoryError;
use bitflags::bitflags;
use core::sync::atomic::{AtomicU64, Ordering};

/// Global VMO counter for generating unique IDs
static NEXT_VMO_ID: AtomicU64 = AtomicU64::new(1);

/// VMO rights - what operations are allowed on this VMO
#[derive(Debug, Clone, Copy)]
pub struct VMORights {
    /// Right to read from the VMO
    pub read: bool,
    /// Right to write to the VMO
    pub write: bool,
    /// Right to execute from the VMO
    pub execute: bool,
    /// Right to duplicate the VMO
    pub duplicate: bool,
    /// Right to transfer the VMO
    pub transfer: bool,
}

/// Virtual Memory Object - represents a contiguous range of physical memory
#[derive(Debug)]
pub struct VMO {
    /// Unique VMO identifier
    pub id: u64,
    /// Size of the VMO in bytes
    pub size: usize,
    /// Physical address (None for unmapped VMOs)
    pub physical_addr: Option<usize>,
    /// Rights associated with this VMO
    pub rights: VMORights,
    /// Reference count
    pub ref_count: core::sync::atomic::AtomicU32,
    /// VMO flags
    pub flags: VmoFlags,
}

/// VMO flags
#[derive(Debug, Clone, Copy)]
pub struct VmoFlags {
    /// Whether this VMO is resizable
    pub resizable: bool,
    /// Whether this VMO is contiguous in physical memory
    pub contiguous: bool,
    /// Whether this VMO is cacheable
    pub cacheable: bool,
    /// Whether this VMO is device memory
    pub device_memory: bool,
}

/// VMO information
#[derive(Debug, Clone, Copy)]
pub struct VmoInfo {
    pub id: u64,
    pub size: usize,
    pub flags: VmoFlags,
    pub ref_count: u32,
    pub committed_bytes: usize,
}

/// VMO manager - manages all VMOs
pub struct VmoManager {
    /// Global VMO registry
    vmos: spin::Mutex<VmoRegistry>,
}

/// VMO registry - stores all VMOs
#[derive(Debug)]
struct VmoRegistry {
    /// Map from VMO ID to VMO
    vmos: arrayvec::ArrayVec<(u64, VMO), 1024>,
    /// Total committed memory
    committed_bytes: usize,
    /// Total available memory
    available_bytes: usize,
}

impl VmoManager {
    /// Create a new VMO manager
    pub fn new() -> Self {
        Self {
            vmos: spin::Mutex::new(VmoRegistry {
                vmos: arrayvec::ArrayVec::new(),
                committed_bytes: 0,
                available_bytes: 2 * 1024 * 1024 * 1024, // 2GB for Phase 1
            }),
        }
    }
    
    /// Create a new VMO
    pub fn create_vmo(&self, size: usize, flags: VmoFlags) -> Result<VMO, MemoryError> {
        // Validate size
        if size == 0 || size % 4096 != 0 {
            return Err(MemoryError::InvalidSize);
        }
        
        let mut registry = self.vmos.lock();
        
        // Check available memory
        if size > registry.available_bytes - registry.committed_bytes {
            return Err(MemoryError::OutOfMemory);
        }
        
        // Create VMO
        let vmo_id = NEXT_VMO_ID.fetch_add(1, Ordering::SeqCst);
        
        let vmo = VMO {
            id: vmo_id,
            size,
            physical_addr: None, // Phase 1: No physical allocation
            rights: VMORights {
                read: true,
                write: true,
                execute: false,
                duplicate: true,
                transfer: true,
            },
            ref_count: core::sync::atomic::AtomicU32::new(1),
            flags,
        };
        
        // Register VMO
        if registry.vmos.is_full() {
            return Err(MemoryError::OutOfMemory);
        }
        
        registry.vmos.push((vmo_id, vmo));
        
        println!("Created VMO {} with size {} bytes", vmo_id, size);
        
        // Return a copy of the VMO
        Ok(VMO {
            id: vmo_id,
            size,
            physical_addr: None,
            rights: VMORights {
                read: true,
                write: true,
                execute: false,
                duplicate: true,
                transfer: true,
            },
            ref_count: core::sync::atomic::AtomicU32::new(1),
            flags,
        })
    }
    
    /// Get VMO by ID
    pub fn get_vmo(&self, vmo_id: u64) -> Option<VMO> {
        let registry = self.vmos.lock();
        
        for (id, vmo) in &registry.vmos {
            if *id == vmo_id {
                return Some(VMO {
                    id: vmo.id,
                    size: vmo.size,
                    physical_addr: vmo.physical_addr,
                    rights: vmo.rights,
                    ref_count: core::sync::atomic::AtomicU32::new(
                        vmo.ref_count.load(Ordering::SeqCst)
                    ),
                    flags: vmo.flags,
                });
            }
        }
        
        None
    }
    
    /// Duplicate a VMO with potentially reduced rights
    pub fn duplicate_vmo(&self, vmo_id: u64, rights: VMORights) -> Result<VMO, MemoryError> {
        let mut registry = self.vmos.lock();
        
        for (id, vmo) in &mut registry.vmos {
            if *id == vmo_id {
                // Check if current rights allow duplication
                if !vmo.rights.duplicate {
                    return Err(MemoryError::PermissionDenied);
                }
                
                // Increase reference count
                vmo.ref_count.fetch_add(1, Ordering::SeqCst);
                
                return Ok(VMO {
                    id: vmo.id,
                    size: vmo.size,
                    physical_addr: vmo.physical_addr,
                    rights,
                    ref_count: core::sync::atomic::AtomicU32::new(
                        vmo.ref_count.load(Ordering::SeqCst)
                    ),
                    flags: vmo.flags,
                });
            }
        }
        
        Err(MemoryError::InvalidAddress)
    }
    
    /// Close a VMO
    pub fn close_vmo(&self, vmo_id: u64) -> Result<(), MemoryError> {
        let mut registry = self.vmos.lock();
        
        for (i, (id, vmo)) in registry.vmos.iter().enumerate() {
            if *id == vmo_id {
                // Decrease reference count
                let old_count = vmo.ref_count.fetch_sub(1, Ordering::SeqCst);
                
                if old_count == 1 {
                    // Last reference, remove VMO
                    registry.vmos.remove(i);
                    println!("Closed VMO {}", vmo_id);
                }
                
                return Ok(());
            }
        }
        
        Err(MemoryError::InvalidAddress)
    }
    
    /// Resize a VMO
    pub fn resize_vmo(&self, vmo_id: u64, new_size: usize) -> Result<(), MemoryError> {
        let mut registry = self.vmos.lock();
        
        for (id, vmo) in &mut registry.vmos {
            if *id == vmo_id {
                if !vmo.flags.resizable {
                    return Err(MemoryError::PermissionDenied);
                }
                
                if new_size % 4096 != 0 {
                    return Err(MemoryError::InvalidSize);
                }
                
                let size_diff = new_size as isize - vmo.size as isize;
                
                if size_diff > 0 {
                    // Growing VMO
                    if (size_diff as usize) > registry.available_bytes - registry.committed_bytes {
                        return Err(MemoryError::OutOfMemory);
                    }
                    
                    registry.committed_bytes += size_diff as usize;
                } else {
                    // Shrinking VMO
                    registry.committed_bytes -= (-size_diff) as usize;
                }
                
                vmo.size = new_size;
                println!("Resized VMO {} to {} bytes", vmo_id, new_size);
                
                return Ok(());
            }
        }
        
        Err(MemoryError::InvalidAddress)
    }
    
    /// Get VMO information
    pub fn get_vmo_info(&self, vmo_id: u64) -> Option<VmoInfo> {
        let registry = self.vmos.lock();
        
        for (id, vmo) in &registry.vmos {
            if *id == vmo_id {
                return Some(VmoInfo {
                    id: vmo.id,
                    size: vmo.size,
                    flags: vmo.flags,
                    ref_count: vmo.ref_count.load(Ordering::SeqCst),
                    committed_bytes: 0, // Phase 1: No physical allocation
                });
            }
        }
        
        None
    }
}

/// Initialize the VMO system
pub fn init(boot_info: &crate::BootInfo) {
    println!("Initializing VMO system...");
    
    // Phase 1: Create global VMO manager
    // Phase 2: Initialize with actual memory from boot info
    
    println!("VMO system initialized");
}

/// Create a new VMO
pub fn create_vmo(size: usize) -> Result<VMO, MemoryError> {
    let flags = VmoFlags {
        resizable: false,
        contiguous: false,
        cacheable: true,
        device_memory: false,
    };
    
    // Phase 1: Use dummy VMO manager
    // Phase 2: Use actual VMO manager
    
    let vmo_id = NEXT_VMO_ID.fetch_add(1, Ordering::SeqCst);
    
    Ok(VMO {
        id: vmo_id,
        size,
        physical_addr: None,
        rights: VMORights {
            read: true,
            write: true,
            execute: false,
            duplicate: true,
            transfer: true,
        },
        ref_count: core::sync::atomic::AtomicU32::new(1),
        flags,
    })
}

/// Duplicate a VMO
pub fn duplicate_vmo(vmo: &VMO, rights: VMORights) -> Result<VMO, MemoryError> {
    if !vmo.rights.duplicate {
        return Err(MemoryError::PermissionDenied);
    }
    
    Ok(VMO {
        id: vmo.id,
        size: vmo.size,
        physical_addr: vmo.physical_addr,
        rights,
        ref_count: core::sync::atomic::AtomicU32::new(
            vmo.ref_count.load(Ordering::SeqCst) + 1
        ),
        flags: vmo.flags,
    })
}

/// Read from VMO
pub fn read_vmo(vmo: &VMO, offset: usize, buffer: &mut [u8]) -> Result<(), MemoryError> {
    if !vmo.rights.read {
        return Err(MemoryError::PermissionDenied);
    }
    
    if offset + buffer.len() > vmo.size {
        return Err(MemoryError::InvalidAddress);
    }
    
    // Phase 1: Fill buffer with zeros
    // Phase 2: Read from actual memory
    
    for byte in buffer.iter_mut() {
        *byte = 0;
    }
    
    println!("Read {} bytes from VMO {} at offset {}", buffer.len(), vmo.id, offset);
    
    Ok(())
}

/// Write to VMO
pub fn write_vmo(vmo: &mut VMO, offset: usize, data: &[u8]) -> Result<(), MemoryError> {
    if !vmo.rights.write {
        return Err(MemoryError::PermissionDenied);
    }
    
    if offset + data.len() > vmo.size {
        return Err(MemoryError::InvalidAddress);
    }
    
    // Phase 1: Do nothing (no actual memory)
    // Phase 2: Write to actual memory
    
    println!("Wrote {} bytes to VMO {} at offset {}", data.len(), vmo.id, offset);
    
    Ok(())
}

impl Default for VMORights {
    fn default() -> Self {
        Self {
            read: true,
            write: true,
            execute: false,
            duplicate: true,
            transfer: true,
        }
    }
}

impl Default for VmoFlags {
    fn default() -> Self {
        Self {
            resizable: false,
            contiguous: false,
            cacheable: true,
            device_memory: false,
        }
    }
}
