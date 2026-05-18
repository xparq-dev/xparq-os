// XPARQ OS - Phase 01: OS & Kernel Foundations
// x86-64 CPU management
// Provides CPU initialization, halt, and power management

#![no_std]

/// CPU information
#[derive(Debug, Clone, Copy)]
pub struct CpuInfo {
    pub cpu_id: u64,
    pub vendor_id: [u8; 12],
    pub family: u8,
    pub model: u8,
    pub stepping: u8,
    pub feature_flags: u64,
}

/// CPU features
#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    pub fpu: bool,        // x87 FPU
    pub mmx: bool,        // MMX instructions
    pub sse: bool,        // SSE instructions
    pub sse2: bool,       // SSE2 instructions
    pub sse3: bool,       // SSE3 instructions
    pub ssse3: bool,      // Supplemental SSE3
    pub sse4_1: bool,     // SSE4.1
    pub sse4_2: bool,     // SSE4.2
    pub avx: bool,        // AVX instructions
    pub avx2: bool,       // AVX2 instructions
    pub fma: bool,        // FMA instructions
    pub aes: bool,        // AES instructions
    pub pclmul: bool,     // PCLMULQDQ
    pub rdrand: bool,     // RDRAND instruction
    pub rdseed: bool,     // RDSEED instruction
    pub rdtscp: bool,     // RDTSCP instruction
    pub invpcid: bool,    // INVPCID instruction
}

/// Per-CPU data
#[derive(Debug)]
pub struct PerCpuData {
    pub cpu_id: u64,
    pub current_thread: Option<u64>,
    pub interrupt_count: u64,
    pub context_switches: u64,
}

/// Global CPU state
static mut CPU_INFO: Option<CpuInfo> = None;
static mut CPU_FEATURES: Option<CpuFeatures> = None;
static mut PER_CPU_DATA: PerCpuData = PerCpuData {
    cpu_id: 0,
    current_thread: None,
    interrupt_count: 0,
    context_switches: 0,
};

/// Early CPU initialization
pub fn early_init() {
    println!("x86-64 early CPU initialization");
    
    // Detect CPU information
    detect_cpu_info();
    
    // Detect CPU features
    detect_cpu_features();
    
    // Set up per-CPU data
    setup_per_cpu_data();
    
    println!("x86-64 early CPU initialization complete");
}

/// Detect CPU information
fn detect_cpu_info() {
    println!("Detecting CPU information...");
    
    unsafe {
        let mut cpu_info = CpuInfo {
            cpu_id: 0,
            vendor_id: [0; 12],
            family: 0,
            model: 0,
            stepping: 0,
            feature_flags: 0,
        };
        
        // Get vendor ID
        let (ebx, ecx, edx) = super::boot::regs::cpuid(0, 0);
        core::ptr::copy_nonoverlapping(
            &ebx as *const u32 as *const u8,
            cpu_info.vendor_id.as_mut_ptr(),
            4,
        );
        core::ptr::copy_nonoverlapping(
            &edx as *const u32 as *const u8,
            cpu_info.vendor_id.as_mut_ptr().add(4),
            4,
        );
        core::ptr::copy_nonoverlapping(
            &ecx as *const u32 as *const u8,
            cpu_info.vendor_id.as_mut_ptr().add(8),
            4,
        );
        
        // Get CPU signature and feature flags
        let (eax, ebx, ecx, edx) = super::boot::regs::cpuid(1, 0);
        
        cpu_info.stepping = (eax & 0xF) as u8;
        cpu_info.model = ((eax >> 4) & 0xF) as u8;
        cpu_info.family = ((eax >> 8) & 0xF) as u8;
        
        // Handle extended family/model
        if cpu_info.family == 0xF {
            let extended_family = ((eax >> 20) & 0xFF) as u8;
            cpu_info.family += extended_family;
        }
        if cpu_info.family == 0x6 || cpu_info.family == 0xF {
            let extended_model = ((eax >> 16) & 0xF) as u8;
            cpu_info.model += (extended_model << 4);
        }
        
        cpu_info.feature_flags = ((edx as u64) << 32) | (ebx as u64);
        
        // Get CPU ID (APIC ID)
        let (eax, _, _, _) = super::boot::regs::cpuid(1, 0);
        cpu_info.cpu_id = ((eax >> 24) & 0xFF) as u64;
        
        CPU_INFO = Some(cpu_info);
        
        println!("CPU ID: {}", cpu_info.cpu_id);
        println!("Vendor: {}", core::str::from_utf8_unchecked(&cpu_info.vendor_id));
        println!("Family: 0x{:x}, Model: 0x{:x}, Stepping: 0x{:x}", cpu_info.family, cpu_info.model, cpu_info.stepping);
    }
}

