// XPARQ OS - Phase 01: OS & Kernel Foundations
// Memory management module
// Implements VMO (Virtual Memory Object) and VMAR (Virtual Memory Address Region)

#![no_std]

pub mod vmo;
pub mod vmar;

// Re-export main types
pub use vmo::{VMO, VMORights, VmoInfo};
pub use vmar::{VMAR, VMARRights, VmarInfo};

/// Initialize memory management system
pub fn init(boot_info: &crate::BootInfo) {
    println!("Initializing memory management...");
    
    // Initialize VMO system
    vmo::init(boot_info);
    
    // Initialize VMAR system
    vmar::init(boot_info);
    
    println!("Memory management initialized");
}

/// Memory management errors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryError {
    /// Out of memory
    OutOfMemory,
    /// Invalid address
    InvalidAddress,
    /// Permission denied
    PermissionDenied,
    /// Already mapped
    AlreadyMapped,
    /// Not mapped
    NotMapped,
    /// Invalid size
    InvalidSize,
    /// Alignment error
    AlignmentError,
}
