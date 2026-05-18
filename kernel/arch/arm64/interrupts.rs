// XPARQ OS - Phase 01: OS & Kernel Foundations
// ARM64 interrupt management
// Handles exception vectors, interrupt controller, and interrupt handling

#![no_std]

use core::sync::atomic::{AtomicU64, Ordering};

/// Exception vector table
#[repr(C, align(2048))]
pub struct ExceptionVectors {
    pub current_el_sp0: [u8; 0x80],    // 0x000: Current EL with SP0
    pub current_el_spx: [u8; 0x80],    // 0x080: Current EL with SPx
    pub lower_el_aarch64: [u8; 0x80],  // 0x100: Lower EL using AArch64
    pub lower_el_aarch32: [u8; 0x80],  // 0x180: Lower EL using AArch32
}

/// Exception types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExceptionType {
    Synchronous,
    Irq,
    Fiq,
    SError,
}

/// Exception classes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExceptionClass {
    Unknown,
    Wfi,
    Wfe,
    Svc,
    Hvc,
    Smc,
    MsrMrs,
    SystemInstruction,
    InstructionAbort,
    DataAbort,
    PcAlignment,
    SpAlignment,
    SveSimdFault,
    TrappedFpException,
    TrappedSveException,
    TrappedSimdFpException,
    SmeException,
    SmeInstructionException,
    Eret,
    Pstate,
    IllegalExecutionState,
    Bti,
}

/// GIC (Generic Interrupt Controller) interface
pub struct GIC {
    /// GIC Distributor base address
    pub distributor_base: usize,
    /// GIC CPU interface base address
    pub cpu_interface_base: usize,
}

/// GIC Distributor registers
#[repr(C)]
pub struct GICDistributor {
    pub ctrl: volatile::Volatile<u32>,        // 0x000
    pub typer: volatile::Volatile<u32>,       // 0x004
    pub iidr: volatile::Volatile<u32>,        // 0x008
    pub _reserved1: [volatile::Volatile<u32>; 9], // 0x00C-0x02C
    pub statusr: volatile::Volatile<u32>,      // 0x030
    pub _reserved2: [volatile::Volatile<u32>; 3], // 0x034-0x03C
    pub setspi_nsr: volatile::Volatile<u32>,   // 0x040
    pub clrspi_nsr: volatile::Volatile<u32>,   // 0x044
    pub setspi_sr: volatile::Volatile<u32>,    // 0x048
    pub clrspi_sr: volatile::Volatile<u32>,    // 0x04C
    pub _reserved3: [volatile::Volatile<u32>; 8], // 0x050-0x06C
    pub igroupr: [volatile::Volatile<u32>; 32], // 0x080-0x0FC
    pub isenabler: [volatile::Volatile<u32>; 32], // 0x100-0x17C
    pub icer: [volatile::Volatile<u32>; 32],   // 0x180-0x1FC
    pub ispendr: [volatile::Volatile<u32>; 32], // 0x200-0x27C
    pub icpendr: [volatile::Volatile<u32>; 32], // 0x280-0x2FC
    pub isactiver: [volatile::Volatile<u32>; 32], // 0x300-0x37C
    pub icactiver: [volatile::Volatile<u32>; 32], // 0x380-0x3FC
    pub ipriorityr: [volatile::Volatile<u8>; 1024], // 0x400-0x7FC
    pub _reserved4: [volatile::Volatile<u32>; 256], // 0x800-0xBFC
    pub itargetsr: [volatile::Volatile<u8>; 1024], // 0x800-0xBFC
    pub _reserved5: [volatile::Volatile<u32>; 256], // 0xC00-0xEFC
    pub icfgr: [volatile::Volatile<u32>; 64], // 0xC00-0xEFC
    pub _reserved6: [volatile::Volatile<u32>; 64], // 0xF00-0xFFC
    pub sgir: volatile::Volatile<u32>,        // 0xF00
    pub _reserved7: [volatile::Volatile<u32>; 3], // 0xF04-0xF0C
    pub cpendsgir: [volatile::Volatile<u32>; 4], // 0xF10-0xF1C
    pub spendsgir: [volatile::Volatile<u32>; 4], // 0xF20-0xF2C
    pub _reserved8: [volatile::Volatile<u32>; 52], // 0xF30-0xFFC
}