/// Detect CPU features
fn detect_cpu_features() {
    println!("Detecting CPU features...");
    
    unsafe {
        let mut features = CpuFeatures {
            fpu: false,
            mmx: false,
            sse: false,
            sse2: false,
            sse3: false,
            ssse3: false,
            sse4_1: false,
            sse4_2: false,
            avx: false,
            avx2: false,
            fma: false,
            aes: false,
            pclmul: false,
            rdrand: false,
            rdseed: false,
            rdtscp: false,
            invpcid: false,
        };
        
        // Basic feature flags (CPUID 1)
        let (_, _, ecx, edx) = super::boot::regs::cpuid(1, 0);
        
        features.fpu = (edx & (1 << 0)) != 0;
        features.mmx = (edx & (1 << 23)) != 0;
        features.sse = (edx & (1 << 25)) != 0;
        features.sse2 = (edx & (1 << 26)) != 0;
        features.sse3 = (ecx & (1 << 0)) != 0;
        features.pclmul = (ecx & (1 << 1)) != 0;
        features.ssse3 = (ecx & (1 << 9)) != 0;
        features.fma = (ecx & (1 << 12)) != 0;
        features.sse4_1 = (ecx & (1 << 19)) != 0;
        features.sse4_2 = (ecx & (1 << 20)) != 0;
        features.aes = (ecx & (1 << 25)) != 0;
        features.rdrand = (ecx & (1 << 30)) != 0;
        
        // Extended features (CPUID 7)
        let (_, ebx, ecx, edx) = super::boot::regs::cpuid(7, 0);
        
        features.avx2 = (ebx & (1 << 5)) != 0;
        features.rdseed = (ebx & (1 << 18)) != 0;
        features.invpcid = (ebx & (1 << 10)) != 0;
        
        // Check for AVX support
        let (_, _, ecx, _) = super::boot::regs::cpuid(1, 0);
        features.avx = (ecx & (1 << 28)) != 0;
        
        // Check for RDTSCP support
        let (_, _, ecx, _) = super::boot::regs::cpuid(0x80000001, 0);
        features.rdtscp = (ecx & (1 << 27)) != 0;
        
        CPU_FEATURES = Some(features);
        
        println!("FPU: {}", features.fpu);
        println!("MMX: {}", features.mmx);
        println!("SSE: {}", features.sse);
        println!("SSE2: {}", features.sse2);
        println!("SSE3: {}", features.sse3);
        println!("SSSE3: {}", features.ssse3);
        println!("SSE4.1: {}", features.sse4_1);
        println!("SSE4.2: {}", features.sse4_2);
        println!("AVX: {}", features.avx);
        println!("AVX2: {}", features.avx2);
        println!("FMA: {}", features.fma);
        println!("AES: {}", features.aes);
        println!("PCLMUL: {}", features.pclmul);
        println!("RDRAND: {}", features.rdrand);
        println!("RDSEED: {}", features.rdseed);
        println!("RDTSCP: {}", features.rdtscp);
        println!("INVPCID: {}", features.invpcid);
    }
}

/// Set up per-CPU data
fn setup_per_cpu_data() {
    println!("Setting up per-CPU data...");
    
    unsafe {
        PER_CPU_DATA.cpu_id = get_cpu_id();
        PER_CPU_DATA.current_thread = None;
        PER_CPU_DATA.interrupt_count = 0;
        PER_CPU_DATA.context_switches = 0;
    }
    
    println!("Per-CPU data set up");
}

/// Get CPU ID (APIC ID)
pub fn get_cpu_id() -> u64 {
    unsafe {
        let (eax, _, _, _) = super::boot::regs::cpuid(1, 0);
        ((eax >> 24) & 0xFF) as u64
    }
}

