// XPARQ OS - Phase 01: OS & Kernel Foundations
// Virtual Memory Address Region (VMAR) implementation
// Provides address space management and memory mapping

#![no_std]

use super::MemoryError;
use super::vmo::{VMO, VMORights};
use bitflags::bitflags;
use core::sync::atomic::{AtomicU64, Ordering};

/// Global VMAR counter for generating unique IDs
static NEXT_VMAR_ID: AtomicU64 = AtomicU64::new(1);

/// VMAR rights - what operations are allowed on this VMAR
#[derive(Debug, Clone, Copy)]
pub struct VMARRights {
    /// Right to map memory in this VMAR
    pub map: bool,
    /// Right to protect memory in this VMAR
    pub protect: bool,
    /// Right to unmap memory in this VMAR
    pub unmap: bool,
    /// Right to destroy this VMAR
    pub destroy: bool,
    /// Right to create child VMARs
    pub create_child: bool,
}

/// VMAR flags
#[derive(Debug, Clone, Copy)]
pub struct VmarFlags {
    /// Whether this VMAR can be resized
    pub resizable: bool,
    /// Whether this VMAR can contain executable mappings
    pub allow_executable: bool,
    /// Whether this VMAR can contain writable mappings
    pub allow_writable: bool,
    /// Whether this VMAR is a specific address range
    pub specific_range: bool,
}

/// Virtual Memory Address Region - represents a region of virtual address space
#[derive(Debug)]
pub struct VMAR {
    /// Unique VMAR identifier
    pub id: u64,
    /// Base virtual address
    pub base: usize,
    /// Size of the VMAR in bytes
    pub size: usize,
    /// Parent VMAR (None for root VMAR)
    pub parent_vmar: Option<u64>,
    /// Rights associated with this VMAR
    pub rights: VMARRights,
    /// VMAR flags
    pub flags: VmarFlags,
    /// Child VMARs
    pub children: arrayvec::ArrayVec<u64, 16>,
    /// Memory mappings
    pub mappings: arrayvec::ArrayVec<Mapping, 64>,
}

/// Memory mapping information
#[derive(Debug, Clone, Copy)]
pub struct Mapping {
    /// Virtual address
    pub virtual_addr: usize,
    /// Size of mapping
    pub size: usize,
    /// VMO being mapped
    pub vmo_id: u64,
    /// VMO offset
    pub vmo_offset: usize,
    /// Mapping rights
    pub rights: VMORights,
    /// Mapping flags
    pub flags: MappingFlags,
}

/// Mapping flags
#[derive(Debug, Clone, Copy)]
pub struct MappingFlags {
    /// Whether this mapping is readable
    pub readable: bool,
    /// Whether this mapping is writable
    pub writable: bool,
    /// Whether this mapping is executable
    pub executable: bool,
    /// Whether this mapping is cacheable
    pub cacheable: bool,
}

/// VMAR information
#[derive(Debug, Clone, Copy)]
pub struct VmarInfo {
    pub id: u64,
    pub base: usize,
    pub size: usize,
    pub flags: VmarFlags,
    pub child_count: u32,
    pub mapping_count: u32,
}

/// VMAR manager - manages all VMARs
pub struct VmarManager {
    /// Global VMAR registry
    vmars: spin::Mutex<VmarRegistry>,
}

/// VMAR registry - stores all VMARs
#[derive(Debug)]
struct VmarRegistry {
    /// Map from VMAR ID to VMAR
    vmars: arrayvec::ArrayVec<(u64, VMAR), 1024>,
    /// Root VMAR ID
    root_vmar_id: u64,
}

impl VmarManager {
    /// Create a new VMAR manager
    pub fn new() -> Self {
        Self {
            vmars: spin::Mutex::new(VmarRegistry {
                vmars: arrayvec::ArrayVec::new(),
                root_vmar_id: 0,
            }),
        }
    }
    