/// GIC CPU Interface registers
#[repr(C)]
pub struct GICCpuInterface {
    pub ctrl: volatile::Volatile<u32>,        // 0x000
    pub pmr: volatile::Volatile<u32>,         // 0x004
    pub bpr: volatile::Volatile<u32>,         // 0x008
    pub iar: volatile::Volatile<u32>,         // 0x00C
    pub eoir: volatile::Volatile<u32>,        // 0x010
    pub rpr: volatile::Volatile<u32>,         // 0x014
    pub hpiir: volatile::Volatile<u32>,       // 0x018
    pub abpr: volatile::Volatile<u32>,        // 0x01C
    pub aiar: volatile::Volatile<u32>,        // 0x020
    pub aeoir: volatile::Volatile<u32>,       // 0x024
    pub arpr: volatile::Volatile<u32>,        // 0x028
    pub _reserved1: [volatile::Volatile<u32>; 52], // 0x02C-0xFC
    pub apr: [volatile::Volatile<u32>; 4],    // 0x100-0x10C
    pub nsapr: [volatile::Volatile<u32>; 4],  // 0x110-0x11C
    pub _reserved2: [volatile::Volatile<u32>; 3], // 0x120-0x128
    pub hpiir: volatile::Volatile<u32>,       // 0x12C
    pub _reserved3: [volatile::Volatile<u32>; 9], // 0x130-0x150
    pub dir: volatile::Volatile<u32>,         // 0x1000
    pub _reserved4: [volatile::Volatile<u32>; 1023], // 0x1004-0x1FFC
}

/// Global exception vectors
static mut EXCEPTION_VECTORS: Option<ExceptionVectors> = None;
static mut GIC_INSTANCE: Option<GIC> = None;

/// Interrupt statistics
static INTERRUPT_COUNT: AtomicU64 = AtomicU64::new(0);
static EXCEPTION_COUNT: AtomicU64 = AtomicU64::new(0);

/// Set up exception vectors
pub fn setup_vectors() {
    println!("Setting up exception vectors...");
    
    unsafe {
        // Allocate exception vectors
        EXCEPTION_VECTORS = Some(ExceptionVectors {
            current_el_sp0: [0; 0x80],
            current_el_spx: [0; 0x80],
            lower_el_aarch64: [0; 0x80],
            lower_el_aarch32: [0; 0x80],
        });
        
        if let Some(vectors) = &mut EXCEPTION_VECTORS {
            // Set up basic exception handlers
            setup_exception_handlers(vectors);
            
            // Set VBAR_EL1 to point to our exception vectors
            let vbar = vectors as *const ExceptionVectors as u64;
            core::arch::asm!("msr VBAR_EL1, {}", in(reg) vbar);
            
            // Data synchronization barrier
            super::boot::regs::dsb();
            super::boot::regs::isb();
        }
    }
    
    println!("Exception vectors set up");
}

/// Set up exception handlers
fn setup_exception_handlers(vectors: &mut ExceptionVectors) {
    // Phase 1: Basic exception handlers
    // Phase 2: Full exception handling with proper context save/restore
    
    // Current EL with SP0 - synchronous
    vectors.current_el_sp0[0..4].copy_from_slice(&[
        0x01, 0x00, 0x20, 0xD4, // b current_el_sp0_sync
    ]);
    
    // Current EL with SP0 - IRQ
    vectors.current_el_sp0[0x80..0x84].copy_from_slice(&[
        0x01, 0x00, 0x20, 0xD4, // b current_el_sp0_irq
    ]);
    
    // Current EL with SP0 - FIQ
    vectors.current_el_sp0[0x100..0x104].copy_from_slice(&[
        0x01, 0x00, 0x20, 0xD4, // b current_el_sp0_fiq
    ]);
    
    // Current EL with SP0 - SError
    vectors.current_el_sp0[0x180..0x184].copy_from_slice(&[
        0x01, 0x00, 0x20, 0xD4, // b current_el_sp0_serror
    ]);
    
    // Similar setup for other exception levels...
    // For Phase 1, we'll keep it simple
}

/// Initialize GIC
pub fn init_gic() {
    println!("Initializing GIC...");
    
    // Phase 1: Use standard QEMU GIC addresses
    // Phase 2: Get addresses from device tree
    
    const GICD_BASE: usize = 0x08000000; // GIC Distributor
    const GICC_BASE: usize = 0x08010000; // GIC CPU Interface
    
    unsafe {
        GIC_INSTANCE = Some(GIC {
            distributor_base: GICD_BASE,
            cpu_interface_base: GICC_BASE,
        });
        
        if let Some(gic) = &GIC_INSTANCE {
            let distributor = &mut *(gic.distributor_base as *mut GICDistributor);
            let cpu_interface = &mut *(gic.cpu_interface_base as *mut GICCpuInterface);
            
            // Disable GIC while configuring
            distributor.ctrl.write(0);
            
            // Set all interrupts to group 1 (non-secure)
            for i in 0..32 {
                distributor.igroupr[i].write(0xFFFFFFFF);
            }
            
            // Enable GIC
            distributor.ctrl.write(1);
            
            // Enable CPU interface
            cpu_interface.ctrl.write(1);
            
            // Set priority mask
            cpu_interface.pmr.write(0xFF);
        }
    }
    
    println!("GIC initialized");
}

