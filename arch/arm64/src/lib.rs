//! XPARQ OS ARM64 Architecture Support - Phase 2: Dev Environment Setup
//! 
//! This module provides ARM64-specific support for XPARQ OS, including:
//! - Bootloader entry point and initialization
//! - ARM Exception Level (EL0-EL3) management
//! - Interrupt handling and GIC configuration
//! - Memory management with ARM page tables
//! - TrustZone security integration
//! 
//! Architecture: ARM64 (AArch64)
//! Exception Levels: EL0 (Userspace), EL1 (Kernel), EL2 (Hypervisor), EL3 (Secure Monitor)
//! Security: TrustZone + ARMv8.3 Pointer Authentication
//! 
//! Roadmap Phase: Phase 2 - Dev Environment Setup
//! Previous Phase: Phase 1 - OS Foundations
//! Next Phase: Phase 3 - Hardware Abstraction Layer

#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(asm_experimental_arch)]

// Core ARM64 modules
mod boot;
mod memory;
mod interrupts;
mod timer;
mod console;
mod context;
mod cpu;
mod trustzone;

// Re-export architecture-specific functions
pub use boot::{arm64_entry, boot_main};
pub use memory::{init as memory_init, enable_vm as memory_enable_vm};
pub use interrupts::{init as interrupts_init, enable as interrupts_enable, disable as interrupts_disable};
pub use timer::{init as timer_init, current_time as timer_current_time};
pub use console::{init as console_init, write_str as console_write_str, write_byte as console_write_byte};
pub use context::{switch as context_switch};
pub use cpu::{current_cpu as cpu_current_cpu, cpu_count as cpu_cpu_count, halt as cpu_halt};

/// ARM64 initialization entry point
/// 
/// This is called from the bootloader after basic hardware setup.
/// It initializes ARM64-specific components before jumping to the
/// main kernel.
pub fn init(boot_info: &crate::ArchBootInfo) {
    println!("Initializing ARM64 architecture support...");
    
    // Initialize ARM64 memory management
    memory::init();
    
    // Initialize interrupt controller (GIC)
    interrupts::init();
    
    // Initialize system timer
    timer::init();
    
    // Initialize early console
    console::init();
    
    // Initialize CPU-specific features
    cpu::init();
    
    // Initialize TrustZone security (if available)
    trustzone::init();
    
    println!("ARM64 architecture initialization complete");
}

/// ARM64 panic handler
/// 
/// Called when the kernel encounters an unrecoverable error.
pub fn panic_halt() -> ! {
    println!("ARM64 KERNEL PANIC - Halting system");
    
    // Disable interrupts
    interrupts::disable();
    
    // Halt all CPUs
    cpu::halt_all();
    
    // Infinite loop
    loop {
        core::hint::spin_loop();
    }
}

/// ARM64-specific boot information processing
pub fn process_boot_info(boot_info: &crate::ArchBootInfo) -> BootInfo {
    BootInfo {
        device_tree_ptr: boot_info.device_tree,
        cpu_count: boot_info.cpu_count,
        memory_regions: parse_device_tree_memory(boot_info.device_tree),
        uart_base: find_uart_base(boot_info.device_tree),
    }
}

/// ARM64 boot information structure
#[derive(Debug)]
pub struct BootInfo {
    pub device_tree_ptr: usize,
    pub cpu_count: u32,
    pub memory_regions: &'static [MemoryRegion],
    pub uart_base: usize,
}

/// Memory region for ARM64
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
    Device,
    AcpiReclaimable,
    AcpiNvs,
}

