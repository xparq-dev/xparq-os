; XPARQ OS - x86-64 Bootloader
; Real → Protected → Long, VGA only, no VBE
; Debug on VGA text buffer at 0xB8000

[BITS 16]
[ORG 0x7C00]

%macro vga_putc 2
    mov di, 0xB8000 + (80 * 2 * %1) + (%2 * 2)
    mov al, %2 + '0'
    mov ah, 0x1F
    stosw
%endmacro

start:
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7B00
    mov [boot_drive], dl

    ; Write "X" at (0,0)
    mov ax, 0xB8000
    mov es, ax
    mov byte [es:0], 'X'
    mov byte [es:1], 0x1F

    ; Reset disk
    xor ax, ax
    mov dl, [boot_drive]
    int 0x13
    jc error
    mov byte [es:2], 'R'
    mov byte [es:3], 0x2F

    ; Load kernel
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
    mov ax, 0xB8000
    mov es, ax
    mov byte [es:4], 'K'
    mov byte [es:5], 0x3F

    ; A20
    mov ax, 0x2401
    int 0x15
    cli
    mov ax, 0xB8000
    mov es, ax
    mov byte [es:6], 'A'
    mov byte [es:7], 0x4F

    ; GDT
    lgdt [gdt_desc]
    mov ax, 0xB8000
    mov es, ax
    mov byte [es:8], 'G'
    mov byte [es:9], 0x5F

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
    mov edi, 0xB8000
    mov byte [edi+10], '3'
    mov byte [edi+11], 0x6F

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
    mov edi, 0xB8000
    mov byte [edi+12], '6'
    mov byte [edi+13], 0x70

    jmp 0x10000

error:
    mov ax, 0xB8000
    mov es, ax
    mov byte [es:0], 'E'
    mov byte [es:1], 0x4F
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