/// Get CPU information
pub fn get_cpu_info() -> Option<CpuInfo> {
    unsafe { CPU_INFO }
}

/// Get CPU features
pub fn get_cpu_features() -> Option<CpuFeatures> {
    unsafe { CPU_FEATURES }
}

/// Get per-CPU data
pub fn get_per_cpu_data() -> &'static PerCpuData {
    unsafe { &PER_CPU_DATA }
}

/// Update per-CPU data
pub fn update_per_cpu_data<F>(updater: F)
where
    F: FnOnce(&mut PerCpuData),
{
    unsafe {
        updater(&mut PER_CPU_DATA);
    }
}

/// Halt the CPU
pub fn halt() -> ! {
    println!("Halting CPU...");
    
    // Disable interrupts
    super::interrupts::disable();
    
    // Enter low power state
    loop {
        // Halt instruction
        core::arch::asm!("hlt");
        
        // Check if we should continue (Phase 2: power management)
        // For Phase 1, just halt permanently
        break;
    }
    
    // If we somehow exit the loop, halt permanently
    loop {
        core::arch::asm!("hlt");
    }
}

/// Power management utilities
pub mod power {
    use super::*;
    
    /// Enter sleep state
    pub fn enter_sleep_state() {
        println!("Entering sleep state...");
        
        // Phase 1: Basic sleep
        // Phase 2: Deep sleep with context save
        
        unsafe {
            // Halt instruction
            core::arch::asm!("hlt");
        }
        
        println!("Woke from sleep state");
    }
    
    /// CPU frequency scaling
    pub fn set_cpu_frequency(freq: u32) {
        println!("Setting CPU frequency to {} MHz", freq);
        
        // Phase 1: Dummy implementation
        // Phase 2: Real frequency scaling via MSRs
    }
    
    /// CPU power states
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum CpuPowerState {
        C0,    // Running
        C1,    // Halt
        C2,    // Sleep
        C3,    // Deep sleep
        C6,    // Power down
    }
    
    /// Get current CPU power state
    pub fn get_power_state() -> CpuPowerState {
        // Phase 1: Always running
        // Phase 2: Real power state detection
        
        CpuPowerState::C0
    }
    
    /// Set CPU power state
    pub fn set_power_state(state: CpuPowerState) {
        println!("Setting CPU power state to {:?}", state);
        
        match state {
            CpuPowerState::C0 => {
                // Wake up CPU
                println!("CPU running");
            }
            CpuPowerState::C1 => {
                // Enter halt state
                println!("CPU halt");
                enter_sleep_state();
            }
            CpuPowerState::C2 => {
                // Enter sleep state
                println!("CPU sleep");
                enter_sleep_state();
            }
            CpuPowerState::C3 => {
                // Enter deep sleep
                println!("CPU deep sleep");
                halt();
            }
            CpuPowerState::C6 => {
                // Power down CPU
                println!("CPU power down");
                halt();
            }
        }
    }
}

/// Cache management
pub mod cache {
    /// Invalidate instruction cache
    pub fn invalidate_icache() {
        unsafe {
            // x86-64 doesn't have explicit instruction cache invalidate
            // Use serializing instruction
            core::arch::asm!("mfence");
        }
    }
    
    /// Clean data cache
    pub fn clean_dcache() {
        unsafe {
            // Use CLFLUSH for specific cache lines
            // Phase 1: Use WBINVD for entire cache
            core::arch::asm!("wbinvd");
        }
    }
    
    /// Clean and invalidate data cache
    pub fn clean_invalidate_dcache() {
        unsafe {
            core::arch::asm!("wbinvd");
        }
    }
    
    /// Invalidate data cache
    pub fn invalidate_dcache() {
        unsafe {
            // x86-64 doesn't have explicit data cache invalidate
            // Use WBINVD
            core::arch::asm!("wbinvd");
        }
    }
    
    /// Memory barrier
    pub fn memory_barrier() {
        unsafe {
            core::arch::asm!("mfence");
        }
    }
    
    /// Invalidate specific cache line
    pub fn invalidate_cache_line(addr: usize) {
        unsafe {
            core::arch::asm!("clflush [{}]", in(reg) addr);
        }
    }
}