    /// Initialize with root VMAR
    pub fn init(&mut self, base: usize, size: usize) -> Result<(), MemoryError> {
        let mut registry = self.vmars.lock();
        
        // Create root VMAR
        let root_vmar_id = NEXT_VMAR_ID.fetch_add(1, Ordering::SeqCst);
        
        let root_vmar = VMAR {
            id: root_vmar_id,
            base,
            size,
            parent_vmar: None,
            rights: VMARRights {
                map: true,
                protect: true,
                unmap: true,
                destroy: false, // Root VMAR cannot be destroyed
                create_child: true,
            },
            flags: VmarFlags {
                resizable: false,
                allow_executable: true,
                allow_writable: true,
                specific_range: false,
            },
            children: arrayvec::ArrayVec::new(),
            mappings: arrayvec::ArrayVec::new(),
        };
        
        registry.vmars.push((root_vmar_id, root_vmar));
        registry.root_vmar_id = root_vmar_id;
        
        println!("Created root VMAR {} at 0x{:x} with size {} bytes", root_vmar_id, base, size);
        
        Ok(())
    }
    
    /// Create a new VMAR
    pub fn create_vmar(&self, parent_vmar_id: u64, offset: usize, size: usize, flags: VmarFlags) -> Result<VMAR, MemoryError> {
        let mut registry = self.vmars.lock();
        
        let parent_index = registry.vmars.iter().position(|(id, _)| *id == parent_vmar_id)
            .ok_or(MemoryError::InvalidAddress)?;
            
        // Check rights
        if !registry.vmars[parent_index].1.rights.create_child {
            return Err(MemoryError::PermissionDenied);
        }
        
        // Validate offset and size
        if offset + size > registry.vmars[parent_index].1.size {
            return Err(MemoryError::InvalidAddress);
        }
        
        if offset % 4096 != 0 || size % 4096 != 0 {
            return Err(MemoryError::AlignmentError);
        }
        
        // Check for overlap with existing children and mappings
        let parent_vmar = &registry.vmars[parent_index].1;
        
        for child_id in &parent_vmar.children {
            if let Some((_, child_vmar)) = registry.vmars.iter().find(|(id, _)| *id == *child_id) {
                if (offset < child_vmar.base + child_vmar.size) && 
                   (offset + size > child_vmar.base) {
                    return Err(MemoryError::AlreadyMapped);
                }
            }
        }
        
        for mapping in &parent_vmar.mappings {
            if (offset < mapping.virtual_addr + mapping.size) && 
               (offset + size > mapping.virtual_addr) {
                return Err(MemoryError::AlreadyMapped);
            }
        }
        
        let parent_base = parent_vmar.base;
        
        // Create VMAR
        let vmar_id = NEXT_VMAR_ID.fetch_add(1, Ordering::SeqCst);
        
        let vmar = VMAR {
            id: vmar_id,
            base: parent_base + offset,
            size,
            parent_vmar: Some(parent_vmar_id),
            rights: VMARRights {
                map: true,
                protect: true,
                unmap: true,
                destroy: true,
                create_child: true,
            },
            flags,
            children: arrayvec::ArrayVec::new(),
            mappings: arrayvec::ArrayVec::new(),
        };
        
        // Add to parent's children
        if registry.vmars[parent_index].1.children.is_full() {
            return Err(MemoryError::ResourceExhausted);
        }
        registry.vmars[parent_index].1.children.push(vmar_id);
        
        // Register VMAR
        if registry.vmars.is_full() {
            return Err(MemoryError::ResourceExhausted);
        }
        registry.vmars.push((vmar_id, vmar));
        
        println!("Created VMAR {} at 0x{:x} with size {} bytes", vmar_id, parent_base + offset, size);
        
        // Return a copy of the VMAR
        Ok(VMAR {
            id: vmar_id,
            base: parent_base + offset,
            size,
            parent_vmar: Some(parent_vmar_id),
            rights: VMARRights {
                map: true,
                protect: true,
                unmap: true,
                destroy: true,
                create_child: true,
            },
            flags,
            children: arrayvec::ArrayVec::new(),
            mappings: arrayvec::ArrayVec::new(),
        })
    }
    
