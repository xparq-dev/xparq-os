; XPARQ OS - x86-64 Bootloader (SIMPLE)
; Real → Protected → Long, no VBE, VGA text only
; Debug serial output at each step

[BITS 16]
[ORG 0x7C00]

; Macro for serial output 0x3F8
%macro putc_serial 1
    mov dx, 0x3FD
%%wait:
    in al, dx
    test al, 0x20
    jz %%wait
    mov al, %1
    mov dx, 0x3F8
    out dx, al
%endmacro

start:
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    mov [boot_drive], dl

    putc_serial 'S'

    ; Reset disk
    xor ax, ax
    mov dl, [boot_drive]
    int 0x13
    jc error
    putc_serial 'R'

    ; Load kernel (64 sectors = 32KB from sector 2)
    mov ax, 0x1000
    mov es, ax
    xor bx, bx
    mov ah, 0x02
    mov al, 64
    mov ch, 0
    mov cl, 2
    mov dh, 0
    mov dl, [boot_drive]
    int 0x13
    jc error
    putc_serial 'K'

    ; A20
    mov ax, 0x2401
    int 0x15
    cli

    ; GDT
    lgdt [gdt_desc]

    ; Protected mode
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp 0x08:pmode

[BITS 32]
pmode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; Page tables at 0x7000
    mov edi, 0x7000
    xor eax, eax
    mov ecx, 0x1000 / 4
    rep stosd
    mov dword [0x7000], 0x7100 | 3

    mov edi, 0x7100
    xor eax, eax
    mov ecx, 0x1000 / 4
    rep stosd
    mov dword [0x7100], 0x7200 | 3

    mov edi, 0x7200
    mov eax, 0x83
    mov ecx, 512
.setpd:
    mov [edi], eax
    mov dword [edi + 4], 0
    add eax, 0x200000
    add edi, 8
    loop .setpd

    ; Enable PAE
    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax

    ; CR3
    mov eax, 0x7000
    mov cr3, eax

    ; Long mode enable
    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr

    ; Paging enable
    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax

    jmp 0x18:lmode

[BITS 64]
lmode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    jmp 0x10000

error:
    putc_serial 'E'
    cli
    hlt

; GDT
gdt_start:
    dq 0
    dw 0xFFFF, 0, 0x9A, 0xCF, 0 ; code32
    dw 0xFFFF, 0, 0x92, 0xCF, 0 ; data32
    dw 0, 0, 0x9A, 0x20, 0     ; code64
gdt_end:
gdt_desc:
    dw gdt_end - gdt_start - 1
    dd gdt_start

boot_drive: db 0

times 510 - ($ - $$) db 0
dw 0xAA55
