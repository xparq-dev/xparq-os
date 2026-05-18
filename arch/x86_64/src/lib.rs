//! XPARQ OS x86-64 Architecture Support - Phase 2: Dev Environment Setup
//! 
//! This module provides x86-64-specific support for XPARQ OS, including:
//! - UEFI bootloader entry point and initialization
//! - x86-64 privilege levels (Ring 0-3) management
//! - Interrupt handling and APIC configuration
//! - Memory management with PML4 page tables
//! - ACPI integration for power management
//! - PCIe device enumeration (Phase 3)
//! 
//! Architecture: x86-64 (AMD64)
//! Privilege Levels: Ring 0 (Kernel), Ring 3 (Userspace)
//! Boot Method: UEFI with Secure Boot support
//! Page Tables: 4-level PML4 with 4KB pages
//! 
//! Roadmap Phase: Phase 2 - Dev Environment Setup
//! Previous Phase: Phase 1 - OS Foundations
//! Next Phase: Phase 3 - Hardware Abstraction Layer

#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(asm_experimental_arch)]

// Core x86-64 modules
mod boot;
mod memory;
mod interrupts;
mod timer;
mod console;
mod context;
mod cpu;
mod acpi;
mod uefi;

// Re-export architecture-specific functions
pub use boot::{x86_64_entry, boot_main};
pub use memory::{init as memory_init, enable_vm as memory_enable_vm};
pub use interrupts::{init as interrupts_init, enable as interrupts_enable, disable as interrupts_disable};
pub use timer::{init as timer_init, current_time as timer_current_time};
pub use console::{init as console_init, write_str as console_write_str, write_byte as console_write_byte};
pub use context::{switch as context_switch};
pub use cpu::{current_cpu as cpu_current_cpu, cpu_count as cpu_cpu_count, halt as cpu_halt};

/// x86-64 initialization entry point
/// 
/// This is called from the UEFI bootloader after basic hardware setup.
/// It initializes x86-64-specific components before jumping to the
/// main kernel.
pub fn init(boot_info: &crate::ArchBootInfo) {
    println!("Initializing x86-64 architecture support...");
    
    // Initialize x86-64 memory management
    memory::init();
    
    // Initialize interrupt controller (APIC)
    interrupts::init();
    
    // Initialize system timer
    timer::init();
    
    // Initialize early console
    console::init();
    
    // Initialize CPU-specific features
    cpu::init();
    
    // Initialize ACPI for hardware discovery
    acpi::init(boot_info.rsdp);
    
    println!("x86-64 architecture initialization complete");
}

/// x86-64 panic handler
/// 
/// Called when the kernel encounters an unrecoverable error.
pub fn panic_halt() -> ! {
    println!("x86-64 KERNEL PANIC - Halting system");
    
    // Disable interrupts
    interrupts::disable();
    
    // Halt all CPUs
    cpu::halt_all();
    
    // Infinite loop
    loop {
        core::hint::spin_loop();
    }
}

/// x86-64-specific boot information processing
pub fn process_boot_info(boot_info: &crate::ArchBootInfo) -> BootInfo {
    BootInfo {
        rsdp_ptr: boot_info.rsdp,
        bootloader_brand: boot_info.bootloader_brand,
        memory_regions: parse_uefi_memory_map(boot_info),
        framebuffer: find_framebuffer(boot_info),
    }
}

/// x86-64 boot information structure
#[derive(Debug)]
pub struct BootInfo {
    pub rsdp_ptr: usize,
    pub bootloader_brand: &'static str,
    pub memory_regions: &'static [MemoryRegion],
    pub framebuffer: Option<FramebufferInfo>,
}

/// Memory region for x86-64
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
    Mmio,
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

/// Parse UEFI memory map
fn parse_uefi_memory_map(boot_info: &crate::ArchBootInfo) -> &'static [MemoryRegion] {
    // Phase 1: Return static memory regions
    // Phase 2: Parse actual UEFI memory map
    
    static REGIONS: [MemoryRegion; 6] = [
        MemoryRegion { base: 0x00000000, size: 0x0009FC00, kind: MemoryRegionKind::Usable },      // 640KB conventional memory
        MemoryRegion { base: 0x0009FC00, size: 0x00000400, kind: MemoryRegionKind::Reserved },     // EBDA
        MemoryRegion { base: 0x000A0000, size: 0x00020000, kind: MemoryRegionKind::Reserved },     // VGA BIOS
        MemoryRegion { base: 0x000F0000, size: 0x00010000, kind: MemoryRegionKind::Reserved },     // System BIOS
        MemoryRegion { base: 0x00100000, size: 0x7FF00000, kind: MemoryRegionKind::Usable },      // 2GB RAM
        MemoryRegion { base: 0xFEC00000, size: 0x00100000, kind: MemoryRegionKind::Mmio },        // APIC
    ];
    
    &REGIONS
}

/// Find framebuffer information
fn find_framebuffer(boot_info: &crate::ArchBootInfo) -> Option<FramebufferInfo> {
    // Phase 1: Return None (will be set up in Phase 3)
    // Phase 2: Parse from UEFI Graphics Output Protocol
    None
}

/// x86-64 CPU feature detection
pub mod features {
    use x86_64::registers::model_specific::Msr;
    
    /// Check if CPU supports specific features
    pub fn has_feature(feature: CpuFeature) -> bool {
        match feature {
            CpuFeature::Apic => x86_64::registers::control::Cr4::read().contains(x86_64::registers::control::Cr4Flags::OSFXSR),
            CpuFeature::X2Apic => x86_64::registers::control::Cr4::read().contains(x86_64::registers::control::Cr4Flags::OSXSAVE),
            CpuFeature::Virt => {
                let mut msr = Msr::new(0x3A); // IA32_FEATURE_CONTROL
                msr.read().unwrap_or(0) & 0x1 != 0
            }
            CpuFeature::Tsc => true, // All x86-64 CPUs have TSC
            CpuFeature::Sse => true, // All x86-64 CPUs have SSE
            CpuFeature::Avx => {
                let cpuid = x86_64::cpuid::CpuId::new();
                let result = cpuid.get_extended_processor_and_feature_identifiers().unwrap();
                result.has_avx()
            }
        }
    }
    
    /// CPU features
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum CpuFeature {
        Apic,      // Local APIC support
        X2Apic,    // x2APIC support
        Virt,      // Virtualization support
        Tsc,       // Time Stamp Counter
        Sse,       // Streaming SIMD Extensions
        Avx,       // Advanced Vector Extensions
    }
}

/// x86-64 system registers access
pub mod sysreg {
    use x86_64::registers::control::{Cr0, Cr3, Cr4};
    use x86_64::registers::model_specific::Msr;
    
    /// Read CR0 register
    #[inline(always)]
    pub fn read_cr0() -> x86_64::registers::control::Cr0Flags {
        Cr0::read()
    }
    
    /// Write CR0 register
    #[inline(always)]
    pub fn write_cr0(flags: x86_64::registers::control::Cr0Flags) {
        Cr0::write(flags);
    }
    
    /// Read CR3 register (page table base)
    #[inline(always)]
    pub fn read_cr3() -> x86_64::PhysAddr {
        Cr3::read()
    }
    
    /// Write CR3 register
    #[inline(always)]
    pub fn write_cr3(addr: x86_64::PhysAddr) {
        unsafe { Cr3::write(addr) };
    }
    
    /// Read CR4 register
    #[inline(always)]
    pub fn read_cr4() -> x86_64::registers::control::Cr4Flags {
        Cr4::read()
    }
    
    /// Write CR4 register
    #[inline(always)]
    pub fn write_cr4(flags: x86_64::registers::control::Cr4Flags) {
        Cr4::write(flags);
    }
    
    /// Read model-specific register
    #[inline(always)]
    pub fn read_msr(msr: u32) -> Result<u64, ()> {
        Msr::new(msr).read()
    }
    
    /// Write model-specific register
    #[inline(always)]
    pub fn write_msr(msr: u32, value: u64) -> Result<(), ()> {
        Msr::new(msr).write(value)
    }
    
