; XPARQ OS - x86-64 Bootloader
; Real → Protected → Long, VGA only, no VBE
; Debug on VGA text buffer at 0xB8000

[BITS 16]
[ORG 0x7C00]

; Dummy BIOS Parameter Block (BPB) to keep BIOS happy
jmp short start_after_bpb
nop

; OEM ID (8 bytes)
db 'XPARQ   '

; BPB for FAT12/16
dw 512             ; Bytes per sector
db 1               ; Sectors per cluster
dw 1               ; Reserved sectors
db 2               ; Number of FATs
dw 224             ; Root entries
dw 2880            ; Total sectors (1.44MB floppy)
db 0xF0            ; Media descriptor
dw 9               ; Sectors per FAT
dw 18              ; Sectors per track
dw 2               ; Heads per cylinder
dd 0               ; Hidden sectors
dd 0               ; Large total sectors (0 for FAT12)

; EBPB (Extended Boot Record)
db 0x00            ; Drive number
db 0x00            ; Reserved
db 0x29            ; Extended boot signature
dd 0x12345678      ; Volume ID (random)
db 'XPARQ BOOT '   ; Volume label (11 bytes)
db 'FAT12   '      ; File system type (8 bytes)

start_after_bpb:
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7B00
    mov [boot_drive], dl

    ; Clear VGA text buffer
    mov ax, 0xB800
    mov es, ax
    mov di, 0
    mov cx, 80*25
    mov ax, 0x1F20 ; Space with blue bg
    rep stosw

    ; Write "X" at (0,0)
    mov byte [es:0], 'X'
    mov byte [es:1], 0x1F

    ; Reset disk
    xor ax, ax
    mov dl, [boot_drive]
    int 0x13
    jc error
    mov byte [es:2], 'R'
    mov byte [es:3], 0x2F

    ; Load kernel (80 sectors = 40KB from sector 1)
    mov ax, 0x1000
    mov es, ax
    xor bx, bx
    mov ah, 0x02
    mov al, 80
    mov ch, 0
    mov cl, 1
    mov dh, 0
    mov dl, [boot_drive]
    int 0x13
    jc error
    mov ax, 0xB800
    mov es, ax
    mov byte [es:4], 'K'
    mov byte [es:5], 0x3F

    ; A20
    mov ax, 0x2401
    int 0x15
    cli
    mov ax, 0xB800
    mov es, ax
    mov byte [es:6], 'A'
    mov byte [es:7], 0x4F

    ; GDT
    lgdt [gdt_desc]
    mov ax, 0xB800
    mov es, ax
    mov byte [es:8], 'G'
    mov byte [es:9], 0x5F

    ; Write "J" before jump
    mov byte [es:10], 'J'
    mov byte [es:11], 0x6F

    ; Protected mode
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp short $+2  ; flush the prefetch queue
    jmp 0x0008:pmode

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
    mov ax, 0xB800
    mov es, ax
    mov byte [es:0], 'E'
    mov byte [es:1], 0x4F
    cli
    hlt

; GDT
gdt_start:
    ; Null descriptor (0x00)
    dq 0
    ; Code segment descriptor (0x08): 32-bit ring0, code read/exec, 4K granularity, limit 0xFFFFF
    dw 0xFFFF ; limit (0-15)
    dw 0x0000 ; base (0-15)
    db 0x00 ; base (16-23)
    db 0x9A ; access (present, ring0, code, read/exec)
    db 0xCF ; flags (granularity, 32-bit) + limit (16-19)
    db 0x00 ; base (24-31)
    ; Data segment descriptor (0x10): 32-bit ring0, data read/write, 4K granularity, limit 0xFFFFF
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 0x92
    db 0xCF
    db 0x00
    ; Code64 segment (0x18): 64-bit ring0, code read/exec
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 0x9A
    db 0x20 ; 64-bit flag only
    db 0x00
gdt_end:
gdt_desc:
    dw gdt_end - gdt_start - 1
    dd gdt_start

boot_drive: db 0

times 510 - ($ - $$) db 0
dw 0xAA55
