// XPARQ OS - Phase 01: OS & Kernel Foundations
// x86-64 bootloader entry point
// Handles x86-64 UEFI boot sequence and initialization

#![no_std]

/// x86-64 entry point called from UEFI bootloader
/// 
/// This function is called with:
/// RDI = Boot information structure pointer
/// RSI = Reserved (future use)
/// RDX = Reserved (future use)
/// RCX = Reserved (future use)
#[no_mangle]
#[naked]
pub extern "C" fn x86_64_entry() -> ! {
    unsafe {
        core::arch::asm!(
            "
            // Save boot arguments
            mov rdi, rdi  // Boot info pointer
            mov rsi, rsi  // Reserved
            mov rdx, rdx  // Reserved
            mov rcx, rcx  // Reserved
            
            // Set up stack (using temporary stack in low memory)
            ldr rax, =0x40000  // Temporary stack at 256KB
            mov rsp, rax
            
            // Call rust entry point
            call rust_x86_64_entry
            
            // Should never reach here
            cli
            hlt
            .loop:
            jmp .loop
            ",
            options(noreturn)
        );
    }
}

/// Rust entry point after basic setup
#[no_mangle]
extern "C" fn rust_x86_64_entry(boot_info_ptr: usize) -> ! {
    // Initialize early debugging
    crate::arch::x86_64::serial::init();
    
    println!("XPARQ OS Booting on x86-64...");
    println!("Boot info at 0x{:x}", boot_info_ptr);
    
    // Parse boot information
    let boot_info = parse_boot_info(boot_info_ptr);
    
    // Call architecture-agnostic kernel main
    crate::kernel_main(&boot_info);
}

/// Parse boot information from UEFI
fn parse_boot_info(boot_info_ptr: usize) -> crate::BootInfo {
    println!("Parsing x86-64 boot information...");
    
    // Phase 1: Create dummy boot info
    // Phase 2: Parse actual UEFI boot info
    
    let memory_regions = &[
        crate::MemoryRegion {
            base: 0x100000,
            size: 512 * 1024 * 1024, // 512MB starting at 1MB
            kind: crate::MemoryRegionKind::Usable,
        },
        crate::MemoryRegion {
            base: 0x3F8,
            size: 0x8, // COM1 serial port
            kind: crate::MemoryRegionKind::Mmio,
        },
    ];
    
    let framebuffer = Some(crate::FramebufferInfo {
        address: 0xFD000000,
        width: 1024,
        height: 768,
        stride: 1024,
        format: crate::PixelFormat::Rgb32,
    });
    
    let arch_specific = crate::ArchBootInfo {
        rsdp: find_rsdp(),
        bootloader_brand: "UEFI",
    };
    
    crate::BootInfo {
        memory_regions,
        framebuffer,
        arch_specific,
    }
}

/// Find RSDP (Root System Description Pointer)
fn find_rsdp() -> usize {
    println!("Finding RSDP...");
    
    // Phase 1: Use dummy RSDP address
    // Phase 2: Search EBDA and reserved memory for RSDP
    
    let rsdp_address = 0xF0000; // Dummy address in BIOS area
    
    println!("RSDP at 0x{:x}", rsdp_address);
    
    rsdp_address
}

/// x86-64 CPU initialization
pub fn init_cpu() {
    println!("Initializing x86-64 CPU...");
    
    // Set up control registers
    setup_control_registers();
    
    // Enable caches
    enable_caches();
    
    // Set up memory management unit
    crate::arch::x86_64::mmu::init();
    
    println!("x86-64 CPU initialized");
}

/// Set up x86-64 control registers
fn setup_control_registers() {
    println!("Setting up control registers...");
    
    unsafe {
        let mut cr0: u64;
        let mut cr4: u64;
        
        // Read CR0
        core::arch::asm!("mov {}, cr0", out(reg) cr0);
        
        // Enable protected mode, paging, write protect
        cr0 |= (1 << 31) | (1 << 0) | (1 << 16);
        
        // Disable emulation coprocessor (we're in 64-bit mode)
        cr0 &= !(1 << 2);
        
        // Write CR0
        core::arch::asm!("mov cr0, {}", in(reg) cr0);
        
        // Read CR4
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        
        // Enable PSE (Page Size Extension)
        cr4 |= (1 << 4);
        
        // Write CR4
        core::arch::asm!("mov cr4, {}", in(reg) cr4);
        
        // Enable FXSAVE/FXRSTOR for SSE
        let mut cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        cr4 |= (1 << 9); // OSFXSR
        cr4 |= (1 << 10); // OSXMMEXCPT
        core::arch::asm!("mov cr4, {}", in(reg) cr4);
    }
    
    println!("Control registers set up");
}

