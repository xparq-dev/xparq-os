// XPARQ OS - Simple MBR Bootloader in Rust
// Fits in 512 bytes at 0x7C00

#![no_std]
#![no_main]

use core::arch::asm;

// MBR bootloader entry point at 0x7C00
#[no_mangle]
#[link_section = ".mbr"]
pub extern "C" fn _mbr_start() -> ! {
    unsafe {
        // Set up segments
        asm!(
            "xor ax, ax",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov sp, 0x7C00",
            lateout("ax") _,
            options(nomem, nostack)
        );

        // Print boot message
        print_msg(b"XPARQ OS Bootloader\r\n");

        // Load kernel from sector 1 (LBA 1) to 0x10000
        // Using BIOS INT 13h
        asm!(
            // Set up disk read parameters
            "mov ah, 0x02",      // Read sectors
            "mov al, 64",        // Read 64 sectors (32KB)
            "mov ch, 0",         // Cylinder 0
            "mov cl, 2",         // Sector 2 (1-indexed, sector 1 is MBR)
            "mov dh, 0",         // Head 0
            "mov dl, 0x80",      // First hard disk
            "mov bx, 0x0000",    // Offset 0
            "mov ax, 0x1000",    // Segment 0x1000 = 0x10000 physical
            "mov es, ax",
            "int 0x13",          // BIOS disk interrupt
            "jc {error}",        // Jump if carry (error)
            error = sym disk_error,
            options(nomem, nostack)
        );

        print_msg(b"Kernel loaded\r\n");

        // Jump to kernel at 0x10000 using far jump
        asm!(
            "push 0x1000",       // Push segment
            "push 0x0000",       // Push offset
            "retf",              // Far return to 0x1000:0x0000
            options(nomem, nostack, noreturn)
        );
    }
}

unsafe fn print_msg(msg: &[u8]) {
    for &byte in msg {
        asm!(
            "mov ah, 0x0E",      // BIOS teletype
            "mov al, {byte}",    // Character to print
            "mov bh, 0",         // Page 0
            "mov bl, 7",         // Light gray
            "int 0x10",          // BIOS video interrupt
            byte = in(reg_byte) byte,
            options(nomem, nostack)
        );
    }
}

// Disk error handler
#[no_mangle]
pub extern "C" fn disk_error() -> ! {
    unsafe {
        print_msg(b"Disk error!\r\n");
        asm!("hlt", options(nomem, nostack, noreturn));
    }
}

// Fill to 510 bytes and add boot signature
// The linker script will handle this
#[used]
#[link_section = ".boot_signature"]
static BOOT_SIGNATURE: [u8; 2] = [0x55, 0xAA];

// Panic handler required for no_std
use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