/// Enable interrupts
pub fn enable() {
    println!("Enabling interrupts...");
    
    // Enable GIC
    init_gic();
    
    // Enable IRQ and FIQ
    unsafe {
        let mut daif: u64;
        core::arch::asm!("mrs {}, DAIF", out(reg) daif);
        daif &= !(1 << 7); // Clear I bit (IRQ enable)
        daif &= !(1 << 6); // Clear F bit (FIQ enable)
        core::arch::asm!("msr DAIF, {}", in(reg) daif);
    }
    
    println!("Interrupts enabled");
}

/// Disable interrupts
pub fn disable() {
    println!("Disabling interrupts...");
    
    // Disable IRQ and FIQ
    unsafe {
        let mut daif: u64;
        core::arch::asm!("mrs {}, DAIF", out(reg) daif);
        daif |= (1 << 7); // Set I bit (IRQ disable)
        daif |= (1 << 6); // Set F bit (FIQ disable)
        core::arch::asm!("msr DAIF, {}", in(reg) daif);
    }
    
    println!("Interrupts disabled");
}

/// Enable specific interrupt
pub fn enable_irq(irq: u32) {
    unsafe {
        if let Some(gic) = &GIC_INSTANCE {
            let distributor = &mut *(gic.distributor_base as *mut GICDistributor);
            let reg_index = irq / 32;
            let bit_index = irq % 32;
            
            distributor.isenabler[reg_index as usize].write(1 << bit_index);
        }
    }
}

/// Disable specific interrupt
pub fn disable_irq(irq: u32) {
    unsafe {
        if let Some(gic) = &GIC_INSTANCE {
            let distributor = &mut *(gic.distributor_base as *mut GICDistributor);
            let reg_index = irq / 32;
            let bit_index = irq % 32;
            
            distributor.icer[reg_index as usize].write(1 << bit_index);
        }
    }
}

