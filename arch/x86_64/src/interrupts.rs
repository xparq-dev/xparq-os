//! x86-64 Interrupt Handling - Phase 2: Dev Environment Setup
//! 
//! This module provides x86-64 interrupt handling for XPARQ OS, including:
//! - Local APIC interrupt controller management
//! - IRQ handling and routing
//! - Exception handling with proper context save/restore
//! - Per-CPU interrupt state
//! - Interrupt statistics
//! 
//! Interrupt Controller: Local APIC (x2APIC if available)
//! Exception Types: x86-64 exceptions (divide by zero, page fault, etc.)
/// IRQ Types: Legacy ISA IRQs, PCI IRQs, APIC timer
//! Priority Levels: APIC priority register (0-15)
//! 
//! Roadmap Phase: Phase 2 - Dev Environment Setup

use super::{apic, sysreg, asm_utils};
use x86_64::structures::idt::InterruptDescriptorTable;

/// Interrupt types
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum InterruptType {
    Exception = 0,
    Irq = 1,
    Nmi = 2,
    Timer = 3,
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
    pub exception_count: u64,
    pub irq_count: u64,
    pub nmi_count: u64,
    pub timer_count: u64,
}

/// Global interrupt manager
static mut INTERRUPT_STATE: Option<InterruptState> = None;

/// Initialize interrupt handling
pub fn init() {
    println!("Initializing x86-64 interrupt handling...");
    
    // Initialize IDT
    super::exception::init_idt();
    
    // Initialize Local APIC
    apic::init();
    
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
    
    println!("x86-64 interrupt handling initialized for CPU {}", cpu_id);
}

/// Enable interrupts
pub fn enable() {
    let state = unsafe { INTERRUPT_STATE.as_mut().unwrap() };
    
    unsafe {
        x86_64::instructions::interrupts::enable();
    }
    
    state.enabled = true;
    println!("Interrupts enabled for CPU {}", state.cpu_id);
}

/// Disable interrupts
pub fn disable() {
    let state = unsafe { INTERRUPT_STATE.as_mut().unwrap() };
    
    unsafe {
        x86_64::instructions::interrupts::disable();
    }
    
    state.enabled = false;
    println!("Interrupts disabled for CPU {}", state.cpu_id);
}

/// Set interrupt priority mask
pub fn set_priority_mask(mask: u8) {
    let state = unsafe { INTERRUPT_STATE.as_mut().unwrap() };
    state.priority_mask = mask;
    
    // Phase 2: Set APIC task priority register
    println!("Set interrupt priority mask to {}", mask);
}

/// Enable specific interrupt
pub fn enable_irq(interrupt: u8) {
    apic::enable_irq(interrupt);
    apic::set_priority(interrupt, 0x80); // Medium priority
    println!("Enabled IRQ {}", interrupt);
}

/// Disable specific interrupt
pub fn disable_irq(interrupt: u8) {
    // Phase 2: Implement APIC disable
    println!("Disabled IRQ {} (placeholder)", interrupt);
}

/// Handle interrupt (called from assembly)
pub fn handle_interrupt(interrupt_type: u32, interrupt_number: u32, error_code: u64) {
    let int_type = match interrupt_type {
        0 => InterruptType::Exception,
        1 => InterruptType::Irq,
        2 => InterruptType::Nmi,
        3 => InterruptType::Timer,
        _ => return,
    };
    
    let state = unsafe { INTERRUPT_STATE.as_mut().unwrap() };
    
    // Update statistics
    state.stats.total_interrupts += 1;
    match int_type {
        InterruptType::Exception => state.stats.exception_count += 1,
        InterruptType::Irq => state.stats.irq_count += 1,
        InterruptType::Nmi => state.stats.nmi_count += 1,
        InterruptType::Timer => state.stats.timer_count += 1,
    }
    
    // Handle specific interrupt
    match int_type {
        InterruptType::Exception => handle_exception(interrupt_number, error_code),
        InterruptType::Irq => handle_irq(interrupt_number),
        InterruptType::Nmi => handle_nmi(),
        InterruptType::Timer => handle_timer(),
    }
}

/// Handle exception
fn handle_exception(exception_number: u32, error_code: u64) {
    println!("Handling exception {} on CPU {}", exception_number, super::cpu::current_cpu());
    
    // Phase 2: Dispatch to specific exception handlers
    // Phase 3: Full exception processing with signal handling
    
    match exception_number {
        14 => {
            // Page fault
            let fault_address = sysreg::read_cr3(); // Phase 2: Read CR2 for fault address
            println!("Page fault at 0x{:x}, error_code=0x{:x}", fault_address.as_u64(), error_code);
        }
        13 => {
            // General protection fault
            println!("General protection fault, error_code=0x{:x}", error_code);
        }
        _ => {
            println!("Unknown exception {}", exception_number);
        }
    }
}

