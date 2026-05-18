// XPARQ OS - Phase 02: Simple Kernel for Boot Verification
#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;
use core::arch::global_asm;

/// PVH note for QEMU x86-64 -kernel support (only on x86)
#[cfg(target_arch = "x86_64")]
#[used]
#[link_section = ".note.xparq.pvh"]
static PVH_NOTE: [u8; 20] = [
    4,0,0,0, 4,0,0,0, 18,0,0,0,
    b'X',b'e',b'n',0,
    0x00,0x10,0x00,0x00,
];

/// Architecture-specific UART implementation
#[cfg(target_arch = "aarch64")]
mod uart {
    const UART_BASE: usize = 0x09000000; // PL011
    unsafe fn wait() {
        while ((UART_BASE as *const u8).add(0x18).read_volatile() & (1 << 5)) != 0 {}
    }
    pub unsafe fn putc(c: u8) {
        wait();
        (UART_BASE as *mut u8).write_volatile(c);
    }
    pub unsafe fn puts(s: &[u8]) {
        for &b in s { putc(b); }
    }
}

#[cfg(target_arch = "x86_64")]
mod uart {
    const UART_PORT: u16 = 0x3F8;
    const UART_LSR: u16 = 0x3FD;

    unsafe fn inb(port: u16) -> u8 {
        let val: u8;
        core::arch::asm!("inb %dx, %al", out("al") val, in("dx") port);
        val
    }
    unsafe fn outb(port: u16, val: u8) {
        core::arch::asm!("outb %al, %dx", in("al") val, in("dx") port);
    }
    unsafe fn wait() {
        while (inb(UART_LSR) & 0x20) == 0 {}
    }
    pub unsafe fn putc(c: u8) {
        wait();
        outb(UART_PORT, c);
    }
    pub unsafe fn puts(s: &[u8]) {
        for &b in s { putc(b); }
    }
}

/// Kernel entry point (called from assembly stub)
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        uart::puts("[XPARQ OS] Booting on AArch64...".as_bytes());
        uart::putc(b'\n'); uart::putc(b'\r');
        uart::puts("[XPARQ OS] Kernel initialized.".as_bytes());
        uart::putc(b'\n'); uart::putc(b'\r');
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        uart::puts("[XPARQ OS] Booting on x86-64...".as_bytes());
        uart::putc(b'\n'); uart::putc(b'\r');
        uart::puts("[XPARQ OS] Kernel initialized.".as_bytes());
        uart::putc(b'\n'); uart::putc(b'\r');
    }

    loop {
        #[cfg(target_arch = "aarch64")]
        unsafe { core::arch::asm!("wfi") };
        #[cfg(target_arch = "x86_64")]
        unsafe { core::arch::asm!("hlt") };
    }
}

/// Panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        #[cfg(target_arch = "aarch64")]
        unsafe { core::arch::asm!("wfi") };
        #[cfg(target_arch = "x86_64")]
        unsafe { core::arch::asm!("hlt") };
    }
}

// =============================================
// Assembly entry stubs
// =============================================

#[cfg(target_arch = "x86_64")]
global_asm!(
    ".section .text.entry",
    ".globl _start",
    "_start:",
    "    lea rsp, [rip + stack_top]",
    // Debug: output 'S' serial
    "    mov al, 'S'",
    "    out 0x3F8, al",
    "    call kernel_main",
    "    cli",
    "1:  hlt",
    "    jmp 1b",
    ".section .bss",
    ".align 16",
    "stack_bottom:",
    ".zero 4096",
    "stack_top:",
);

#[cfg(target_arch = "aarch64")]
global_asm!(
    ".section .text.entry",
    ".globl _start",
    "_start:",
    "    adr x0, stack_top",
    "    mov sp, x0",
    "    bl kernel_main",
    "1:  wfi",
    "    b 1b",
    ".section .bss",
    ".align 16",
    "stack_bottom:",
    ".zero 4096",
    "stack_top:",
);