/// TLB management
pub mod tlb {
    /// Invalidate entire TLB
    pub fn invalidate_tlb() {
        unsafe {
            // Reload CR3 to invalidate TLB
            let cr3 = super::boot::regs::read_cr3();
            super::boot::regs::write_cr3(cr3);
        }
    }
    
    /// Invalidate TLB entry by address
    pub fn invalidate_tlb_entry(addr: usize) {
        unsafe {
            // Use INVLPG instruction
            core::arch::asm!("invlpg [{}]", in(reg) addr);
        }
    }
    
    /// Invalidate TLB entry by PCID
    pub fn invalidate_tlb_entry_pcid(addr: usize, pcid: u16) {
        unsafe {
            if let Some(features) = super::get_cpu_features() {
                if features.invpcid {
                    // Use INVPCID instruction
                    let mut desc = InvPcidDescriptor {
                        pcid: pcid,
                        address: addr as u64,
                    };
                    core::arch::asm!("invpcid {}, {}", in(reg) &mut desc, in(reg) 0u8);
                } else {
                    // Fallback to INVLPG
                    invalidate_tlb_entry(addr);
                }
            } else {
                invalidate_tlb_entry(addr);
            }
        }
    }
    
    /// INVPCID descriptor
    #[repr(C)]
    struct InvPcidDescriptor {
        pcid: u16,
        _padding: u16,
        address: u64,
    }
}

/// Performance monitoring
pub mod performance {
    use super::*;
    
    /// Performance counter types
    #[derive(Debug, Clone, Copy)]
    pub enum PerformanceCounter {
        InstructionsRetired,
        Cycles,
        CacheReferences,
        CacheMisses,
        BranchInstructions,
        BranchMisses,
        TscCycles,
    }
    
    /// Enable performance counter
    pub fn enable_counter(counter: PerformanceCounter) {
        println!("Enabling performance counter: {:?}", counter);
        
        // Phase 1: Dummy implementation
        // Phase 2: Real PMU configuration via MSRs
        
        unsafe {
            // Enable performance monitoring
            let mut ia32_perf_global_ctrl: u64;
            ia32_perf_global_ctrl = super::boot::regs::rdmsr(0x38F);
            ia32_perf_global_ctrl |= (1 << 32) | (1 << 33); // Enable fixed counters
            super::boot::regs::wrmsr(0x38F, ia32_perf_global_ctrl);
        }
    }
    
    /// Read performance counter
    pub fn read_counter(counter: PerformanceCounter) -> u64 {
        // Phase 1: Return dummy value
        // Phase 2: Read actual PMU counter
        
        match counter {
            PerformanceCounter::InstructionsRetired => {
                unsafe { super::boot::regs::rdmsr(0xC1) } // IA32_PMC0
            }
            PerformanceCounter::Cycles => {
                unsafe { super::boot::regs::rdmsr(0xC2) } // IA32_PMC1
            }
            PerformanceCounter::TscCycles => {
                super::boot::regs::rdtsc()
            }
            _ => {
                1000000 // Dummy value
            }
        }
    }
    
    /// Reset performance counter
    pub fn reset_counter(counter: PerformanceCounter) {
        println!("Resetting performance counter: {:?}", counter);
        
        // Phase 1: Dummy implementation
        // Phase 2: Real PMU counter reset
        
        unsafe {
            match counter {
                PerformanceCounter::InstructionsRetired => {
                    super::boot::regs::wrmsr(0xC1, 0); // IA32_PMC0
                }
                PerformanceCounter::Cycles => {
                    super::boot::regs::wrmsr(0xC2, 0); // IA32_PMC1
                }
                _ => {
                    // Dummy implementation
                }
            }
        }
    }
    
    /// Get TSC frequency
    pub fn get_tsc_frequency() -> u64 {
        // Phase 1: Return dummy frequency
        // Phase 2: Read actual TSC frequency from CPUID or MSR
        
        2000000000 // 2GHz
    }
    
    /// Calibrate TSC
    pub fn calibrate_tsc() {
        println!("Calibrating TSC...");
        
        // Phase 1: Dummy implementation
        // Phase 2: Real TSC calibration using PIT or HPET
        
        println!("TSC calibrated");
    }
}
