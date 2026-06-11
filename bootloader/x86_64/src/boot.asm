; XPARQ OS - x86-64 Bootloader COMPLETE MINIMAL
[BITS 16]
[ORG 0x7C00]

jmp short start_after_bpb
nop
db 'XPARQ   '
dw 512, 1, 1
db 2, 224
dw 2880, 0xF0, 9, 18, 2
dd 0,0
db 0,0,0x29
dd 0x12345678
db 'XPARQ BOOT '
db 'FAT12   '

start_after_bpb:
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7B00
    mov [boot_drive], dl

    ; Clear VGA buffer
    mov ax, 0xB800
    mov es, ax
    mov di, 0
    mov cx, 80*25
    mov ax, 0x1F20
    rep stosw

    ; X
    mov byte [es:0], 'X'
    mov byte [es:1], 0x1F

    ; Reset disk R
    xor ax, ax
    mov dl, [boot_drive]
    int 0x13
    jc error
    mov byte [es:2], 'R'
    mov byte [es:3], 0x2F

    ; Load kernel to 0x10000
    mov ax, 0x1000
    mov es, ax
    xor bx, bx
    mov ah, 0x02
    mov al, 80
    mov ch,0
    mov cl,1
    mov dh,0
    mov dl, [boot_drive]
    int 0x13
    jc error
    mov ax,0xB800
    mov es, ax
    mov byte [es:4], 'K'
    mov byte [es:5], 0x3F

    ; Fast A20
    in al,0x92
    or al,2
    out 0x92,al
    cli
    mov byte [es:6], 'A'
    mov byte [es:7],0x4F

    ; Load GDT
    lgdt [gdt_desc]
    mov byte [es:8], 'G'
    mov byte [es:9],0x5F

    ; Enter protected mode
    mov eax, cr0
    or eax,1
    mov cr0,eax
    jmp 0x08:0x7C00 + pmode - start_after_bpb

[BITS 32]
pmode:
    mov ax,0x10
    mov ds,ax
    mov es,ax
    mov ss,ax
    mov fs,ax
    mov gs,ax
    mov esp,0x7B00

    ; Write 3
    mov edi,0xB8000
    mov byte [edi+10], '3'
    mov byte [edi+11],0x6F

    ; Set up page tables at 0x7000
    mov edi,0x7000
    xor eax,eax
    mov ecx,0x1000 /4
    rep stosd

    ; PML4T at 0x7000 -> PDPT at 0x7100
    mov dword [0x7000],0x7100 |3

    ; PDPT at 0x7100 -> PDT at 0x7200
    mov dword [0x7100],0x7200 |3

    ; PDT at 0x7200 - 512 2MB entries, identity map 0-1GB
    mov edi,0x7200
    mov eax,0x83
    mov ecx,512
.pde:
    mov [edi],eax
    mov dword [edi+4],0
    add eax,0x200000
    add edi,8
    loop .pde

    ; Enable PAE (bit 5 of CR4)
    mov eax,cr4
    or eax,(1<<5)
    mov cr4,eax

    ; Load CR3
    mov eax,0x7000
    mov cr3,eax

    ; Enable EFER.LME (long mode enable)
    mov ecx,0xC0000080
    rdmsr
    or eax,(1<<8)
    wrmsr

    ; Enable paging (bit 31 of CR0)
    mov eax,cr0
    or eax,0x80000000
    mov cr0,eax

    ; Long jump to 64-bit code
    jmp 0x18:0x7C00 + lmode - start_after_bpb

[BITS 64]
lmode:
    mov ax,0x10
    mov ds,ax
    mov es,ax
    mov ss,ax
    mov fs,ax
    mov gs,ax
    mov rsp,0x7B00

    ; Write 6
    mov edi,0xB8000
    mov byte [edi+12], '6'
    mov byte [edi+13],0x70

    ; Jump to kernel at 0x10000!
    jmp 0x10000

[BITS 16]
error:
    mov ax,0xB800
    mov es,ax
    mov byte [es:0],'E'
    mov byte [es:1],0x4F
    cli
    hlt

; GDT - NOW WITH CODE64 SEGMENT!
gdt_start:
    dq 0
gdt_code32:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 0x9A
    db 0xCF
    db 0x00
gdt_data32:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 0x92
    db 0xCF
    db 0x00
gdt_code64:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 0x9A
    db 0x20
    db 0x00
gdt_end:

gdt_desc:
    dw gdt_end - gdt_start - 1
    dd gdt_start

boot_drive: db 0
times 510 - ($ - $$) db 0
dw 0xAA55
