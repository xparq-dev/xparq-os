; XPARQ OS - x86-64 Bootloader COMPLETE MINIMAL, SHORT
[BITS 16]
[ORG 0x7C00]

jmp short start_after_bpb
nop
db 'XPARQ   '
dw 512,1,1
db 2,224
dw 2880,0xF0,9,18,2
dd 0,0
db 0,0,0x29
dd 0x12345678
db 'XPARQ BOOT '
db 'FAT12   '

start_after_bpb:
    xor ax,ax
    mov ds,ax
    mov es,ax
    mov ss,ax
    mov sp,0x7B00
    mov [boot_drive],dl

    ; Load kernel (8 chunks of 120 sectors = 480 KB)
    mov cx, 8
.load_loop:
    mov ah, 0x42
    mov si, dap_structure
    mov dl, [boot_drive]
    int 0x13
    jc error
    
    ; Increment LBA by 120
    add dword [dap_lba], 120
    adc dword [dap_lba+4], 0
    
    ; Increment buffer segment by 120 sectors (120 * 512 / 16 = 3840 = 0x0F00)
    add word [dap_buf_seg], 0x0F00
    
    loop .load_loop

    ; --- Enable VESA Graphics Mode ---
    mov ax, 0x4F01
    mov cx, 0x0144
    mov di, 0x7E00 ; Save VbeModeInfo to 0x7E00 for the kernel
    int 0x10
    
    mov ax, 0x4F02
    mov bx, 0x4144
    int 0x10
    ; ---------------------------------
    ; Fast A20
    in al,0x92
    or al,2
    out 0x92,al
    cli

    ; GDT load
    lgdt [gdt_desc]

    ; Enter protected mode
    mov eax,cr0
    or eax,1
    mov cr0,eax
    jmp 0x08:pmode

[BITS 32]
pmode:
    mov ax,0x10
    mov ds,ax
    mov es,ax
    mov ss,ax
    mov fs,ax
    mov gs,ax
    mov esp,0x7B00

    ; Page tables at 0x70000
    mov edi,0x70000
    xor eax,eax
    mov ecx,0x3000/4 ; Zero 3 pages (PML4, PDPT, PD)
    rep stosd
    mov dword [0x70000],0x71000 | 3
    mov dword [0x71000],0x72000 | 3
    mov edi,0x72000
    mov eax,0x83
    mov ecx,512
.pde:
    mov [edi],eax
    mov dword [edi+4],0
    add eax,0x200000
    add edi,8
    loop .pde

    ; PAE on
    mov eax,cr4
    or eax,(1<<5)
    mov cr4,eax

    ; CR3
    mov eax,0x70000
    mov cr3,eax

    ; LME on
    mov ecx,0xC0000080
    rdmsr
    or eax,(1<<8)
    wrmsr

    ; Paging on
    mov eax,cr0
    or eax,0x80000000
    mov cr0,eax

    jmp 0x18:lmode

[BITS 64]
lmode:
    mov ax,0x10
    mov ds,ax
    mov es,ax
    mov ss,ax
    mov fs,ax
    mov gs,ax
    mov rsp,0x80000

    ; Copy kernel from 0x10000 to 0x100000 (480 KB)
    mov rsi, 0x10000
    mov rdi, 0x100000
    mov rcx, 61440 ; 491520 bytes / 8
    rep movsq

    jmp 0x100000

[BITS 16]
error:
    mov ax,0xB800
    mov es,ax
    mov byte [es:0],'E'
    mov byte [es:1],0x4F
    cli
    hlt

; Compact GDT
gdt_start:
    dq 0
    dq 0x00CF9A000000FFFF
    dq 0x00CF92000000FFFF
    dq 0x00209A000000FFFF
gdt_end:

gdt_desc:
    dw gdt_end - gdt_start - 1
    dd gdt_start

dap_structure:
    db 0x10             ; Size of packet (16 bytes)
    db 0                ; Always 0
    dw 120              ; Number of sectors to read
dap_buf_off: 
    dw 0                ; Target offset
dap_buf_seg: 
    dw 0x1000           ; Target segment
dap_lba: 
    dq 1                ; LBA start (Sector 1 = kernel start)

boot_drive: db 0
times 510 - ($ - $$) db 0
dw 0xAA55
