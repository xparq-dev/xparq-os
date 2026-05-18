//! Architecture Abstraction Layer - Phase 1: OS & Kernel Foundations
//! 
//! This module provides architecture-agnostic interfaces for hardware-specific
//! operations. It abstracts the differences between ARM64 and x86-64 architectures
//! to allow the rest of the kernel to be architecture-independent.
//! 
//! Phase 1: Basic architecture detection and initialization
//! Phase 2: Full multi-architecture support with proper abstraction

use crate::BootInfo;

/// Architecture detection and initialization
/// 
/// This function determines the current architecture and calls the
/// appropriate initialization routine.
pub fn init(boot_info: &BootInfo) {
    #[cfg(feature = "arm64")]
    {
        println!("Initializing ARM64 architecture support");
        crate::arm64::init(&boot_info.arch_specific);
    }
    
    #[cfg(feature = "x86_64")]
    {
        println!("Initializing x86-64 architecture support");
        crate::x86_64::init(&boot_info.arch_specific);
    }
    
    // Common architecture-independent initialization
    init_common();
}

/// Common initialization for all architectures
fn init_common() {
    println!("Setting up common kernel services");
    
    // Initialize interrupt handling
    interrupts::init();
    
    // Initialize timer
    timer::init();
    
    // Initialize early console
    console::init();
}

/// Architecture-specific operations
pub mod interrupts {
    /// Initialize interrupt handling for the current architecture
    pub fn init() {
        #[cfg(feature = "arm64")]
        crate::arm64::interrupts::init();
        
        #[cfg(feature = "x86_64")]
        crate::x86_64::interrupts::init();
    }
    
    /// Enable interrupts
    pub fn enable() {
        #[cfg(feature = "arm64")]
        crate::arm64::interrupts::enable();
        
        #[cfg(feature = "x86_64")]
        crate::x86_64::interrupts::enable();
    }
    
    /// Disable interrupts
    pub fn disable() {
        #[cfg(feature = "arm64")]
        crate::arm64::interrupts::disable();
        
        #[cfg(feature = "x86_64")]
        crate::x86_64::interrupts::disable();
    }
}

/// Timer operations
pub mod timer {
    /// Initialize system timer
    pub fn init() {
        #[cfg(feature = "arm64")]
        crate::arm64::timer::init();
        
        #[cfg(feature = "x86_64")]
        crate::x86_64::timer::init();
    }
    
    /// Get current time in nanoseconds
    pub fn current_time() -> u64 {
        #[cfg(feature = "arm64")]
        return crate::arm64::timer::current_time();
        
        #[cfg(feature = "x86_64")]
        return crate::x86_64::timer::current_time();
        
        #[allow(unreachable_code)]
        0
    }
}

/// Console operations
pub mod console {
    /// Initialize early console output
    pub fn init() {
        #[cfg(feature = "arm64")]
        crate::arm64::console::init();
        
        #[cfg(feature = "x86_64")]
        crate::x86_64::console::init();
    }
    
    /// Write a string to the console
    pub fn write_str(s: &str) {
        #[cfg(feature = "arm64")]
        crate::arm64::console::write_str(s);
        
        #[cfg(feature = "x86_64")]
        crate::x86_64::console::write_str(s);
    }
    
    /// Write a byte to the console
    pub fn write_byte(b: u8) {
        #[cfg(feature = "arm64")]
        crate::arm64::console::write_byte(b);
        
        #[cfg(feature = "x86_64")]
        crate::x86_64::console::write_byte(b);
    }
}

/// Memory management operations
pub mod memory {
    /// Initialize memory management
    pub fn init() {
        #[cfg(feature = "arm64")]
        crate::arm64::memory::init();
        
        #[cfg(feature = "x86_64")]
        crate::x86_64::memory::init();
    }
    
    /// Enable virtual memory
    pub fn enable_vm() {
        #[cfg(feature = "arm64")]
        crate::arm64::memory::enable_vm();
        
        #[cfg(feature = "x86_64")]
        crate::x86_64::memory::enable_vm();
    }
    
    /// Get physical address from virtual address
    pub fn virt_to_phys(virt: usize) -> Option<usize> {
        #[cfg(feature = "arm64")]
        return crate::arm64::memory::virt_to_phys(virt);
        
        #[cfg(feature = "x86_64")]
        return crate::x86_64::memory::virt_to_phys(virt);
        
        #[allow(unreachable_code)]
        None
    }
}

/// CPU operations
pub mod cpu {
    /// Get current CPU ID
    pub fn current_cpu() -> u32 {
        #[cfg(feature = "arm64")]
        return crate::arm64::cpu::current_cpu();
        
        #[cfg(feature = "x86_64")]
        return crate::x86_64::cpu::current_cpu();
        
        #[allow(unreachable_code)]
        0
    }
    
    /// Get number of CPUs in the system
    pub fn cpu_count() -> u32 {
        #[cfg(feature = "arm64")]
        return crate::arm64::cpu::cpu_count();
        
        #[cfg(feature = "x86_64")]
        return crate::x86_64::cpu::cpu_count();
        
        #[allow(unreachable_code)]
        1
    }
    
    /// Halt the current CPU
    pub fn halt() {
        #[cfg(feature = "arm64")]
        crate::arm64::cpu::halt();
        
        #[cfg(feature = "x86_64")]
        crate::x86_64::cpu::halt();
    }
}

/// Serial port implementations for console output
pub mod serial {
    use core::fmt;
    
    /// ARM64 serial port implementation
    #[cfg(target_arch = "aarch64")]
    pub struct SerialPort;
    
    #[cfg(target_arch = "aarch64")]
    impl fmt::Write for SerialPort {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            // Phase 1: Simple serial output to PL011 UART
            // Phase 2: Use proper serial driver
            for byte in s.bytes() {
                // Write to PL011 UART at 0x9000000
                let uart = 0x9000000 as *mut u8;
                unsafe {
                    // Wait for UART to be ready
                    while *uart.add(5) & 0x20 == 0 {}
                    *uart = byte;
                }
            }
            Ok(())
        }
    }
    
    /// x86_64 serial port implementation
    #[cfg(target_arch = "x86_64")]
    pub struct SerialPort;
    
    #[cfg(target_arch = "x86_64")]
    impl fmt::Write for SerialPort {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            // Phase 1: Simple serial output to COM1
            // Phase 2: Use proper serial driver
            for byte in s.bytes() {
                // Write to COM1 at 0x3F8
                let com1 = 0x3F8 as *mut u8;
                unsafe {
                    // Wait for UART to be ready
                    while *com1.add(5) & 0x20 == 0 {}
                    *com1 = byte;
                }
            }
            Ok(())
        }
    }
}

/// Architecture detection utilities
pub mod detection {
    /// Check if running on ARM64
    pub fn is_arm64() -> bool {
        cfg!(target_arch = "aarch64")
    }
    
    /// Check if running on x86-64
    pub fn is_x86_64() -> bool {
        cfg!(target_arch = "x86_64")
    }
    
    /// Get architecture name as string
    pub fn arch_name() -> &'static str {
        if is_arm64() {
            "ARM64"
        } else if is_x86_64() {
            "x86-64"
        } else {
            "Unknown"
        }
    }
}

// Re-export serial ports for easy access
#[cfg(target_arch = "aarch64")]
pub use serial::SerialPort as arm64;

#[cfg(target_arch = "x86_64")]
pub use serial::SerialPort as x86_64;