/// Enable x86-64 caches
fn enable_caches() {
    println!("Enabling caches...");
    
    unsafe {
        let mut cr0: u64;
        core::arch::asm!("mov {}, cr0", out(reg) cr0);
        
        // Clear CD (Cache Disable) and NW (Not Write-through) bits
        cr0 &= !(1 << 29) & !(1 << 30);
        
        // Write CR0
        core::arch::asm!("mov cr0, {}", in(reg) cr0);
        
        // Invalidate caches
        core::arch::asm!("wbinvd");
    }
    
    println!("Caches enabled");
}

/// x86-64 system register access utilities
pub mod regs {
    /// Read model-specific register
    #[inline(always)]
    pub unsafe fn rdmsr(msr: u32) -> u64 {
        let low: u32;
        let high: u32;
        core::arch::asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
        );
        ((high as u64) << 32) | (low as u64)
    }
    
    /// Write model-specific register
    #[inline(always)]
    pub unsafe fn wrmsr(msr: u32, value: u64) {
        let low = value as u32;
        let high = (value >> 32) as u32;
        core::arch::asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
        );
    }
    
    /// Read control register 0
    #[inline(always)]
    pub fn read_cr0() -> u64 {
        let cr0: u64;
        unsafe { core::arch::asm!("mov {}, cr0", out(reg) cr0); }
        cr0
    }
    
    /// Write control register 0
    #[inline(always)]
    pub unsafe fn write_cr0(value: u64) {
        core::arch::asm!("mov cr0, {}", in(reg) value);
    }
    
    /// Read control register 3
    #[inline(always)]
    pub fn read_cr3() -> u64 {
        let cr3: u64;
        unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3); }
        cr3
    }
    
    /// Write control register 3
    #[inline(always)]
    pub unsafe fn write_cr3(value: u64) {
        core::arch::asm!("mov cr3, {}", in(reg) value);
    }
    
    /// Read control register 4
    #[inline(always)]
    pub fn read_cr4() -> u64 {
        let cr4: u64;
        unsafe { core::arch::asm!("mov {}, cr4", out(reg) cr4); }
        cr4
    }
    
    /// Write control register 4
    #[inline(always)]
    pub unsafe fn write_cr4(value: u64) {
        core::arch::asm!("mov cr4, {}", in(reg) value);
    }
    
    /// Get CPUID information
    pub fn cpuid(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
        let (eax, ebx, ecx, edx);
        unsafe {
            core::arch::asm!(
                "cpuid",
                in("eax") leaf,
                in("ecx") subleaf,
                out("eax") eax,
                out("ebx") ebx,
                out("ecx") ecx,
                out("edx") edx,
            );
        }
        (eax, ebx, ecx, edx)
    }
    
    /// Read time stamp counter
    #[inline(always)]
    pub fn rdtsc() -> u64 {
        let low: u32;
        let high: u32;
        unsafe {
            core::arch::asm!(
                "rdtsc",
                out("eax") low,
                out("edx") high,
            );
        }
        ((high as u64) << 32) | (low as u64)
    }
    
    /// Read time stamp counter with serialization
    #[inline(always)]
    pub fn rdtscp() -> (u64, u32) {
        let low: u32;
        let high: u32;
        let aux: u32;
        unsafe {
            core::arch::asm!(
                "rdtscp",
                out("eax") low,
                out("edx") high,
                out("ecx") aux,
            );
        }
        (((high as u64) << 32) | (low as u64), aux)
    }
}

/// x86-64 exception handling
pub mod exceptions {
    use core::sync::atomic::{AtomicU64, Ordering};
    
    static EXCEPTION_COUNT: AtomicU64 = AtomicU64::new(0);
    
    /// Interrupt Descriptor Table (IDT)
    #[repr(C, align(16))]
    pub struct IDTEntry {
        pub offset_low: u16,
        pub selector: u16,
        pub ist: u8,
        pub type_attr: u8,
        pub offset_mid: u16,
        pub offset_high: u32,
        pub reserved: u32,
    }
    
    impl IDTEntry {
        /// Create a new IDT entry
        pub fn new(offset: u64, selector: u16, type_attr: u8, ist: u8) -> Self {
            Self {
                offset_low: (offset & 0xFFFF) as u16,
                selector,
                ist,
                type_attr,
                offset_mid: ((offset >> 16) & 0xFFFF) as u16,
                offset_high: ((offset >> 32) & 0xFFFFFFFF) as u32,
                reserved: 0,
            }
        }
        