/// Parse memory regions from device tree
fn parse_device_tree_memory(dt_ptr: usize) -> &'static [MemoryRegion] {
    // Phase 1: Return static memory regions
    // Phase 2: Parse actual device tree
    
    static REGIONS: [MemoryRegion; 4] = [
        MemoryRegion { base: 0x40000000, size: 0x8000000, kind: MemoryRegionKind::Usable }, // 128MB RAM
        MemoryRegion { base: 0x9000000, size: 0x100000, kind: MemoryRegionKind::Device },   // UART
        MemoryRegion { base: 0x8000000, size: 0x100000, kind: MemoryRegionKind::Device },   // GIC
        MemoryRegion { base: 0x08000000, size: 0x2000000, kind: MemoryRegionKind::Reserved }, // Firmware
    ];
    
    &REGIONS
}

/// Find UART base address from device tree
fn find_uart_base(dt_ptr: usize) -> usize {
    // Phase 1: Return fixed PL011 UART address
    // Phase 2: Parse from device tree
    0x9000000
}

/// ARM64 CPU feature detection
pub mod features {
    /// Check if CPU supports specific features
    pub fn has_feature(feature: CpuFeature) -> bool {
        // Read ID_AA64PFR0_EL1 register
        let id_aa64pfr0_el1: u64;
        unsafe {
            core::arch::asm!("mrs {}, ID_AA64PFR0_EL1", out(reg) id_aa64pfr0_el1);
        }
        
        match feature {
            CpuFeature::El3 => (id_aa64pfr0_el1 & 0xF000) != 0,
            CpuFeature::El2 => (id_aa64pfr0_el1 & 0xF00) != 0,
            CpuFeature::PointerAuth => (id_aa64pfr0_el1 & 0xF) != 0,
            CpuFeature::GenericTimer => true, // ARMv8 always has generic timer
        }
    }
    
    /// CPU features
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum CpuFeature {
        El3,           // EL3 (Secure Monitor) support
        El2,           // EL2 (Hypervisor) support  
        PointerAuth,   // Pointer Authentication
        GenericTimer,  // Generic Timer
    }
}

/// ARM64 system registers access
pub mod sysreg {
    /// Read system register
    #[inline(always)]
    pub fn mrs(reg: &str) -> u64 {
        let result: u64;
        unsafe {
            core::arch::asm!(concat!("mrs {}, ", reg), out(reg) result);
        }
        result
    }
    
    /// Write system register
    #[inline(always)]
    pub fn msr(reg: &str, value: u64) {
        unsafe {
            core::arch::asm!(concat!("msr ", reg, ", {}"), in(reg) value);
        }
    }
    
    /// Get current exception level
    pub fn current_el() -> u32 {
        let current_el: u64;
        unsafe {
            core::arch::asm!("mrs {}, CurrentEL", out(reg) current_el);
        }
        ((current_el >> 2) & 0x3) as u32
    }
    
    /// Enable/disable MMU
    pub fn set_sctlr(mmu_enabled: bool) {
        let mut sctlr = mrs("SCTLR_EL1");
        if mmu_enabled {
            sctlr |= 1; // Set M bit
        } else {
            sctlr &= !1; // Clear M bit
        }
        msr("SCTLR_EL1", sctlr);
    }
    
    /// Invalidate TLB
    pub fn tlbialle1is() {
        unsafe {
            core::arch::asm!("tlbi alle1is");
        }
    }
}

/// ARM64 assembly utilities
pub mod asm_utils {
    /// Memory barrier
    #[inline(always)]
    pub fn dmb() {
        unsafe {
            core::arch::asm!("dmb ish");
        }
    }
    
    /// Data synchronization barrier
    #[inline(always)]
    pub fn dsb() {
        unsafe {
            core::arch::asm!("dsb ish");
        }
    }
    
    /// Instruction synchronization barrier
    #[inline(always)]
    pub fn isb() {
        unsafe {
            core::arch::asm!("isb");
        }
    }
    
    /// Wait for interrupt
    #[inline(always)]
    pub fn wfi() {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
    
    /// Wait for event
    #[inline(always)]
    pub fn wfe() {
        unsafe {
            core::arch::asm!("wfe");
        }
    }
}

/// ARM64 exception handling
pub mod exception {
    use super::sysreg;
    
