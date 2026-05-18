//! ARM64 Bootloader - Phase 2: Dev Environment Setup
//! 
//! This module provides the ARM64 bootloader entry point for XPARQ OS.
//! It handles the transition from firmware/bootloader to the kernel,
//! including:
//! - Early assembly entry point
//! - CPU state initialization
//! - Stack setup
//! - Memory layout preparation
//! - Jump to kernel main
//! 
//! Entry Point: arm64_entry (assembly) -> boot_main (Rust)
//! Exception Level: EL1 (Kernel mode)
//! Stack: 16KB per CPU
//! Memory: Identity-mapped for early boot
//! 
//! Roadmap Phase: Phase 2 - Dev Environment Setup

use super::{BootInfo, sysreg, asm_utils};

/// ARM64 bootloader entry point (assembly)
/// 
/// This is the first code that runs on ARM64. It's called from the
/// bootloader/firmware and performs minimal setup before calling
/// boot_main.
/// 
/// Stack: Uses temporary stack at 0x40000000
/// Registers: Preserves x0-x3 for boot arguments
#[no_mangle]
#[naked]
pub extern "C" fn arm64_entry() -> ! {
    unsafe {
        core::arch::asm!(
            // Set up temporary stack
            "adr x1, __boot_stack_top",
            "mov sp, x1",
            
            // Save boot arguments in x0-x3
            // x0: Device tree pointer
            // x1: CPU count (will be overwritten by stack setup)
            // x2: Reserved
            // x3: Reserved
            
            // Jump to Rust boot main
            "b {boot_main}",
            boot_main = sym boot_main,
            options(noreturn)
        );
    }
}

/// Rust bootloader main function
/// 
/// This is called from the assembly entry point with the following
/// arguments in registers:
/// - x0: Device tree pointer
/// - x1: CPU count (from bootloader)
/// - x2: Reserved (0)
/// - x3: Reserved (0)
#[no_mangle]
pub extern "C" fn boot_main() -> ! {
    // Get boot arguments from registers
    let dt_ptr: usize;
    let cpu_count: u32;
    let _reserved2: u64;
    let _reserved3: u64;
    
    unsafe {
        core::arch::asm!(
            "",
            out("x0") dt_ptr,
            out("x1") cpu_count,
            out("x2") _reserved2,
            out("x3") _reserved3,
        );
    }
    
    // Early debug output (UART)
    super::console::early_init();
    
    println!("XPARQ OS ARM64 Bootloader v0.1.0");
    println!("Device tree: 0x{:x}, CPU count: {}", dt_ptr, cpu_count);
    
    // Validate we're running at EL1
    let current_el = sysreg::current_el();
    if current_el != 1 {
        panic!("Running at EL{} instead of EL1", current_el);
    }
    
    // Initialize ARM64 architecture
    let boot_info = super::process_boot_info(&crate::ArchBootInfo {
        device_tree: dt_ptr,
        cpu_count,
    });
    
    // Set up per-CPU data
    setup_percpu_data();
    
    // Initialize memory management (identity mapping first)
    super::memory::setup_identity_mapping();
    
    // Enable caches and MMU
    enable_caches_and_mmu();
    
    // Jump to kernel main
    println!("Jumping to XPARQ OS kernel...");
    
    let kernel_main: extern "C" fn(&crate::BootInfo) -> ! = crate::xparq_kernel_main;
    kernel_main(&crate::BootInfo {
        memory_regions: boot_info.memory_regions,
        framebuffer: None, // Will be set up in Phase 3
        arch_specific: crate::ArchBootInfo {
            device_tree: dt_ptr,
            cpu_count,
        },
    });
}

/// Set up per-CPU data structures
fn setup_percpu_data() {
    // Phase 1: Basic per-CPU setup
    // Phase 2: Full per-CPU data with SMP support
    
    println!("Setting up per-CPU data...");
    
    // Get current CPU ID (assumes CPU 0 for now)
    let cpu_id = 0;
    
    // Set TPIDR_EL1 (thread pointer) for per-CPU data
    // Phase 2: Point to actual per-CPU data structure
    sysreg::msr("TPIDR_EL1", cpu_id as u64);
    
    println!("Per-CPU data setup complete for CPU {}", cpu_id);
}

/// Enable caches and MMU
fn enable_caches_and_mmu() {
    println!("Enabling caches and MMU...");
    
    // Ensure memory barriers before enabling MMU
    asm_utils::dsb();
    asm_utils::isb();
    
    // Enable MMU with identity mapping
    sysreg::set_sctlr(true);
    
    // Invalidate TLB
    sysreg::tlbialle1is();
    
    // Ensure memory barriers after enabling MMU
    asm_utils::dsb();
    asm_utils::isb();
    
    println!("Caches and MMU enabled");
}

