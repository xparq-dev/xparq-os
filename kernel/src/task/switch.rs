// XPARQ OS - Phase 6: Assembly Context Switch
// Provides the low-level register swap

use core::arch::global_asm;

// This assembly routine expects:
// rdi = *mut u64 (pointer to current task's stack_ptr, where we save rsp)
// rsi = u64 (the new rsp to load)
global_asm!(
    r#"
    .global switch_context
    switch_context:
        // Save callee-saved registers of current task
        push rbp
        push rbx
        push r12
        push r13
        push r14
        push r15
        pushfq
        
        // Save rsp to the current task's struct
        mov [rdi], rsp
        
        // Switch to the new task's stack
        mov rsp, rsi
        
        // Restore callee-saved registers of the new task
        popfq
        pop r15
        pop r14
        pop r13
        pop r12
        pop rbx
        pop rbp
        
        ret
    "#
);

extern "C" {
    pub fn switch_context(current_stack_ptr_out: *mut u64, next_stack_ptr: u64);
    pub fn jump_to_ring3(rip: u64, rsp: u64) -> !;
}

global_asm!(
    r#"
    .global jump_to_ring3
    jump_to_ring3:
        // rdi = target RIP (user space instruction pointer)
        // rsi = target RSP (user space stack pointer)
        
        // Disable interrupts during the transition setup
        cli
        
        // Set up data segments to User Data (0x20 | RPL 3 = 0x23)
        mov ax, 0x23
        mov ds, ax
        mov es, ax
        mov fs, ax
        mov gs, ax
        
        // Push arguments for IRETQ
        push 0x23      // SS: User Data Segment
        push rsi       // RSP: User Stack Pointer
        push 0x202     // RFLAGS: Enable Interrupts (IF)
        push 0x2B      // CS: User Code64 Segment (0x28 | RPL 3 = 0x2B)
        push rdi       // RIP: User Instruction Pointer
        
        // Return to ring 3
        iretq
    "#
);