    /// Exception vector table
    #[repr(align(2048))] // 2KB alignment required
    pub static mut EXCEPTION_VECTOR: [u8; 2048] = [0; 2048];
    
    /// Initialize exception vectors
    pub fn init_vectors() {
        // Phase 1: Basic exception vectors
        // Phase 2: Full exception handling with proper context save/restore
        
        let vector = unsafe { &mut EXCEPTION_VECTOR };
        
        // Current EL with SP0
        vector[0x0..0x80].copy_from_slice(&[0x18, 0x00, 0x00, 0x10]); // b .sync_current_el_sp0
        
        // Current EL with SPx
        vector[0x80..0x100].copy_from_slice(&[0x58, 0x00, 0x00, 0x10]); // b .sync_current_el_spx
        
        // Lower EL using AArch64
        vector[0x100..0x180].copy_from_slice(&[0x98, 0x00, 0x00, 0x10]); // b .sync_lower_el_aarch64
        
        // Lower EL using AArch32
        vector[0x180..0x200].copy_from_slice(&[0xD8, 0x00, 0x00, 0x10]); // b .sync_lower_el_aarch32
        
        // Set VBAR_EL1 to point to our vector table
        let vector_addr = vector.as_ptr() as u64;
        sysreg::msr("VBAR_EL1", vector_addr);
        
        println!("Exception vectors initialized at 0x{:x}", vector_addr);
    }
    
    /// Enable exceptions
    pub fn enable_exceptions() {
        let mut daif = sysreg::mrs("DAIF");
        daif &= !0b1111; // Clear D, A, I, F bits
        sysreg::msr("DAIF", daif);
    }
    
    /// Disable exceptions
    pub fn disable_exceptions() {
        let mut daif = sysreg::mrs("DAIF");
        daif |= 0b1111; // Set D, A, I, F bits
        sysreg::msr("DAIF", daif);
    }
}

/// ARM64 GIC (Generic Interrupt Controller) interface
pub mod gic {
    /// GICv3 distributor base address
    pub const GICD_BASE: usize = 0x8000000;
    
    /// GICv3 redistributor base address
    pub const GICR_BASE: usize = 0x8010000;
    
    /// Initialize GIC
    pub fn init() {
        println!("Initializing GICv3...");
        
        // Phase 1: Basic GIC initialization
        // Phase 2: Full interrupt routing and priority management
        
        // Enable distributor
        let gicd_ctlr = unsafe { core::ptr::read_volatile((GICD_BASE + 0x0000) as *const u32) };
        unsafe {
            core::ptr::write_volatile((GICD_BASE + 0x0000) as *mut u32, gicd_ctlr | 0x1);
        }
        
        // Enable redistributor
        let gicr_waker = unsafe { core::ptr::read_volatile((GICR_BASE + 0x0014) as *const u32) };
        unsafe {
            core::ptr::write_volatile((GICR_BASE + 0x0014) as *mut u32, gicr_waker & !0x2);
        }
        
        // Wait for redistributor to be ready
        while unsafe { core::ptr::read_volatile((GICR_BASE + 0x0014) as *const u32) } & 0x4 != 0 {
            core::hint::spin_loop();
        }
        
        println!("GICv3 initialized");
    }
    
    /// Enable specific interrupt
    pub fn enable_irq(irq: u32) {
        let reg_offset = irq / 32;
        let bit_offset = irq % 32;
        
        let gicd_isenabler = unsafe { 
            &mut *((GICD_BASE + 0x0100 + reg_offset * 4) as *mut u32) 
        };
        
        unsafe {
            gicd_isenabler.write_volatile(gicd_isenabler.read_volatile() | (1 << bit_offset));
        }
    }
    
    /// Set interrupt priority
    pub fn set_priority(irq: u32, priority: u8) {
        let gicd_ipriorityr = unsafe {
            &mut *((GICD_BASE + 0x0400 + irq as usize) as *mut u8)
        };
        
        unsafe {
            gicd_ipriorityr.write_volatile(priority);
        }
    }
}