    /// Enable/disable paging
    pub fn set_paging(enabled: bool) {
        let mut cr0 = Cr0::read();
        if enabled {
            cr0 |= x86_64::registers::control::Cr0Flags::PAGING;
        } else {
            cr0 &= !x86_64::registers::control::Cr0Flags::PAGING;
        }
        Cr0::write(cr0);
    }
    
    /// Invalidate TLB
    pub fn invalidate_tlb() {
        unsafe {
            x86_64::instructions::tlb::flush_all();
        }
    }
}

/// x86-64 assembly utilities
pub mod asm_utils {
    /// Memory barrier
    #[inline(always)]
    pub fn mfence() {
        unsafe {
            x86_64::instructions::mfence();
        }
    }
    
    /// Serializing instruction
    #[inline(always)]
    pub fn serialize() {
        unsafe {
            x86_64::instructions::serializing::serialize();
        }
    }
    
    /// Halt CPU
    #[inline(always)]
    pub fn hlt() {
        unsafe {
            x86_64::instructions::hlt();
        }
    }
    
    /// Pause instruction (for spin loops)
    #[inline(always)]
    pub fn pause() {
        unsafe {
            x86_64::instructions::pause();
        }
    }
    
    /// CPUID instruction
    #[inline(always)]
    pub fn cpuid(eax: u32) -> x86_64::cpuid::CpuIdResult {
        x86_64::cpuid::CpuId::new().cpuid(eax)
    }
    
    /// Read Time Stamp Counter
    #[inline(always)]
    pub fn rdtsc() -> u64 {
        unsafe {
            x86_64::registers::rdtsc()
        }
    }
}

/// x86-64 exception handling
pub mod exception {
    use x86_64::registers::control::Cr2;
    
    /// Exception vector table
    #[repr(align(16))]
    pub static mut IDT: x86_64::structures::idt::InterruptDescriptorTable = x86_64::structures::idt::InterruptDescriptorTable::new();
    
    /// Initialize exception vectors
    pub fn init_idt() {
        // Phase 1: Basic IDT setup
        // Phase 2: Full exception handling with proper context save/restore
        
        unsafe {
            IDT.divide_error.set_handler_fn(divide_error_handler);
            IDT.debug.set_handler_fn(debug_handler);
            IDT.non_maskable_interrupt.set_handler_fn(nmi_handler);
            IDT.breakpoint.set_handler_fn(breakpoint_handler);
            IDT.overflow.set_handler_fn(overflow_handler);
            IDT.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
            IDT.invalid_opcode.set_handler_fn(invalid_opcode_handler);
            IDT.device_not_available.set_handler_fn(device_not_available_handler);
            IDT.double_fault.set_handler_fn(double_fault_handler);
            IDT.invalid_tss.set_handler_fn(invalid_tss_handler);
            IDT.segment_not_present.set_handler_fn(segment_not_present_handler);
            IDT.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
            IDT.general_protection_fault.set_handler_fn(general_protection_fault_handler);
            IDT.page_fault.set_handler_fn(page_fault_handler);
            IDT.x87_floating_point.set_handler_fn(x87_floating_point_handler);
            IDT.alignment_check.set_handler_fn(alignment_check_handler);
            IDT.machine_check.set_handler_fn(machine_check_handler);
            IDD.simd_floating_point.set_handler_fn(simd_floating_point_handler);
            IDT.virtualization.set_handler_fn(virtualization_handler);
            IDT.security.set_handler_fn(security_handler);
            
            IDT.load();
        }
        
        println!("IDT initialized");
    }
    
    /// Enable interrupts
    pub fn enable_interrupts() {
        unsafe {
            x86_64::instructions::interrupts::enable();
        }
    }
    
    /// Disable interrupts
    pub fn disable_interrupts() {
        unsafe {
            x86_64::interrupts::disable();
        }
    }
    