        /// Create an interrupt gate
        pub fn interrupt_gate(offset: u64, selector: u16, dpl: u8) -> Self {
            let type_attr = 0x8E | ((dpl & 0x3) << 5); // Present, DPL, 64-bit interrupt gate
            Self::new(offset, selector, type_attr, 0)
        }
        
        /// Create a trap gate
        pub fn trap_gate(offset: u64, selector: u64, dpl: u8) -> Self {
            let type_attr = 0x8F | ((dpl & 0x3) << 5); // Present, DPL, 64-bit trap gate
            Self::new(offset, selector as u16, type_attr, 0)
        }
    }
    
    /// Global IDT
    static mut IDT: [IDTEntry; 256] = [IDTEntry {
        offset_low: 0,
        selector: 0,
        ist: 0,
        type_attr: 0,
        offset_mid: 0,
        offset_high: 0,
        reserved: 0,
    }; 256];
    
    /// Set up IDT
    pub fn setup_idt() {
        println!("Setting up IDT...");
        
        unsafe {
            // Set up exception handlers
            for i in 0..32 {
                IDT[i] = IDTEntry::interrupt_gate(exception_handler as u64, 0x08, 0);
            }
            
            // Set up interrupt handlers
            for i in 32..256 {
                IDT[i] = IDTEntry::interrupt_gate(interrupt_handler as u64, 0x08, 0);
            }
            
            // Load IDT
            let idt_ptr = IDTPointer {
                limit: (256 * 16 - 1) as u16,
                base: IDT.as_ptr() as u64,
            };
            
            core::arch::asm!("lidt [{}]", in(reg) &idt_ptr);
        }
        
        println!("IDT set up");
    }
    
    /// IDT pointer structure
    #[repr(C, packed)]
    struct IDTPointer {
        limit: u16,
        base: u64,
    }
    
    /// Generic exception handler
    #[no_mangle]
    extern "C" fn exception_handler() {
        let count = EXCEPTION_COUNT.fetch_add(1, Ordering::SeqCst);
        println!("Exception #{}", count);
        
        // Phase 1: Basic exception handling
        // Phase 2: Full exception processing
        
        // Get exception information
        unsafe {
            let mut error_code: u64 = 0;
            let mut vector: u64 = 0;
            
            // For Phase 1, we'll use dummy values
            // Phase 2: Read actual exception vector and error code
            
            println!("Exception vector: {}", vector);
            println!("Error code: 0x{:x}", error_code);
        }
        
        // For now, just return from exception
        // Phase 2: Proper exception handling
    }
    
    /// Generic interrupt handler
    #[no_mangle]
    extern "C" fn interrupt_handler() {
        println!("Interrupt received");
        
        // Phase 1: Basic interrupt handling
        // Phase 2: Full interrupt processing
        
        // Send EOI to PIC
        send_eoi(32); // Dummy IRQ number
    }
    
    /// Send End of Interrupt to PIC
    fn send_eoi(irq: u32) {
        // Phase 1: Dummy implementation
        // Phase 2: Real PIC handling
        
        unsafe {
            // Send EOI to master PIC
            let pic1_cmd = 0x20 as *mut u8;
            pic1_cmd.write_volatile(0x20);
            
            // If IRQ > 7, send EOI to slave PIC
            if irq >= 8 {
                let pic2_cmd = 0xA0 as *mut u8;
                pic2_cmd.write_volatile(0x20);
            }
        }
    }
    
    /// System call handler
    #[no_mangle]
    extern "C" fn syscall_handler() {
        println!("System call received");
        
        // Phase 1: Basic syscall handling
        // Phase 2: Full syscall implementation
        
        // Get syscall number from registers
        let syscall_number = 0; // Phase 2: Read from RAX
        
        println!("System call: {}", syscall_number);
        
        match syscall_number {
            0 => {
                // Exit syscall
                println!("Exit syscall");
            }
            1 => {
                // Write syscall
                println!("Write syscall");
            }
            _ => {
                println!("Unknown syscall: {}", syscall_number);
            }
        }
    }
}

/// x86-64 memory barriers
pub mod barriers {
    /// Memory fence
    #[inline(always)]
    pub fn mfence() {
        unsafe {
            core::arch::asm!("mfence");
        }
    }
    
    /// Store fence
    #[inline(always)]
    pub fn sfence() {
        unsafe {
            core::arch::asm!("sfence");
        }
    }
    
    /// Load fence
    #[inline(always)]
    pub fn lfence() {
        unsafe {
            core::arch::asm!("lfence");
        }
    }
    
    /// Full memory barrier
    #[inline(always)]
    pub fn memory_barrier() {
        unsafe {
            core::arch::asm!("mfence");
        }
    }
}