/// Boot stack definitions
/// 
/// Each CPU gets its own 16KB stack for early boot
#[link_section = ".bss.boot_stack"]
static mut BOOT_STACK: [u8; 16 * 1024] = [0; 16 * 1024];

/// Boot stack top symbol (referenced by assembly)
#[no_mangle]
#[link_section = ".bss.boot_stack"]
static mut __boot_stack_top: u8 = 0;

/// Boot linker script symbols
extern "C" {
    static __boot_stack_start: u8;
    static __boot_stack_end: u8;
}

/// Validate boot stack setup
fn validate_boot_stack() {
    let stack_start = unsafe { &__boot_stack_start as *const _ as usize };
    let stack_end = unsafe { &__boot_stack_end as *const _ as usize };
    let stack_size = stack_end - stack_start;
    
    println!("Boot stack: 0x{:x} - 0x{:x} ({} bytes)", 
             stack_start, stack_end, stack_size);
    
    if stack_size != 16 * 1024 {
        panic!("Boot stack size mismatch: expected 16KB, got {} bytes", stack_size);
    }
}

/// Memory layout definitions
pub mod layout {
    /// Kernel load address
    pub const KERNEL_LOAD_ADDR: usize = 0x40080000;
    
    /// Boot stack address
    pub const BOOT_STACK_ADDR: usize = 0x40000000;
    
    /// Boot stack size
    pub const BOOT_STACK_SIZE: usize = 16 * 1024;
    
    /// Device tree maximum size
    pub const DEVICE_TREE_MAX_SIZE: usize = 64 * 1024;
    
    /// Early memory map size
    pub const EARLY_MAP_SIZE: usize = 4 * 1024 * 1024; // 4MB
}

/// Boot error handling
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("BOOT PANIC!");
    println!("Location: {:?}", info.location());
    println!("Message: {}", info);
    
    // Disable interrupts and halt
    super::interrupts::disable();
    super::cpu::halt();
    
    loop {
        core::hint::spin_loop();
    }
}

/// Boot validation functions
pub mod validation {
    /// Validate device tree structure
    pub fn validate_device_tree(dt_ptr: usize) -> Result<(), &'static str> {
        // Phase 1: Basic validation
        // Phase 2: Full device tree parsing and validation
        
        // Check alignment
        if dt_ptr & 0x7 != 0 {
            return Err("Device tree not 8-byte aligned");
        }
        
        // Check magic number
        let dt_magic = unsafe { core::ptr::read_volatile(dt_ptr as *const u32) };
        if dt_magic != 0xd00dfeed {
            return Err("Invalid device tree magic");
        }
        
        // Check size
        let dt_size = unsafe { core::ptr::read_volatile((dt_ptr + 4) as *const u32) };
        if dt_size > super::layout::DEVICE_TREE_MAX_SIZE as u32 {
            return Err("Device tree too large");
        }
        
        println!("Device tree validation passed: {} bytes", dt_size);
        Ok(())
    }
    
    /// Validate CPU count
    pub fn validate_cpu_count(cpu_count: u32) -> Result<(), &'static str> {
        if cpu_count == 0 || cpu_count > 256 {
            return Err("Invalid CPU count");
        }
        
        println!("CPU count validation passed: {} CPUs", cpu_count);
        Ok(())
    }
    
    /// Validate memory layout
    pub fn validate_memory_layout() -> Result<(), &'static str> {
        // Phase 1: Basic layout validation
        // Phase 2: Full memory map validation
        
        let kernel_start = super::layout::KERNEL_LOAD_ADDR;
        let stack_end = super::layout::BOOT_STACK_ADDR + super::layout::BOOT_STACK_SIZE;
        
        if kernel_start < stack_end {
            return Err("Kernel overlaps with boot stack");
        }
        
        println!("Memory layout validation passed");
        Ok(())
    }
}

/// Boot configuration
pub mod config {
    /// Boot configuration options
    #[derive(Debug, Clone, Copy)]
    pub struct BootConfig {
        /// Enable early debug output
        pub early_debug: bool,
        /// Enable MMU during boot
        pub enable_mmu: bool,
        /// Enable caches during boot
        pub enable_caches: bool,
        /// Boot verbosity level
        pub verbosity: u32,
    }
    
    /// Default boot configuration
    pub const DEFAULT_CONFIG: BootConfig = BootConfig {
        early_debug: true,
        enable_mmu: true,
        enable_caches: true,
        verbosity: 1,
    };
    
    /// Get current boot configuration
    pub fn get_config() -> BootConfig {
        // Phase 2: Read from device tree or configuration registers
        DEFAULT_CONFIG
    }
}
