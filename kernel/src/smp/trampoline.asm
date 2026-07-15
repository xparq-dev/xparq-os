.code16

.global TRAMPOLINE_START
.global TRAMPOLINE_END
.global AP_STACK_PTR
.global AP_PAGE_TABLE
.global AP_CODE_SEG
.global AP_DATA_SEG
.global AP_READY_FLAG

TRAMPOLINE_START:
    cli
    cld

    // The AP starts at 0x0000:0x8000 (CS=0x800, IP=0x0000) or similar
    // Disable NMI (already done typically, but just in case)
    
    // Load a temporary GDT for 32-bit protected mode
    // We compute the physical address of the GDT
    xor ax, ax
    mov ds, ax
    
    // Enable A20 line (fast A20)
    in al, 0x92
    or al, 2
    out 0x92, al

    // Load Temporary GDT
    .equ OFFSET_GDT, TRAMPOLINE_GDT_PTR - TRAMPOLINE_START
    lgdt [OFFSET_GDT + 0x8000]

    // Enter 32-bit Protected Mode
    mov eax, cr0
    or al, 1
    mov cr0, eax

    // Far jump to flush instruction prefetch queue and load CS
    .equ OFFSET_PM, protected_mode_entry - TRAMPOLINE_START
    .att_syntax prefix
    ljmp $0x08, $OFFSET_PM + 0x8000
    .intel_syntax noprefix

.code32
protected_mode_entry:
    // Load data segments
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    // Enable PAE
    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax

    // Load Page Table (CR3) provided by BSP
    .equ OFFSET_PT, AP_PAGE_TABLE - TRAMPOLINE_START
    mov eax, dword ptr [OFFSET_PT + 0x8000]
    mov cr3, eax

    // Enable Long Mode in EFER
    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr

    // Enable Paging to enter Long Mode
    mov eax, cr0
    or eax, (1 << 31)
    mov cr0, eax

    // Jump to 64-bit code using the temporary GDT 64-bit code segment
    .equ OFFSET_LM, long_mode_entry - TRAMPOLINE_START
    .att_syntax prefix
    ljmp $0x18, $OFFSET_LM + 0x8000
    .intel_syntax noprefix

.code64
long_mode_entry:
    // Set data segments to 0 in 64-bit mode (or kernel data segment)
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    // Load Stack Pointer
    .equ OFFSET_SP, AP_STACK_PTR - TRAMPOLINE_START
    mov rsp, qword ptr [OFFSET_SP + 0x8000]

    // Jump to the rust ap_entry function
    // ap_entry is an absolute 64-bit address, we need to load it into a register first
    movabs rax, offset ap_entry
    call rax

hang:
    hlt
    jmp hang

.align 8
TRAMPOLINE_GDT:
    .quad 0x0000000000000000 // Null Descriptor
    .quad 0x00CF9A000000FFFF // 32-bit Code Descriptor (0x08)
    .quad 0x00CF92000000FFFF // 32-bit Data Descriptor (0x10)
    .quad 0x00209A0000000000 // 64-bit Code Descriptor (0x18)
    .quad 0x0000920000000000 // 64-bit Data Descriptor (0x20)
TRAMPOLINE_GDT_PTR:
    .word 5 * 8 - 1
    .long TRAMPOLINE_GDT - TRAMPOLINE_START + 0x8000

.align 8
AP_STACK_PTR:  .quad 0
AP_PAGE_TABLE: .quad 0
AP_CODE_SEG:   .quad 0
AP_DATA_SEG:   .quad 0
AP_READY_FLAG: .long 0

TRAMPOLINE_END:

