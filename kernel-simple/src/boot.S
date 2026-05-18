/* XPARQ OS - Phase 02: ARM64 Boot Entry Point */
/* Minimal assembly entry point for QEMU virt machine */

.section .text.entry
.global _start
.global boot_stack_top

_start:
    /* Setup stack pointer */
    ldr x0, =boot_stack_top
    mov sp, x0
    
    /* Call Rust entry point */
    bl kernel_main
    
    /* Infinite loop if kernel returns */
1:  wfi
    b 1b

/* Stack space */
.section .bss
.align 16
boot_stack_bottom:
    .skip 4096
boot_stack_top:
