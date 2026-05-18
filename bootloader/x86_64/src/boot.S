# XPARQ OS - Simple MBR Bootloader (x86_64)
# Fits in 512 bytes, loads at 0x7C00

.code16
.global _start

_start:
    # Set up segments
    xorw %ax, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %ss
    movw $0x7C00, %sp

    # Save drive number
    movb %dl, drive_number

    # Print boot message
    movw $boot_msg, %si
    call print_string

    # Load kernel from disk (LBA 1, 64 sectors = 32KB)
    movw $0x1000, %ax      # Load to 0x1000:0x0000 = 0x10000
    movw %ax, %es
    xorw %bx, %bx
    movb $1, %cl           # Sector 1 (after MBR)
    movb $64, %dh          # Read 64 sectors
    call read_sectors

    # Check if read successful
    jc disk_error

    # Print success message
    movw $load_msg, %si
    call print_string

    # Jump to kernel at 0x10000 (or 0x100000 if we loaded higher)
    # For now, jump to 0x10000
    ljmp $0x1000, $0x0000

# Print string at DS:SI
print_string:
    pusha
    movb $0x0E, %ah        # BIOS teletype
1:
    lodsb
    testb %al, %al
    jz 2f
    int $0x10
    jmp 1b
2:
    popa
    ret

# Read sectors from disk using CHS
# ES:BX = buffer, CL = start sector, DH = sector count
disk_error:
    movw $error_msg, %si
    call print_string
    jmp .

read_sectors:
    pusha
    movb $0x02, %ah        # Read sectors
    movb %dh, %al          # Sector count
    movb $0x00, %ch        # Cylinder 0
    movb %cl, %cl          # Sector number
    movb $0x00, %dh        # Head 0
    int $0x13
    jc disk_error
    popa
    ret

boot_msg:    .asciz "XPARQ OS Bootloader\r\n"
load_msg:    .asciz "Kernel loaded\r\n"
error_msg:   .asciz "Disk error!\r\n"
drive_number:.byte 0

# Padding to 510 bytes
.org 510
.word 0xAA55             # Boot signature
