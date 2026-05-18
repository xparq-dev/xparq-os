; Simple MBR Bootloader - loads kernel at 0x10000 and jumps
; No protected mode, no copy - just load and jump

[BITS 16]
[ORG 0x7C00]

start:
    ; Setup segments
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    
    ; Save DL (boot drive)
    mov [boot_drive], dl
    
    ; Print 'XPARQ'
    mov si, msg_xparq
    call print
    
    ; Reset disk - use DL from BIOS
    xor ah, ah
    mov dl, [boot_drive]
    int 0x13
    jc error_reset
    
    ; Load kernel using Extended LBA Read (INT 13h AH=42h)
    ; DAP (Disk Address Packet) setup
    mov word [dap_count], 16      ; Number of sectors (8KB)
    mov word [dap_buf_seg], 0x1000 ; Buffer segment
    mov word [dap_buf_off], 0x0000 ; Buffer offset
    mov dword [dap_lba], 1        ; Start LBA 1 (sector 2)
    
    mov ah, 0x42
    mov si, dap_structure
    mov dl, [boot_drive]
    int 0x13
    jc error_load
    
    ; Print 'OK' then jump
    mov si, msg_ok
    call print
    
    ; Jump to kernel at 0x1000:0x0000
    jmp 0x1000:0x0000

print:
    lodsb
    test al, al
    jz .done
    mov ah, 0x0E
    int 0x10
    jmp print
.done:
    ret

error_reset:
    mov si, msg_reset_err
    call print
    hlt
    jmp $

error_load:
    mov si, msg_load_err
    call print
    hlt
    jmp $

msg_xparq: db "[XPARQ OS] Booting on x86-64...", 13, 10, 0
msg_ok:    db "[XPARQ OS] Kernel initialized.", 13, 10, 0
msg_reset_err: db "Disk Reset Error", 0
msg_load_err:  db "Disk Load Error", 0
boot_drive: db 0

; DAP structure for INT 13h AH=42h (must be exactly 16 bytes)
dap_structure:
    db 0x10             ; [0] Size of packet (16 bytes)
    db 0                ; [1] Reserved
dap_count:    dw 16               ; [2-3] Number of sectors to transfer
dap_buf_off:  dw 0x0000           ; [4-5] Transfer buffer offset
dap_buf_seg:  dw 0x1000           ; [6-7] Transfer buffer segment
dap_lba:      dq 1                ; [8-15] Starting absolute sector number

; Boot signature
times 510-($-$$) db 0
dw 0xAA55
