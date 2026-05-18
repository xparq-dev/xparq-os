// XPARQ OS - Phase 01: OS & Kernel Foundations
// ARM64 architecture module
// Provides ARM64-specific implementations for bootloader, MMU, and UART

#![no_std]

#[cfg(target_arch = "aarch64")]
pub mod boot;
#[cfg(target_arch = "aarch64")]
pub mod mmu;
#[cfg(target_arch = "aarch64")]
pub mod uart;

// Architecture-specific modules
pub mod cpu;
pub mod interrupts;

// Re-export main functions
#[cfg(target_arch = "aarch64")]
pub use boot::arm64_entry;
pub use cpu::halt;
pub use interrupts::enable;

/// Early initialization for ARM64
#[cfg(target_arch = "aarch64")]
pub fn early_init(boot_info: &crate::BootInfo) {
    println!("ARM64 early initialization");
    
    // Initialize UART for early debugging
    uart::init();
    
    // Architecture-specific early init
    cpu::early_init();
    
    // Set up exception vectors
    interrupts::setup_vectors();
    
    println!("ARM64 early initialization complete");
}

/// ARM64-specific boot information
#[derive(Debug, Clone, Copy)]
pub struct Arm64BootInfo {
    pub cpu_count: u32,
    pub current_cpu: u32,
    pub timer_frequency: u64,
    pub uart_base: usize,
}

/// Get ARM64 boot information
#[cfg(target_arch = "aarch64")]
pub fn get_boot_info(boot_info: &crate::BootInfo) -> Arm64BootInfo {
    Arm64BootInfo {
        cpu_count: 1, // Phase 1: Single CPU
        current_cpu: 0,
        timer_frequency: 1000000, // 1MHz generic timer
        uart_base: 0x9000000, // PL011 UART base address
    }
}
