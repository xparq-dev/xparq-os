; XPARQ OS - Compact x86-64 Bootloader
; Real mode → Protected mode → Long mode → kernel
; Fits in 512 bytes. Loads kernel at 0x10000.

[BITS 16]
[ORG 0x7C00]

start:
    xor ax,ax
    mov ds,ax
    mov es,ax
    mov ss,ax
    mov sp,0x7C00
    mov [boot_drive],dl          ; save drive
    
    ; Reset disk
    xor ax,ax
    mov dl,[boot_drive]
    int 0x13
    jc error
    ; Load kernel using CHS (16 sectors, cylinder 0, head 0, sector 2)
    mov ax,0x1000        ; ES = 0x1000
    mov es,ax
    xor bx,bx            ; offset 0
    mov ah,0x02          ; read sectors
    mov al,4             ; sector count = 4 (2048 bytes)
    mov ch,0             ; cylinder 0
    mov cl,2             ; sector 2 (1-indexed)
    mov dh,0             ; head 0
    mov dl,[boot_drive]
    int 0x13
    jc error



    ; A20 via BIOS
    mov ax,0x2401
    int 0x15
    ; ignore error
    cli                         ; disable interrupts before mode switch

    ; Load GDT
    lgdt [gdt_desc]

    ; CR0.PE = 1
    mov eax,cr0
    or  eax,1
    mov cr0,eax
    jmp 0x08:pmode               ; 0x08 = code32

[BITS 32]
pmode:
    ; Set data segs
    mov ax,0x10
    mov ds,ax
    mov es,ax
    mov ss,ax

    ; Set up identity-mapped 1GB page tables at 0x7000
    ; PML4 at 0x7000
    mov edi,0x7000
    xor eax,eax
    mov ecx,0x400        ; 0x1000/4 = 1024? Actually we need zero 4KB => 1024 dwords = 0x400
    rep stosd

    ; PML4[0] → PDP at 0x7100
    mov dword [0x7000],0x7100|3

    ; PDP at 0x7100: entry 0 → PD at 0x7200
    mov edi,0x7100
    xor eax,eax
    mov ecx,0x400
    rep stosd
    mov dword [0x7100],0x7200|3

    ; PD at 0x7200: 512 entries of 2MB pages, cover 1GB
    mov edi,0x7200
    mov eax,0x83         ; Present+Writable+PageSize=1 (2MB)
    mov ecx,512
.setpd:
    mov [edi],eax
    mov dword [edi+4],0
    add eax,0x200000
    add edi,8
    loop .setpd

    ; Enable PAE
    mov eax,cr4
    or  eax,1<<5
    mov cr4,eax

    ; Set CR3
    mov eax,0x7000
    mov cr3,eax

    ; Enable Long Mode via EFER
    mov ecx,0xC0000080
    rdmsr
    or  eax,1<<8
    wrmsr

    ; Enable paging (CR0.PG)
    mov eax, cr0
    or  eax, 0x80000000
    mov cr0, eax

    ; Debug: print 'P' (paging enabled)
    mov bl, 'P'
    call uart_putc

    ; Jump to 64-bit code segment
    jmp 0x18:lmode

[BITS 64]
lmode:
    mov ax, 0x10          ; use data32 descriptor (valid in long mode)
    mov ds, ax
    mov es, ax
    mov ss, ax
    ; UART debug: 'J'
    mov dx, 0x3FD
.waitJ:
    in al, dx
    test al, 0x20
    jz .waitJ
    mov al, 'J'
    mov dx, 0x3F8
    out dx, al
    jmp 0x10000           ; jump to kernel entry              ; kernel entry

error:
    mov si,msg_err
    call print
    mov al, 'E'
    mov dx, 0x3F8
    out dx, al
    cli
    hlt

; -------------------------------------
;  Utilities (16-bit)
; -------------------------------------
print:                      ; SI = NUL-terminated string
    lodsb
    test al,al
    jz .done
    mov ah,0x0E
    int 0x10
    jmp print
.done:
    ret

; UART putc (char in BL)
uart_putc:
    push ax
    mov dx, 0x3FD          ; LSR
.wait:
    in al, dx
    test al, 0x20
    jz .wait
    pop ax                 ; char in al
    mov dx, 0x3F8         ; THR
    out dx, al
    ret

; -------------------------------------
;  Data (must be before boot sig)
; -------------------------------------
msg_err   db 'E',0

; GDT (null, code32, data32, code64)
gdt_start:
    dq 0                    ; null descriptor (8 bytes)

    ; code32: base=0 limit=4GB exec/read, gran=1, 32-bit
    dw 0xFFFF
    dw 0
    db 0
    db 0x9A
    db 0xCF
    db 0

    ; data32: base=0 limit=4GB read/write, gran=1
    dw 0xFFFF
    dw 0
    db 0
    db 0x92
    db 0xCF
    db 0

    ; code64: base=0 exec/read, L=1 (64-bit)
    dw 0
    dw 0
    db 0
    db 0x9A
    db 0x20
    db 0
gdt_end:

gdt_desc:
    dw gdt_end - gdt_start -1
    dd gdt_start

dap:
    db 16
    db 0
dap_cnt  dw 16
dap_off  dw 0
dap_seg  dw 0x1000
dap_lba  dq 1

boot_drive: db 0

; Boot signature
times 510-($-$$) db 0
dw 0xAA55