    /// Map a VMO into a VMAR
    pub fn map_vmo(&self, vmar_id: u64, vmo: &VMO, vmar_offset: usize, vmo_offset: usize, size: usize, rights: VMORights) -> Result<usize, MemoryError> {
        let mut registry = self.vmars.lock();
        
        let vmar_index = registry.vmars.iter().position(|(id, _)| *id == vmar_id)
            .ok_or(MemoryError::InvalidAddress)?;
            
        // Check rights
        if !registry.vmars[vmar_index].1.rights.map {
            return Err(MemoryError::PermissionDenied);
        }
        
        // Validate parameters
        if vmar_offset + size > registry.vmars[vmar_index].1.size {
            return Err(MemoryError::InvalidAddress);
        }
        
        if vmo_offset + size > vmo.size {
            return Err(MemoryError::InvalidAddress);
        }
        
        if vmar_offset % 4096 != 0 || size % 4096 != 0 {
            return Err(MemoryError::AlignmentError);
        }
        
        let vmar = &registry.vmars[vmar_index].1;
        let vmar_base = vmar.base;
        
        // Check for overlap
        for child_id in &vmar.children {
            if let Some((_, child_vmar)) = registry.vmars.iter().find(|(id, _)| *id == *child_id) {
                if (vmar_offset < child_vmar.base - vmar_base + child_vmar.size) && 
                   (vmar_offset + size > child_vmar.base - vmar_base) {
                    return Err(MemoryError::AlreadyMapped);
                }
            }
        }
        
        for mapping in &vmar.mappings {
            if (vmar_offset < mapping.virtual_addr - vmar_base + mapping.size) && 
               (vmar_offset + size > mapping.virtual_addr - vmar_base) {
                return Err(MemoryError::AlreadyMapped);
            }
        }
        
        // Create mapping
        let mapping = Mapping {
            virtual_addr: vmar_base + vmar_offset,
            size,
            vmo_id: vmo.id,
            vmo_offset,
            rights,
            flags: MappingFlags {
                readable: rights.read,
                writable: rights.write,
                executable: rights.execute,
                cacheable: true,
            },
        };
        
        // Add mapping
        if registry.vmars[vmar_index].1.mappings.is_full() {
            return Err(MemoryError::ResourceExhausted);
        }
        registry.vmars[vmar_index].1.mappings.push(mapping);
        
        println!("Mapped VMO {} into VMAR {} at 0x{:x} with size {} bytes", vmo.id, vmar_id, vmar_base + vmar_offset, size);
        
        Ok(vmar_base + vmar_offset)
    }
    
    /// Unmap memory from a VMAR
    pub fn unmap(&self, vmar_id: u64, addr: usize, size: usize) -> Result<(), MemoryError> {
        let mut registry = self.vmars.lock();
        
        // Find VMAR
        let vmar = registry.vmars.iter_mut()
            .find(|(id, _)| *id == vmar_id)
            .map(|(_, vmar)| vmar)
            .ok_or(MemoryError::InvalidAddress)?;
        
        // Check rights
        if !vmar.rights.unmap {
            return Err(MemoryError::PermissionDenied);
        }
        
        // Find and remove mapping
        for (i, mapping) in vmar.mappings.iter().enumerate() {
            if mapping.virtual_addr == addr && mapping.size == size {
                vmar.mappings.remove(i);
                println!("Unmapped memory at 0x{:x} with size {} bytes from VMAR {}", addr, size, vmar_id);
                return Ok(());
            }
        }
        
        Err(MemoryError::NotMapped)
    }
    
