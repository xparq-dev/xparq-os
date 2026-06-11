; XPARQ OS - x86-64 Bootloader with VBE
; Real mode → Protected mode → Long mode → kernel
; Sets VBE 1024x768x32 framebuffer, or falls back to VGA text

[BITS 16]
[ORG 0x7C00]

start:
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    mov [boot_drive], dl          ; save drive
    
    ; Reset disk
    xor ax, ax
    mov dl, [boot_drive]
    int 0x13
    jc error
    
    ; Load kernel (64 sectors = 32768 bytes)
    mov ax, 0x1000        ; ES = 0x1000
    mov es, ax
    xor bx, bx            ; offset 0
    mov ah, 0x02          ; read sectors
    mov al, 64            ; sector count
    mov ch, 0             ; cylinder 0
    mov cl, 2             ; sector 2
    mov dh, 0             ; head 0
    mov dl, [boot_drive]
    int 0x13
    jc error
    
    ; Try to set VBE mode
    mov ax, 0x4F02
    mov bx, 0x4118        ; 1024x768x32, linear framebuffer
    int 0x10
    cmp ax, 0x004F
    jne .vbe_failed
    
    ; Get VBE mode info
    mov ax, 0x4F01
    mov cx, 0x118
    mov di, 0x7E00
    int 0x10
    cmp ax, 0x004F
    jne .vbe_failed
    
    jmp .vbe_ok
.vbe_failed:
    ; Clear mode info struct
    mov di, 0x7E00
    mov cx, 256
    xor ax, ax
    rep stosb
.vbe_ok:
    
    ; A20 via BIOS
    mov ax, 0x2401
    int 0x15
    cli                         ; disable interrupts
    
    ; Load GDT
    lgdt [gdt_desc]
    
    ; CR0.PE = 1
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp 0x08:pmode

[BITS 32]
pmode:
    ; Set data segs
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    
    ; Set up identity-mapped 1GB page tables at 0x7000
    mov edi, 0x7000
    xor eax, eax
    mov ecx, 0x1000 / 4        ; 1KB → 1024 dwords
    rep stosd
    
    ; PML4[0] → PDP at 0x7100
    mov dword [0x7000], 0x7100 | 3
    
    ; PDP at 0x7100: entry 0 → PD at 0x7200
    mov edi, 0x7100
    xor eax, eax
    mov ecx, 0x1000 / 4
    rep stosd
    mov dword [0x7100], 0x7200 | 3
    
    ; PD at 0x7200: 512 entries of 2MB pages
    mov edi, 0x7200
    mov eax, 0x83         ; Present+Writable+PageSize=1
    mov ecx, 512
.setpd:
    mov [edi], eax
    mov dword [edi + 4], 0
    add eax, 0x200000
    add edi, 8
    loop .setpd
    
    ; Enable PAE
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax
    
    ; Set CR3
    mov eax, 0x7000
    mov cr3, eax
    
    ; Enable Long Mode via EFER
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr
    
    ; Enable paging
    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax
    
    ; Jump to 64-bit
    jmp 0x18:lmode

[BITS 64]
lmode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    
    ; UART debug: 'B' (booted)
    mov dx, 0x3FD
.wait:
    in al, dx
    test al, 0x20
    jz .wait
    mov al, 'B'
    mov dx, 0x3F8
    out dx, al
    
    jmp 0x10000           ; jump to kernel

error:
    mov si, msg_err
    call print
    mov al, 'E'
    mov dx, 0x3F8
    out dx, al
    cli
    hlt

; -------------------------------------
;  Utilities (16-bit)
; -------------------------------------
print:
    lodsb
    test al, al
    jz .done
    mov ah, 0x0E
    int 0x10
    jmp print
.done:
    ret

; -------------------------------------
;  Data
; -------------------------------------
msg_err   db 'E', 0

; GDT
gdt_start:
    dq 0                    ; null descriptor
    
    ; code32
    dw 0xFFFF
    dw 0
    db 0
    db 0x9A
    db 0xCF
    db 0
    
    ; data32
    dw 0xFFFF
    dw 0
    db 0
    db 0x92
    db 0xCF
    db 0
    
    ; code64
    dw 0
    dw 0
    db 0
    db 0x9A
    db 0x20
    db 0
gdt_end:

gdt_desc:
    dw gdt_end - gdt_start - 1
    dd gdt_start

boot_drive: db 0

; Boot signature
times 510 - ($ - $$) db 0
dw 0xAA55
