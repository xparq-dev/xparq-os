//! XPARQ OS Kernel - Phase 1: OS & Kernel Foundations
//! 
//! This is the core kernel of XPARQ OS, built on Zircon's object-capability model.
//! It provides the fundamental OS services including process management, memory
//! management, and inter-process communication.
//! 
//! Architecture: Multi-architecture support (ARM64 + x86-64)
//! Security Model: Object-capability based (like Zircon)
//! Memory Model: Virtual Memory Objects (VMO) + VMAR regions
//! 
//! Roadmap Phase: Phase 1 - OS & Kernel Foundations
//! Next Phase: Phase 2 - Dev Environment Setup

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(const_fn_trait_bound)]

// Simple println macro for no_std debugging
macro_rules! println {
    ($($arg:tt)*) => {
        // Phase 1: No output in no_std
        // Phase 2: Use actual console/serial output
        // For now, we'll use a simple serial output implementation
        #[cfg(target_arch = "aarch64")]
        {
            // ARM64 serial output
            use core::fmt::Write;
            let mut serial = crate::arch::serial::SerialPort;
            let _ = write!(serial, $($arg)*);
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            // x86_64 serial output
            use core::fmt::Write;
            let mut serial = crate::arch::serial::SerialPort;
            let _ = write!(serial, $($arg)*);
        }
    };
}

// Core modules
mod arch;
mod memory;
mod objects;
mod syscalls;
mod scheduler;

// Architecture-specific modules
#[cfg(feature = "arm64")]
pub mod arm64;

#[cfg(feature = "x86_64")]
pub mod x86_64;

// Re-export core kernel types
pub use objects::{Handle, Object, ObjectType};
pub use memory::{Vmo, Vmar, MemoryManager};
pub use scheduler::{Thread, Process, Scheduler};

/// XPARQ OS Kernel Entry Point
/// 
/// This is called from the architecture-specific bootloader after basic
/// hardware initialization is complete.
/// 
/// # Arguments
/// * `boot_info` - Architecture-specific boot information
/// 
/// # Returns
/// Never returns on success, panics on failure
#[no_mangle]
pub extern "C" fn xparq_kernel_main(boot_info: &BootInfo) -> ! {
    println!("XPARQ OS Kernel v0.1.0 - Initializing...");
    
    // Initialize architecture-specific components
    arch::init(boot_info);
    
    // Initialize memory manager
    memory::init();
    
    // Initialize object system
    objects::init();
    
    // Initialize scheduler
    scheduler::init();
    
    println!("XPARQ OS Kernel initialization complete");
    
    // Start the scheduler - this never returns
    scheduler::start();
}

/// Boot information structure passed from bootloader
/// 
/// This contains essential information about the system state
/// at boot time, including memory layout and hardware configuration.
#[derive(Debug)]
pub struct BootInfo {
    pub memory_regions: &'static [MemoryRegion],
    pub framebuffer: Option<FramebufferInfo>,
    pub arch_specific: ArchBootInfo,
}

/// Memory region descriptor
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub base: usize,
    pub size: usize,
    pub kind: MemoryRegionKind,
}

/// Memory region types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionKind {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
}

/// Framebuffer information for display initialization
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub address: usize,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
}

/// Pixel format enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelFormat {
    Rgb32,
    Bgr32,
}

/// Architecture-specific boot information
#[derive(Debug)]
#[cfg(feature = "arm64")]
pub struct ArchBootInfo {
    pub device_tree: usize,
    pub cpu_count: u32,
}

/// Architecture-specific boot information
#[derive(Debug)]
#[cfg(feature = "x86_64")]
pub struct ArchBootInfo {
    pub rsdp: usize,
    pub bootloader_brand: &'static str,
}

/// Kernel panic handler
/// 
/// This is called when the kernel encounters an unrecoverable error.
/// It prints diagnostic information and halts the system.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("KERNEL PANIC!");
    println!("Location: {:?}", info.location());
    println!("Message: {}", info);
    
    // Architecture-specific panic handling
    #[cfg(feature = "arm64")]
    arch::arm64::panic_halt();
    
    #[cfg(feature = "x86_64")]
    arch::x86_64::panic_halt();
    
    // Fallback - infinite loop
    loop {
        core::hint::spin_loop();
    }
}

/// Memory allocation error handler
/// 
/// Called when the kernel runs out of memory.
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Memory allocation failed: {:?}", layout);
}

/// Kernel heap allocator
/// 
/// This will be replaced with a proper VMO-based allocator
/// in Phase 3 when we implement the full memory management system.
#[global_allocator]
static ALLOCATOR: DummyAllocator = DummyAllocator;

/// Temporary dummy allocator for Phase 1
/// 
/// In Phase 2, this will be replaced with a proper heap allocator
/// based on VMOs and the Zircon memory model.
struct DummyAllocator;

unsafe impl core::alloc::GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        // For Phase 1, we'll use a simple bump allocator
        // This will be replaced in Phase 2 with proper VMO management
        unimplemented!("Heap allocator not implemented in Phase 1")
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        unimplemented!("Heap deallocation not implemented in Phase 1")
    }
    
    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        let ptr = unsafe { self.alloc(layout) };
        if !ptr.is_null() {
            unsafe { core::ptr::write_bytes(ptr, 0, layout.size()) };
        }
        ptr
    }
    
    unsafe fn realloc(&self, ptr: *mut u8, _layout: core::alloc::Layout, new_size: usize) -> *mut u8 {
        unimplemented!("Heap reallocation not implemented in Phase 1")
    }
}
