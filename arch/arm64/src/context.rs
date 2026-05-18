//! ARM64 Context Switching - Phase 2: Dev Environment Setup
//! 
//! This module provides ARM64 context switching for XPARQ OS, including:
//! - Thread context save/restore
//! - Register state management
//! - Stack switching
//! - Exception frame handling
//! 
//! Context Size: 16 registers + FP + LR + SP + PC + PSTATE
//! Calling Convention: AAPCS64
//! Stack Alignment: 16-byte
/// Exception Frame: Saved on entry to kernel
//! 
//! Roadmap Phase: Phase 2 - Dev Environment Setup

/// Thread context structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ThreadContext {
    /// General purpose registers x0-x30
    pub x: [u64; 31],
    /// Stack pointer
    pub sp: u64,
    /// Program counter
    pub pc: u64,
    /// Processor state (PSTATE)
    pub pstate: u64,
    /// Floating point registers (Phase 2)
    pub v: [u128; 32],
    /// FP control register (Phase 2)
    pub fpsr: u32,
    /// FP status register (Phase 2)
    pub fpcr: u32,
}

impl ThreadContext {
    /// Create new context for thread start
    pub fn new(entry: usize, stack: usize, arg1: u64, arg2: u64) -> Self {
        let mut ctx = Self {
            x: [0; 31],
            sp: stack as u64,
            pc: entry as u64,
            pstate: 0, // Will be set to proper value
            v: [0; 32],
            fpsr: 0,
            fpcr: 0,
        };
        
        // Set up arguments for thread entry
        ctx.x[0] = arg1;
        ctx.x[1] = arg2;
        
        // Set PSTATE for kernel mode
        ctx.pstate = 0x3C5; // EL1h, IRQ/FIQ masked, little endian
        
        ctx
    }
    
    /// Get stack pointer
    pub fn stack_pointer(&self) -> usize {
        self.sp as usize
    }
    
    /// Set stack pointer
    pub fn set_stack_pointer(&mut self, sp: usize) {
        self.sp = sp as u64;
    }
    
    /// Get program counter
    pub fn program_counter(&self) -> usize {
        self.pc as usize
    }
    
    /// Set program counter
    pub fn set_program_counter(&mut self, pc: usize) {
        self.pc = pc as u64;
    }
}

/// Perform context switch between threads
/// 
/// This function saves the current thread context and restores the next thread context.
/// It's called from the scheduler when switching to a new thread.
/// 
/// # Arguments
/// * `prev_thread` - Thread being switched away from
/// * `next_thread` - Thread being switched to
pub fn switch(prev_thread: *mut crate::Thread, next_thread: *mut crate::Thread) {
    // Phase 1: Basic context switch
    // Phase 2: Full context save/restore with floating point
    
    println!("Context switch: Thread {} -> Thread {}", 
             unsafe { (*prev_thread).id }, 
             unsafe { (*next_thread).id });
    
    // Get context pointers
    let prev_ctx = unsafe { &mut (*prev_thread).context };
    let next_ctx = unsafe { &(*next_thread).context };
    
    // Perform assembly context switch
    unsafe {
        context_switch_asm(prev_ctx, next_ctx);
    }
}

/// Assembly context switch function
/// 
/// This function performs the actual register save/restore.
/// It saves the current context and restores the new context.
#[naked]
unsafe extern "C" fn context_switch_asm(prev_ctx: *mut ThreadContext, next_ctx: *const ThreadContext) {
    core::arch::asm!(
        // Save callee-saved registers
        "stp x19, x20, [x0, #16 * 8]",
        "stp x21, x22, [x0, #18 * 8]",
        "stp x23, x24, [x0, #20 * 8]",
        "stp x25, x26, [x0, #22 * 8]",
        "stp x27, x28, [x0, #24 * 8]",
        "stp x29, x30, [x0, #26 * 8]",
        
        // Save stack pointer and program counter
        "mov x3, sp",
        "str x3, [x0, #31 * 8]",
        "adr x3, 1f",
        "str x3, [x0, #32 * 8]",
        
        // Restore callee-saved registers
        "ldp x19, x20, [x1, #16 * 8]",
        "ldp x21, x22, [x1, #18 * 8]",
        "ldp x23, x24, [x1, #20 * 8]",
        "ldp x25, x26, [x1, #22 * 8]",
        "ldp x27, x28, [x1, #24 * 8]",
        "ldp x29, x30, [x1, #26 * 8]",
        
        // Restore stack pointer and program counter
        "ldr x3, [x1, #31 * 8]",
        "mov sp, x3",
        "ldr x3, [x1, #32 * 8]",
        "br x3",
        
        "1:",
        "ret",
        options(noreturn)
    );
}

/// Exception frame structure
/// 
/// This represents the register state saved when entering an exception.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ExceptionFrame {
    /// General purpose registers x0-x30
    pub x: [u64; 31],
    /// Stack pointer at exception entry
    pub sp: u64,
    /// Program counter at exception entry
    pub pc: u64,
    /// Processor state at exception entry
    pub pstate: u64,
    /// Exception syndrome
    pub esr: u64,
    /// Exception link register
    pub elr: u64,
}

impl ExceptionFrame {
    /// Create new exception frame
    pub fn new() -> Self {
        Self {
            x: [0; 31],
            sp: 0,
            pc: 0,
            pstate: 0,
            esr: 0,
            elr: 0,
        }
    }
    
    /// Get register value
    pub fn get_reg(&self, reg: u8) -> u64 {
        if reg < 31 {
            self.x[reg as usize]
        } else {
            0
        }
    }
    
    /// Set register value
    pub fn set_reg(&mut self, reg: u8, value: u64) {
        if reg < 31 {
            self.x[reg as usize] = value;
        }
    }
}

/// Handle exception entry
/// 
/// This function is called from assembly exception handlers to save the context.
pub fn exception_entry(frame: &mut ExceptionFrame) {
    println!("Exception entry: PC=0x{:x}, PSTATE=0x{:x}, ESR=0x{:x}", 
             frame.pc, frame.pstate, frame.esr);
    
    // Phase 2: Handle specific exception types
    // Phase 3: Full exception processing with signal handling
}

/// Handle exception return
/// 
/// This function is called before returning from an exception.
pub fn exception_return(frame: &ExceptionFrame) {
    println!("Exception return: PC=0x{:x}, PSTATE=0x{:x}", frame.pc, frame.pstate);
    
    // Phase 2: Prepare for exception return
    // Phase 3: Check for signal delivery, etc.
}

/// Initialize context switching
pub fn init() {
    println!("Initializing ARM64 context switching...");
    
    // Phase 2: Initialize floating point state
    // Phase 3: Initialize performance counters
    
    println!("ARM64 context switching initialized");
}

/// Thread context extension for ARM64
pub trait ThreadContextExt {
    /// Get context pointer
    fn context_ptr(&mut self) -> *mut ThreadContext;
    
    /// Set up initial context
    fn setup_initial_context(&mut self, entry: usize, stack: usize, arg1: u64, arg2: u64);
}

impl ThreadContextExt for crate::Thread {
    fn context_ptr(&mut self) -> *mut ThreadContext {
        &mut self.context
    }
    
    fn setup_initial_context(&mut self, entry: usize, stack: usize, arg1: u64, arg2: u64) {
        self.context = ThreadContext::new(entry, stack, arg1, arg2);
    }
}