    // Exception handlers
    extern "x86-interrupt" fn divide_error_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Divide error exception");
        panic!("Divide by zero");
    }
    
    extern "x86-interrupt" fn debug_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Debug exception");
    }
    
    extern "x86-interrupt" fn nmi_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Non-maskable interrupt");
    }
    
    extern "x86-interrupt" fn breakpoint_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Breakpoint exception");
    }
    
    extern "x86-interrupt" fn overflow_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Overflow exception");
        panic!("Overflow");
    }
    
    extern "x86-interrupt" fn bound_range_exceeded_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Bound range exceeded exception");
        panic!("Bound range exceeded");
    }
    
    extern "x86-interrupt" fn invalid_opcode_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Invalid opcode exception");
        panic!("Invalid opcode");
    }
    
    extern "x86-interrupt" fn device_not_available_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Device not available exception");
        panic!("Device not available");
    }
    
    extern "x86-interrupt" fn double_fault_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) -> ! {
        println!("Double fault exception");
        panic!("Double fault");
    }
    
    extern "x86-interrupt" fn invalid_tss_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Invalid TSS exception");
        panic!("Invalid TSS");
    }
    
    extern "x86-interrupt" fn segment_not_present_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Segment not present exception");
        panic!("Segment not present");
    }
    
    extern "x86-interrupt" fn stack_segment_fault_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Stack segment fault exception");
        panic!("Stack segment fault");
    }
    
    extern "x86-interrupt" fn general_protection_fault_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame, error_code: u64) {
        println!("General protection fault: error_code={}", error_code);
        panic!("General protection fault");
    }
    
    extern "x86-interrupt" fn page_fault_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame, error_code: u64) {
        let fault_address = Cr2::read();
        println!("Page fault: fault_address=0x{:x}, error_code={}", fault_address.as_u64(), error_code);
        panic!("Page fault");
    }
    
    extern "x86-interrupt" fn x87_floating_point_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("x87 floating point exception");
        panic!("x87 floating point exception");
    }
    
    extern "x86-interrupt" fn alignment_check_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame, error_code: u64) {
        println!("Alignment check exception: error_code={}", error_code);
        panic!("Alignment check");
    }
    
    extern "x86-interrupt" fn machine_check_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) -> ! {
        println!("Machine check exception");
        panic!("Machine check");
    }
    
    extern "x86-interrupt" fn simd_floating_point_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("SIMD floating point exception");
        panic!("SIMD floating point exception");
    }
    
    extern "x86-interrupt" fn virtualization_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Virtualization exception");
        panic!("Virtualization exception");
    }
    
    extern "x86-interrupt" fn security_handler(frame: &mut x86_64::structures::idt::ExceptionStackFrame) {
        println!("Security exception");
        panic!("Security exception");
    }
}

/// x86-64 APIC (Local APIC) interface
pub mod apic {
    use x86_64::registers::model_specific::Msr;
    
    /// Local APIC base address
    pub const APIC_BASE: usize = 0xFEE00000;
    
    /// Initialize Local APIC
    pub fn init() {
        println!("Initializing Local APIC...");
        
        // Enable APIC in CR4
        let mut cr4 = super::sysreg::read_cr4();
        cr4 |= x86_64::registers::control::Cr4Flags::OSFXSR;
        cr4 |= x86_64::registers::control::Cr4Flags::OSXSAVE;
        super::sysreg::write_cr4(cr4);
        
        // Set APIC base MSR
        let apic_base_msr = Msr::new(0x1B); // IA32_APIC_BASE
        let mut apic_base = apic_base_msr.read().unwrap_or(0);
        apic_base |= (1 << 11); // Enable APIC
        apic_base_msr.write(apic_base).unwrap();
        
        // Phase 2: Configure APIC timer, interrupts, etc.
        
        println!("Local APIC initialized");
    }
    
    /// Enable specific interrupt
    pub fn enable_irq(interrupt: u8) {
        // Phase 2: Implement APIC interrupt enable
        println!("Enabled APIC interrupt {}", interrupt);
    }
    
    /// Set interrupt priority
    pub fn set_priority(interrupt: u8, priority: u8) {
        // Phase 2: Implement APIC priority setting
        println!("Set APIC interrupt {} priority to {}", interrupt, priority);
    }
}