/// Handle IRQ
fn handle_irq(irq_number: u32) {
    println!("Handling IRQ {} on CPU {}", irq_number, super::cpu::current_cpu());
    
    // Phase 2: Dispatch to interrupt handlers
    // Phase 3: Full interrupt routing with device drivers
    
    // Acknowledge interrupt
    apic::acknowledge_irq(irq_number as u8);
}

/// Handle NMI
fn handle_nmi() {
    println!("Handling NMI on CPU {}", super::cpu::current_cpu());
    
    // Phase 2: Non-maskable interrupt handling
    // Phase 3: System error handling and recovery
}

/// Handle timer interrupt
fn handle_timer() {
    println!("Handling timer interrupt on CPU {}", super::cpu::current_cpu());
    
    // Phase 2: APIC timer handling
    // Phase 3: High-resolution timer management
    
    // Notify scheduler
    crate::scheduler::yield_cpu();
}

/// APIC extension functions
impl apic {
    /// Acknowledge interrupt
    pub fn acknowledge_irq(irq: u32) {
        // Phase 2: Implement APIC EOI
        println!("Acknowledged IRQ {}", irq);
    }
    
    /// End of interrupt
    pub fn end_of_interrupt(irq: u32) {
        // Phase 2: Implement APIC EOI
        println!("EOI for IRQ {}", irq);
    }
    
    /// Get current interrupt vector
    pub fn get_interrupt_vector() -> u32 {
        // Phase 2: Read APIC ISR register
        0
    }
}

/// Interrupt descriptor table management
pub mod idt {
    use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
    
    /// Set interrupt handler
    pub fn set_handler(idt: &mut InterruptDescriptorTable, vector: u8, handler: extern "x86-interrupt" fn(&mut InterruptStackFrame)) {
        match vector {
            0 => idt.divide_error.set_handler_fn(handler),
            1 => idt.debug.set_handler_fn(handler),
            2 => idt.non_maskable_interrupt.set_handler_fn(handler),
            3 => idt.breakpoint.set_handler_fn(handler),
            4 => idt.overflow.set_handler_fn(handler),
            5 => idt.bound_range_exceeded.set_handler_fn(handler),
            6 => idt.invalid_opcode.set_handler_fn(handler),
            7 => idt.device_not_available.set_handler_fn(handler),
            8 => idt.double_fault.set_handler_fn(handler),
            10 => idt.invalid_tss.set_handler_fn(handler),
            11 => idt.segment_not_present.set_handler_fn(handler),
            12 => idt.stack_segment_fault.set_handler_fn(handler),
            13 => idt.general_protection_fault.set_handler_fn(handler),
            14 => idt.page_fault.set_handler_fn(handler),
            16 => idt.x87_floating_point.set_handler_fn(handler),
            17 => idt.alignment_check.set_handler_fn(handler),
            18 => idt.machine_check.set_handler_fn(handler),
            19 => idt.simd_floating_point.set_handler_fn(handler),
            20 => idt.virtualization.set_handler_fn(handler),
            30 => idt.security.set_handler_fn(handler),
            _ => println!("Unsupported interrupt vector {}", vector),
        }
    }
    
    /// Set interrupt handler with error code
    pub fn set_handler_with_error_code(idt: &mut InterruptDescriptorTable, vector: u8, handler: extern "x86-interrupt" fn(&mut InterruptStackFrame, u64)) {
        match vector {
            8 => idt.double_fault.set_handler_fn(handler),
            11 => idt.segment_not_present.set_handler_fn(handler),
            12 => idt.stack_segment_fault.set_handler_fn(handler),
            13 => idt.general_protection_fault.set_handler_fn(handler),
            14 => idt.page_fault.set_handler_fn(handler),
            17 => idt.alignment_check.set_handler_fn(handler),
            _ => println!("Unsupported interrupt vector with error code {}", vector),
        }
    }
    
    /// Set IRQ handler
    pub fn set_irq_handler(idt: &mut InterruptDescriptorTable, vector: u8, handler: extern "x86-interrupt" fn(&mut InterruptStackFrame)) {
        // Phase 2: Set up IRQ handlers for vectors 32-255
        println!("Setting IRQ handler for vector {}", vector);
    }
}
