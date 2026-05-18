// XPARQ OS - Phase 01: OS & Kernel Foundations
// ARM64 bootloader entry point
// Handles ARM64-specific boot sequence and initialization

#![no_std]

/// ARM64 entry point called from bootloader
/// 
/// This function is called with:
/// x0 = Device tree blob pointer (if available)
/// x1 = Reserved (future use)
/// x2 = Reserved (future use)
/// x3 = Reserved (future use)
#[no_mangle]
#[naked]
pub extern "C" fn arm64_entry() -> ! {
    // Save boot arguments
    // Set up stack
    // Call kernel_main
    unsafe {
        core::arch::asm!(
            "
            // Save boot arguments
            mov x20, x0  // Device tree pointer
            
            // Set up stack (using temporary stack in low memory)
            ldr x3, =0x40000  // Temporary stack at 256KB
            mov sp, x3
            
            // Call rust entry point
            bl rust_arm64_entry
            
            // Should never reach here
            b .
            ",
            options(noreturn)
        );
    }
}

/// Rust entry point after basic setup
#[no_mangle]
extern "C" fn rust_arm64_entry(dt_ptr: usize) -> ! {
    // Initialize early debugging
    crate::arch::arm64::uart::init();
    
    println!("XPARQ OS Booting on AArch64...");
    println!("Device tree at 0x{:x}", dt_ptr);
    
    // Parse device tree (Phase 2)
    let boot_info = parse_device_tree(dt_ptr);
    
    // Call architecture-agnostic kernel main
    crate::kernel_main(&boot_info);
}

/// Parse device tree blob
fn parse_device_tree(dt_ptr: usize) -> crate::BootInfo {
    println!("Parsing device tree...");
    
    // Phase 1: Create dummy boot info
    // Phase 2: Parse actual device tree
    
    let memory_regions = &[
        crate::MemoryRegion {
            base: 0x40000000,
            size: 512 * 1024 * 1024, // 512MB
            kind: crate::MemoryRegionKind::Usable,
        },
        crate::MemoryRegion {
            base: 0x9000000,
            size: 0x1000, // UART
            kind: crate::MemoryRegionKind::Mmio,
        },
    ];
    
    let framebuffer = Some(crate::FramebufferInfo {
        address: 0x44000000,
        width: 1024,
        height: 768,
        stride: 1024,
        format: crate::PixelFormat::Rgb32,
    });
    
    let arch_specific = crate::ArchBootInfo {
        rsdp: 0, // ARM64 doesn't use ACPI RSDP
        bootloader_brand: "QEMU",
    };
    
    crate::BootInfo {
        memory_regions,
        framebuffer,
        arch_specific,
    }
}

/// ARM64 CPU initialization
pub fn init_cpu() {
    println!("Initializing ARM64 CPU...");
    
    // Set up exception levels
    setup_exception_levels();
    
    // Enable caches
    enable_caches();
    
    // Set up memory management unit
    crate::arch::arm64::mmu::init();
    
    println!("ARM64 CPU initialized");
}

/// Set up ARM64 exception levels
fn setup_exception_levels() {
    println!("Setting up exception levels...");
    
    // Phase 1: Stay in EL1 (kernel mode)
    // Phase 2: Set up proper EL0-EL3 transitions
    
    unsafe {
        // CurrentEL register tells us our current exception level
        let mut current_el: u64;
        core::arch::asm!("mrs {}, CurrentEL", out(reg) current_el);
        current_el = (current_el >> 2) & 0x3;
        
        println!("Current exception level: EL{}", current_el);
        
        if current_el != 1 {
            panic!("Kernel must run at EL1");
        }
    }
}

/// Enable ARM64 caches
fn enable_caches() {
    println!("Enabling caches...");
    
    // Phase 1: Basic cache enable
    // Phase 2: Full cache configuration
    
    unsafe {
        // Enable I-cache and D-cache
        let mut sctlr: u64;
        core::arch::asm!("mrs {}, SCTLR_EL1", out(reg) sctlr);
        
        // Set I and C bits (Instruction cache and Data cache)
        sctlr |= (1 << 12) | (1 << 2);
        
        // Write back SCTLR_EL1
        core::arch::asm!("msr SCTLR_EL1, {}", in(reg) sctlr);
        
        // Invalidate instruction cache
        core::arch::asm!("ic iallu");
        
        // Data synchronization barrier
        core::arch::asm!("dsb sy");
        core::arch::asm!("isb sy");
    }
    
    println!("Caches enabled");
}