/// Get pending interrupt
pub fn get_pending_irq() -> Option<u32> {
    unsafe {
        if let Some(gic) = &GIC_INSTANCE {
            let cpu_interface = &mut *(gic.cpu_interface_base as *mut GICCpuInterface);
            let iar = cpu_interface.iar.read();
            
            // Check if interrupt ID is valid
            if iar < 1023 {
                Some(iar as u32)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// End of interrupt
pub fn end_of_interrupt(irq: u32) {
    unsafe {
        if let Some(gic) = &GIC_INSTANCE {
            let cpu_interface = &mut *(gic.cpu_interface_base as *mut GICCpuInterface);
            cpu_interface.eoir.write(irq as u32);
        }
    }
}

/// Exception handlers
#[no_mangle]
extern "C" fn current_el_sp0_sync() {
    exception_handler(ExceptionType::Synchronous);
}

#[no_mangle]
extern "C" fn current_el_sp0_irq() {
    exception_handler(ExceptionType::Irq);
}

#[no_mangle]
extern "C" fn current_el_sp0_fiq() {
    exception_handler(ExceptionType::Fiq);
}

#[no_mangle]
extern "C" fn current_el_sp0_serror() {
    exception_handler(ExceptionType::SError);
}

/// Generic exception handler
fn exception_handler(exception_type: ExceptionType) {
    let count = EXCEPTION_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("Exception #{}: {:?}", count, exception_type);
    
    // Get exception syndrome
    unsafe {
        let esr: u64;
        core::arch::asm!("mrs {}, ESR_EL1", out(reg) esr);
        
        let ec = (esr >> 26) & 0x3F; // Exception class
        let il = (esr >> 25) & 0x1;  // Instruction length
        let iss = esr & 0x1FFFFFF;  // Instruction specific syndrome
        
        println!("ESR_EL1: EC={}, IL={}, ISS=0x{:x}", ec, il, iss);
        
        // Determine exception class
        let exception_class = match ec {
            0x01 => ExceptionClass::Wfi,
            0x02 => ExceptionClass::Wfe,
            0x15 => ExceptionClass::Svc,
            0x16 => ExceptionClass::Hvc,
            0x17 => ExceptionClass::Smc,
            0x18 => ExceptionClass::MsrMrs,
            0x19 => ExceptionClass::SystemInstruction,
            0x20 => ExceptionClass::InstructionAbort,
            0x21 => ExceptionClass::InstructionAbort,
            0x24 => ExceptionClass::DataAbort,
            0x25 => ExceptionClass::DataAbort,
            0x26 => ExceptionClass::SpAlignment,
            0x27 => ExceptionClass::PcAlignment,
            0x2C => ExceptionClass::SveSimdFault,
            0x2D => ExceptionClass::TrappedFpException,
            0x34 => ExceptionClass::TrappedSveException,
            0x35 => ExceptionClass::TrappedSimdFpException,
            0x36 => ExceptionClass::SmeException,
            0x37 => ExceptionClass::SmeInstructionException,
            0x38 => ExceptionClass::Eret,
            0x3C => ExceptionClass::Bti,
            _ => ExceptionClass::Unknown,
        };
        
        println!("Exception class: {:?}", exception_class);
        
        // Handle specific exception types
        match exception_type {
            ExceptionType::Irq => {
                handle_irq();
            }
            ExceptionType::Synchronous => {
                handle_synchronous_exception(exception_class, iss);
            }
            _ => {
                println!("Unhandled exception type: {:?}", exception_type);
            }
        }
    }
}

/// Handle IRQ
fn handle_irq() {
    let count = INTERRUPT_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("IRQ #{}", count);
    
    // Get pending interrupt
    if let Some(irq) = get_pending_irq() {
        println!("IRQ: {}", irq);
        
        // Handle specific IRQs
        match irq {
            0 => {
                // Timer interrupt
                handle_timer_interrupt();
            }
            1 => {
                // UART interrupt
                handle_uart_interrupt();
            }
            _ => {
                println!("Unknown IRQ: {}", irq);
            }
        }
        
        // End of interrupt
        end_of_interrupt(irq);
    }
}

/// Handle timer interrupt
fn handle_timer_interrupt() {
    println!("Timer interrupt");
    
    // Phase 1: Basic timer handling
    // Phase 2: Full timer management
    
    // Read timer value
    let cntpct = super::boot::regs::cntpct();
    let cntfrq = super::boot::regs::cntfrq();
    
    println!("Timer: {} / {} Hz", cntpct, cntfrq);
}

/// Handle UART interrupt
fn handle_uart_interrupt() {
    println!("UART interrupt");
    
    // Phase 1: Basic UART handling
    // Phase 2: Full UART interrupt handling
    
    // Check UART status
    let uart_status = super::uart::get_status();
    println!("UART status: 0x{:x}", uart_status);
    
    // Read pending characters
    while let Some(c) = super::uart::read_char() {
        println!("UART received: 0x{:x}", c);
    }
}

/// Handle synchronous exception
fn handle_synchronous_exception(exception_class: ExceptionClass, iss: u64) {
    match exception_class {
        ExceptionClass::Svc => {
            println!("System call: {}", iss);
            handle_system_call(iss);
        }
        ExceptionClass::InstructionAbort => {
            println!("Instruction abort at 0x{:x}", get_exception_address());
        }
        ExceptionClass::DataAbort => {
            println!("Data abort at 0x{:x}", get_exception_address());
        }
        _ => {
            println!("Unhandled synchronous exception: {:?}", exception_class);
        }
    }
}

/// Handle system call
fn handle_system_call(syscall_number: u64) {
    println!("System call {}", syscall_number);
    
    // Phase 1: Basic syscall handling
    // Phase 2: Full syscall implementation
    
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

/// Get exception address
fn get_exception_address() -> usize {
    unsafe {
        let elr: u64;
        core::arch::asm!("mrs {}, ELR_EL1", out(reg) elr);
        elr as usize
    }
}

/// Interrupt statistics
pub mod stats {
    use super::*;
    
    /// Get interrupt count
    pub fn get_interrupt_count() -> u64 {
        INTERRUPT_COUNT.load(Ordering::SeqCst)
    }
    
    /// Get exception count
    pub fn get_exception_count() -> u64 {
        EXCEPTION_COUNT.load(Ordering::SeqCst)
    }
    
    /// Reset statistics
    pub fn reset_stats() {
        INTERRUPT_COUNT.store(0, Ordering::SeqCst);
        EXCEPTION_COUNT.store(0, Ordering::SeqCst);
    }
}
