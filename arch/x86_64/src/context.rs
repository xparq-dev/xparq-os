//! x86-64 Context Switching - Phase 2: Dev Environment Setup
//! 
//! This module provides x86-64 context switching for XPARQ OS, including:
//! - Thread context save/restore
//! - Register state management
//! - Stack switching
//! - Exception frame handling
//! - FPU/SIMD state management (Phase 2)
//! 
//! Context Size: 16 registers + RSP + RIP + RFLAGS + FPU state
//! Calling Convention: System V AMD64 ABI
//! Stack Alignment: 16-byte
/// Exception Frame: Saved on entry to kernel
//! 
//! Roadmap Phase: Phase 2 - Dev Environment Setup

/// Thread context structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ThreadContext {
    /// General purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    /// Instruction pointer
    pub rip: u64,
    /// Flags register
    pub rflags: u64,
    /// FPU state (Phase 2)
    pub fpu_state: FpuState,
}

/// FPU state structure (simplified for Phase 1)
#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct FpuState {
    pub data: [u8; 512], // FXSAVE area
}

impl ThreadContext {
    /// Create new context for thread start
    pub fn new(entry: usize, stack: usize, arg1: u64, arg2: u64) -> Self {
        let mut ctx = Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: arg2,
            rdi: arg1,
            rbp: 0,
            rsp: stack as u64,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: entry as u64,
            rflags: 0x202, // IF flag set, interrupts enabled
            fpu_state: FpuState { data: [0; 512] },
        };
        
        // Initialize FPU state
        unsafe {
            core::arch::asm!("fxsave {}", in(reg) &mut ctx.fpu_state);
        }
        
        ctx
    }
    
    /// Get stack pointer
    pub fn stack_pointer(&self) -> usize {
        self.rsp as usize
    }
    
    /// Set stack pointer
    pub fn set_stack_pointer(&mut self, sp: usize) {
        self.rsp = sp as u64;
    }
    
    /// Get program counter
    pub fn program_counter(&self) -> usize {
        self.rip as usize
    }
    
    /// Set program counter
    pub fn set_program_counter(&mut self, pc: usize) {
        self.rip = pc as u64;
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
    // Phase 2: Full context save/restore with FPU
    
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
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        
        // Save stack pointer
        "mov [rdi + 40], rsp", // Save RSP to context
        
        // Save instruction pointer
        "lea rax, [rip + 1f]",
        "mov [rdi + 136], rax", // Save RIP to context
        
        // Save flags
        "pushfq",
        "pop rax",
        "mov [rdi + 144], rax", // Save RFLAGS to context
        
        // Restore callee-saved registers
        "mov rax, [rsi + 144]", // Restore RFLAGS
        "push rax",
        "popfq",
        
        "mov rax, [rsi + 136]", // Restore RIP
        "push rax",
        
        "mov rsp, [rsi + 40]", // Restore RSP
        
        "pop rax", // Discard saved RIP
        
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        
        "ret",
        
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
    /// General purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    /// Instruction pointer
    pub rip: u64,
    /// Flags register
    pub rflags: u64,
    /// Error code (if present)
    pub error_code: u64,
    /// Exception number
    pub exception_number: u64,
}

impl ExceptionFrame {
    /// Create new exception frame
    pub fn new() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            rflags: 0,
            error_code: 0,
            exception_number: 0,
        }
    }
    
    /// Get register value
    pub fn get_reg(&self, reg: u8) -> u64 {
        match reg {
            0 => self.rax,
            1 => self.rcx,
            2 => self.rdx,
            3 => self.rbx,
            4 => self.rsp,
            5 => self.rbp,
            6 => self.rsi,
            7 => self.rdi,
            8 => self.r8,
            9 => self.r9,
            10 => self.r10,
            11 => self.r11,
            12 => self.r12,
            13 => self.r13,
            14 => self.r14,
            15 => self.r15,
            _ => 0,
        }
    }
    
    /// Set register value
    pub fn set_reg(&mut self, reg: u8, value: u64) {
        match reg {
            0 => self.rax = value,
            1 => self.rcx = value,
            2 => self.rdx = value,
            3 => self.rbx = value,
            4 => self.rsp = value,
            5 => self.rbp = value,
            6 => self.rsi = value,
            7 => self.rdi = value,
            8 => self.r8 = value,
            9 => self.r9 = value,
            10 => self.r10 = value,
            11 => self.r11 = value,
            12 => self.r12 = value,
            13 => self.r13 = value,
            14 => self.r14 = value,
            15 => self.r15 = value,
            _ => {}
        }
    }
}

/// Handle exception entry
/// 
/// This function is called from assembly exception handlers to save the context.
pub fn exception_entry(frame: &mut ExceptionFrame) {
    println!("Exception entry: RIP=0x{:x}, RFLAGS=0x{:x}, Exception={}", 
             frame.rip, frame.rflags, frame.exception_number);
    
    // Phase 2: Handle specific exception types
    // Phase 3: Full exception processing with signal handling
}

/// Handle exception return
/// 
/// This function is called before returning from an exception.
pub fn exception_return(frame: &ExceptionFrame) {
    println!("Exception return: RIP=0x{:x}, RFLAGS=0x{:x}", frame.rip, frame.rflags);
    
    // Phase 2: Prepare for exception return
    // Phase 3: Check for signal delivery, etc.
}

/// Initialize context switching
pub fn init() {
    println!("Initializing x86-64 context switching...");
    
    // Phase 2: Initialize FPU state
    // Phase 3: Initialize performance counters
    
    println!("x86-64 context switching initialized");
}

/// Thread context extension for x86-64
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

/// FPU state management
pub mod fpu {
    use super::FpuState;
    
    /// Initialize FPU
    pub fn init() {
        println!("Initializing FPU...");
        
        // Enable FPU and SSE
        let mut cr0 = super::sysreg::read_cr0();
        cr0 &= !x86_64::registers::control::Cr0Flags::EM; // Clear EM bit
        cr0 &= !x86_64::registers::control::Cr0Flags::TS; // Clear TS bit
        super::sysreg::write_cr0(cr0);
        
        let mut cr4 = super::sysreg::read_cr4();
        cr4 |= x86_64::registers::control::Cr4Flags::OSFXSR; // Set OSFXSR
        cr4 |= x86_64::registers::control::Cr4Flags::OSXSAVE; // Set OSXSAVE
        super::sysreg::write_cr4(cr4);
        
        println!("FPU initialized");
    }
    
    /// Save FPU state
    pub fn save_state(state: &mut FpuState) {
        unsafe {
            core::arch::asm!("fxsave [{}]", in(reg) state);
        }
    }
    
    /// Restore FPU state
    pub fn restore_state(state: &FpuState) {
        unsafe {
            core::arch::asm!("fxrstor [{}]", in(reg) state);
        }
    }
    
    /// Initialize FPU state
    pub fn init_state(state: &mut FpuState) {
        unsafe {
            core::ptr::write_bytes(state, 0, 512);
        }
    }
}