/// ARM64 system register access utilities
pub mod regs {
    /// Read system register
    #[inline(always)]
    pub unsafe fn mrs(reg: &str) -> u64 {
        let mut value: u64;
        core::arch::asm!(concat!("mrs {}, ", reg), out(reg) value);
        value
    }
    
    /// Write system register
    #[inline(always)]
    pub unsafe fn msr(reg: &str, value: u64) {
        core::arch::asm!(concat!("msr ", reg, ", {}"), in(reg) value);
    }
    
    /// Get current exception level
    pub fn current_el() -> u64 {
        unsafe {
            let el: u64;
            core::arch::asm!("mrs {}, CurrentEL", out(reg) el);
            (el >> 2) & 0x3
        }
    }
    
    /// Get processor ID
    pub fn mpidr() -> u64 {
        unsafe {
            let mpidr: u64;
            core::arch::asm!("mrs {}, MPIDR_EL1", out(reg) mpidr);
            mpidr
        }
    }
    
    /// Get timer frequency
    pub fn cntfrq() -> u64 {
        unsafe {
            let freq: u64;
            core::arch::asm!("mrs {}, CNTFRQ_EL0", out(reg) freq);
            freq
        }
    }
    
    /// Read timer counter
    pub fn cntpct() -> u64 {
        unsafe {
            let cnt: u64;
            core::arch::asm!("mrs {}, CNTPCT_EL0", out(reg) cnt);
            cnt
        }
    }
}

/// ARM64 exception handling
pub mod exceptions {
    use core::sync::atomic::{AtomicU64, Ordering};
    
    static EXCEPTION_COUNT: AtomicU64 = AtomicU64::new(0);
    
    /// Exception vector table
    #[link_section = ".text.exception_vectors"]
    #[naked]
    pub static mut EXCEPTION_VECTORS: [u8; 0x800] = [0; 0x800];
    
    /// Set up exception vectors
    pub fn setup_vectors() {
        println!("Setting up exception vectors...");
        
        // Phase 1: Basic exception vectors
        // Phase 2: Full exception handling
        
        unsafe {
            // Initialize exception vectors with basic handlers
            let vectors = &mut EXCEPTION_VECTORS;
            
            // Current EL with SP0 (0x0)
            vectors[0..0x80].copy_from_slice(&[0; 0x80]);
            
            // Current EL with SPx (0x80)
            vectors[0x80..0x100].copy_from_slice(&[0; 0x80]);
            
            // Lower EL using AArch64 (0x100)
            vectors[0x100..0x180].copy_from_slice(&[0; 0x80]);
            
            // Lower EL using AArch32 (0x180)
            vectors[0x180..0x200].copy_from_slice(&[0; 0x80]);
            
            // Set VBAR_EL1 to point to our exception vectors
            let vbar = EXCEPTION_VECTORS.as_ptr() as u64;
            core::arch::asm!("msr VBAR_EL1, {}", in(reg) vbar);
            
            // Data synchronization barrier
            core::arch::asm!("dsb sy");
            core::arch::asm!("isb sy");
        }
        
        println!("Exception vectors set up");
    }
    
    /// Generic exception handler
    #[no_mangle]
    extern "C" fn exception_handler() {
        let count = EXCEPTION_COUNT.fetch_add(1, Ordering::SeqCst);
        println!("Exception #{} occurred", count);
        
        // Phase 1: Basic exception handling
        // Phase 2: Full exception processing
        
        // Get exception syndrome
        unsafe {
            let esr: u64;
            core::arch::asm!("mrs {}, ESR_EL1", out(reg) esr);
            let ec = (esr >> 26) & 0x3F; // Exception class
            let il = (esr >> 25) & 0x1;  // Instruction length
            let iss = esr & 0x1FFFFFF;  // Instruction specific syndrome
            
            println!("ESR_EL1: EC={}, IL={}, ISS=0x{:x}", ec, il, iss);
        }
        
        // For now, just return from exception
        // Phase 2: Proper exception handling
    }
    
    /// System call handler
    #[no_mangle]
    extern "C" fn syscall_handler() {
        println!("System call received");
        
        // Phase 1: Basic syscall handling
        // Phase 2: Full syscall implementation
    }
}

/// ARM64 memory barriers
pub mod barriers {
    /// Data synchronization barrier
    #[inline(always)]
    pub fn dsb() {
        unsafe {
            core::arch::asm!("dsb sy");
        }
    }
    
    /// Instruction synchronization barrier
    #[inline(always)]
    pub fn isb() {
        unsafe {
            core::arch::asm!("isb sy");
        }
    }
    
    /// Data memory barrier
    #[inline(always)]
    pub fn dmb() {
        unsafe {
            core::arch::asm!("dmb sy");
        }
    }
}