    /// Destroy a VMAR
    pub fn destroy_vmar(&self, vmar_id: u64) -> Result<(), MemoryError> {
        let mut registry = self.vmars.lock();
        
        // Find VMAR
        let vmar_index = registry.vmars.iter()
            .enumerate()
            .find(|(i, (id, _))| *id == vmar_id)
            .map(|(i, _)| i)
            .ok_or(MemoryError::InvalidAddress)?;
        
        let vmar = &registry.vmars[vmar_index].1;
        
        // Check rights
        if !vmar.rights.destroy {
            return Err(MemoryError::PermissionDenied);
        }
        
        // Check if VMAR has children or mappings
        if !vmar.children.is_empty() || !vmar.mappings.is_empty() {
            return Err(MemoryError::AlreadyMapped);
        }
        
        // Remove from parent's children
        if let Some(parent_id) = vmar.parent_vmar {
            if let Some((_, parent_vmar)) = registry.vmars.iter_mut().find(|(id, _)| *id == parent_id) {
                if let Some(child_index) = parent_vmar.children.iter().position(|&id| id == vmar_id) {
                    parent_vmar.children.remove(child_index);
                }
            }
        }
        
        // Remove VMAR
        registry.vmars.remove(vmar_index);
        println!("Destroyed VMAR {}", vmar_id);
        
        Ok(())
    }
}

/// Initialize the VMAR system
pub fn init(boot_info: &crate::BootInfo) {
    println!("Initializing VMAR system...");
    
    // Phase 1: Create root VMAR covering entire address space
    // Phase 2: Use actual memory regions from boot info
    
    let root_base = 0x100000; // 1MB start (avoid low memory)
    let root_size = 0x80000000usize; // 2GB address space for Phase 1
    
    println!("VMAR system initialized");
}

/// Create a new VMAR
pub fn create_vmar(parent_vmar: u64, offset: usize, size: usize) -> Result<VMAR, MemoryError> {
    let flags = VmarFlags {
        resizable: false,
        allow_executable: true,
        allow_writable: true,
        specific_range: false,
    };
    
    // Phase 1: Use dummy VMAR manager
    // Phase 2: Use actual VMAR manager
    
    let vmar_id = NEXT_VMAR_ID.fetch_add(1, Ordering::SeqCst);
    
    Ok(VMAR {
        id: vmar_id,
        base: 0x200000 + offset, // Dummy base address
        size,
        parent_vmar: Some(parent_vmar),
        rights: VMARRights {
            map: true,
            protect: true,
            unmap: true,
            destroy: true,
            create_child: true,
        },
        flags,
        children: arrayvec::ArrayVec::new(),
        mappings: arrayvec::ArrayVec::new(),
    })
}

/// Map a VMO into a VMAR
pub fn map_vmo(vmar: &mut VMAR, vmo: &VMO, offset: usize, vmo_offset: usize, size: usize) -> Result<usize, MemoryError> {
    if !vmar.rights.map {
        return Err(MemoryError::PermissionDenied);
    }
    
    if offset + size > vmar.size {
        return Err(MemoryError::InvalidAddress);
    }
    
    if vmo_offset + size > vmo.size {
        return Err(MemoryError::InvalidAddress);
    }
    
    // Create mapping
    let mapping = Mapping {
        virtual_addr: vmar.base + offset,
        size,
        vmo_id: vmo.id,
        vmo_offset,
        rights: vmo.rights,
        flags: MappingFlags {
            readable: vmo.rights.read,
            writable: vmo.rights.write,
            executable: vmo.rights.execute,
            cacheable: true,
        },
    };
    
    // Add mapping
    if vmar.mappings.is_full() {
        return Err(MemoryError::ResourceExhausted);
    }
    vmar.mappings.push(mapping);
    
    println!("Mapped VMO {} into VMAR {} at offset {} with size {} bytes", vmo.id, vmar.id, offset, size);
    
    Ok(vmar.base + offset)
}

impl Default for VMARRights {
    fn default() -> Self {
        Self {
            map: true,
            protect: true,
            unmap: true,
            destroy: true,
            create_child: true,
        }
    }
}

impl Default for VmarFlags {
    fn default() -> Self {
        Self {
            resizable: false,
            allow_executable: true,
            allow_writable: true,
            specific_range: false,
        }
    }
}

impl Default for MappingFlags {
    fn default() -> Self {
        Self {
            readable: true,
            writable: false,
            executable: false,
            cacheable: true,
        }
    }
}
