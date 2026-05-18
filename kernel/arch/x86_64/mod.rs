// XPARQ OS - Phase 01: OS & Kernel Foundations
// x86-64 architecture module
// Provides x86-64-specific implementations for bootloader, MMU, and serial

#![no_std]

#[cfg(target_arch = "x86_64")]
pub mod boot;
#[cfg(target_arch = "x86_64")]
pub mod mmu;
#[cfg(target_arch = "x86_64")]
pub mod serial;

// Architecture-specific modules
pub mod cpu;
pub mod interrupts;

// Re-export main functions
#[cfg(target_arch = "x86_64")]
pub use boot::x86_64_entry;
pub use cpu::halt;
pub use interrupts::enable;

/// Early initialization for x86-64
#[cfg(target_arch = "x86_64")]
pub fn early_init(boot_info: &crate::BootInfo) {
    println!("x86-64 early initialization");
    
    // Initialize serial for early debugging
    serial::init();
    
    // Architecture-specific early init
    cpu::early_init();
    
    // Set up interrupt descriptor table
    interrupts::setup_idt();
    
    println!("x86-64 early initialization complete");
}

/// x86-64-specific boot information
#[derive(Debug, Clone, Copy)]
pub struct X86_64BootInfo {
    pub cpu_count: u32,
    pub current_cpu: u32,
    pub tsc_frequency: u64,
    pub serial_base: usize,
    pub rsdp_address: usize,
}

/// Get x86-64 boot information
#[cfg(target_arch = "x86_64")]
pub fn get_boot_info(boot_info: &crate::BootInfo) -> X86_64BootInfo {
    X86_64BootInfo {
        cpu_count: 1, // Phase 1: Single CPU
        current_cpu: 0,
        tsc_frequency: 2000000000, // 2GHz generic TSC
        serial_base: 0x3F8, // COM1 serial base address
        rsdp_address: boot_info.arch_specific.rsdp,
    }
}
