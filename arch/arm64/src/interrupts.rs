//! ARM64 Interrupt Handling - Phase 2: Dev Environment Setup
//! 
//! This module provides ARM64 interrupt handling for XPARQ OS, including:
//! - GICv3 interrupt controller management
//! - IRQ, FIQ, and exception handling
//! - Interrupt routing and priority management
//! - Per-CPU interrupt state
//! 
//! Interrupt Controller: GICv3 (Generic Interrupt Controller v3)
//! Exception Types: IRQ, FIQ, SError, Debug
//! Priority Levels: 0-255 (higher number = higher priority)
//! Routing: Per-CPU redistributor + distributor
//! 
//! Roadmap Phase: Phase 2 - Dev Environment Setup

use super::{gic, sysreg, asm_utils};
use bitflags::bitflags;

/// Interrupt types
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum InterruptType {
    Irq = 0,
    Fiq = 1,
    SError = 2,
    Debug = 3,
}

/// Interrupt state for a CPU
#[derive(Debug)]
pub struct InterruptState {
    /// Current CPU ID
    pub cpu_id: u32,
    /// Interrupts enabled flag
    pub enabled: bool,
    /// Current interrupt priority mask
    pub priority_mask: u8,
    /// Interrupt statistics
    pub stats: InterruptStats,
}

/// Interrupt statistics
#[derive(Debug, Default)]
pub struct InterruptStats {
    pub total_interrupts: u64,
    pub irq_count: u64,
    pub fiq_count: u64,
    pub serror_count: u64,
    pub debug_count: u64,
}

/// Global interrupt manager
static mut INTERRUPT_STATE: Option<InterruptState> = None;

/// Initialize interrupt handling
pub fn init() {
    println!("Initializing ARM64 interrupt handling...");
    
    // Initialize GIC
    gic::init();
    
    // Set up exception vectors
    super::exception::init_vectors();
    
    // Initialize per-CPU interrupt state
    let cpu_id = super::cpu::current_cpu();
    let state = InterruptState {
        cpu_id,
        enabled: false,
        priority_mask: 0xFF, // All interrupts masked initially
        stats: InterruptStats::default(),
    };
    
    unsafe {
        INTERRUPT_STATE = Some(state);
    }
    
    // Enable exceptions
    super::exception::enable_exceptions();
    
    println!("ARM64 interrupt handling initialized for CPU {}", cpu_id);
}

/// Enable interrupts
pub fn enable() {
    let state = unsafe { INTERRUPT_STATE.as_mut().unwrap() };
    
    // Clear DAIF bits (D, A, I, F)
    let mut daif = sysreg::mrs("DAIF");
    daif &= !0b1100; // Clear I and F bits
    sysreg::msr("DAIF", daif);
    
    state.enabled = true;
    println!("Interrupts enabled for CPU {}", state.cpu_id);
}

/// Disable interrupts
pub fn disable() {
    let state = unsafe { INTERRUPT_STATE.as_mut().unwrap() };
    
    // Set DAIF bits
    let mut daif = sysreg::mrs("DAIF");
    daif |= 0b1100; // Set I and F bits
    sysreg::msr("DAIF", daif);
    
    state.enabled = false;
    println!("Interrupts disabled for CPU {}", state.cpu_id);
}

/// Set interrupt priority mask
pub fn set_priority_mask(mask: u8) {
    let state = unsafe { INTERRUPT_STATE.as_mut().unwrap() };
    state.priority_mask = mask;
    
    // Set PMR register
    sysreg::msr("PMR_EL1", mask as u64);
}

/// Enable specific interrupt
pub fn enable_irq(irq: u32) {
    gic::enable_irq(irq);
    gic::set_priority(irq, 0x80); // Medium priority
    println!("Enabled IRQ {}", irq);
}

/// Disable specific interrupt
pub fn disable_irq(irq: u32) {
    // Phase 2: Implement GIC disable
    println!("Disabled IRQ {} (placeholder)", irq);
}

/// Handle interrupt (called from assembly)
#[no_mangle]
pub extern "C" fn handle_interrupt(interrupt_type: u32, irq_number: u32) {
    let int_type = match interrupt_type {
        0 => InterruptType::Irq,
        1 => InterruptType::Fiq,
        2 => InterruptType::SError,
        3 => InterruptType::Debug,
        _ => return,
    };
    
    let state = unsafe { INTERRUPT_STATE.as_mut().unwrap() };
    
    // Update statistics
    state.stats.total_interrupts += 1;
    match int_type {
        InterruptType::Irq => state.stats.irq_count += 1,
        InterruptType::Fiq => state.stats.fiq_count += 1,
        InterruptType::SError => state.stats.serror_count += 1,
        InterruptType::Debug => state.stats.debug_count += 1,
    }
    
    // Handle specific interrupt
    match int_type {
        InterruptType::Irq => handle_irq(irq_number),
        InterruptType::Fiq => handle_fiq(irq_number),
        InterruptType::SError => handle_serror(),
        InterruptType::Debug => handle_debug(),
    }
}

/// Handle IRQ
fn handle_irq(irq_number: u32) {
    println!("Handling IRQ {} on CPU {}", irq_number, super::cpu::current_cpu());
    
    // Phase 2: Dispatch to interrupt handlers
    // Phase 3: Full interrupt routing with device drivers
    
    // Acknowledge interrupt
    gic::acknowledge_irq(irq_number);
}

/// Handle FIQ
fn handle_fiq(irq_number: u32) {
    println!("Handling FIQ {} on CPU {}", irq_number, super::cpu::current_cpu());
    
    // Phase 2: High-priority interrupt handling
}

/// Handle SError
fn handle_serror() {
    println!("Handling SError on CPU {}", super::cpu::current_cpu());
    
    // Phase 2: System error handling and recovery
}

/// Handle Debug exception
fn handle_debug() {
    println!("Handling Debug exception on CPU {}", super::cpu::current_cpu());
    
    // Phase 2: Debug exception handling
}

/// GIC extension functions
impl gic {
    /// Acknowledge interrupt
    pub fn acknowledge_irq(irq: u32) {
        // Phase 2: Implement GIC acknowledge
        println!("Acknowledged IRQ {}", irq);
    }
    
    /// End of interrupt
    pub fn end_of_interrupt(irq: u32) {
        // Phase 2: Implement GIC EOI
        println!("EOI for IRQ {}", irq);
    }
}
