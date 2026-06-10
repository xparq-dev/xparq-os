// XPARQ OS - Phase 02: Simple Kernel for Boot Verification
// Minimal kernel for Phase 2 boot verification

#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// UART base address per architecture
#[cfg(target_arch = "aarch64")]
const UART_BASE: usize = 0x09000000; // PL011 on QEMU virt
#[cfg(target_arch = "x86_64")]
const UART_BASE: usize = 0x03F8;     // COM1 on x86

#[cfg(target_arch = "aarch64")]
const ARM64_UART_DR: usize = 0x00;
#[cfg(target_arch = "aarch64")]
const ARM64_UART_FR: usize = 0x18;
#[cfg(target_arch = "aarch64")]
const ARM64_UART_IBRD: usize = 0x24;
#[cfg(target_arch = "aarch64")]
const ARM64_UART_FBRD: usize = 0x28;
#[cfg(target_arch = "aarch64")]
const ARM64_UART_LCRH: usize = 0x2C;
#[cfg(target_arch = "aarch64")]
const ARM64_UART_CR: usize = 0x30;
#[cfg(target_arch = "aarch64")]
const ARM64_UART_ICR: usize = 0x44;

#[cfg(target_arch = "x86_64")]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

#[cfg(target_arch = "x86_64")]
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

#[cfg(target_arch = "aarch64")]
unsafe fn serial_init() {
    let uart = UART_BASE as *mut u32;

    // PL011 setup for QEMU virt board.
    core::ptr::write_volatile(uart.add(ARM64_UART_CR / 4), 0);
    core::ptr::write_volatile(uart.add(ARM64_UART_ICR / 4), 0x7FF);
    core::ptr::write_volatile(uart.add(ARM64_UART_IBRD / 4), 13);
    core::ptr::write_volatile(uart.add(ARM64_UART_FBRD / 4), 1);
    core::ptr::write_volatile(uart.add(ARM64_UART_LCRH / 4), (1 << 4) | (3 << 5));
    core::ptr::write_volatile(uart.add(ARM64_UART_CR / 4), (1 << 0) | (1 << 8) | (1 << 9));
}

#[cfg(target_arch = "x86_64")]
unsafe fn serial_init() {
    let base = UART_BASE as u16;

    // Configure COM1 for 115200 8N1 so early output is stable in QEMU.
    outb(base + 1, 0x00); // Disable interrupts
    outb(base + 3, 0x80); // Enable DLAB
    outb(base + 0, 0x01); // Divisor low byte (115200 baud)
    outb(base + 1, 0x00); // Divisor high byte
    outb(base + 3, 0x03); // 8 bits, no parity, one stop bit
    outb(base + 2, 0xC7); // Enable FIFO, clear them, 14-byte threshold
    outb(base + 4, 0x0B); // IRQs enabled, RTS/DSR set
}

/// Write a single byte to UART (blocking)
unsafe fn uart_putc(c: u8) {
    #[cfg(target_arch = "aarch64")]
    {
        let uart = UART_BASE as *mut u32;
        // Wait while TX FIFO is full.
        while core::ptr::read_volatile(uart.add(ARM64_UART_FR / 4)) & (1 << 5) != 0 {}
        core::ptr::write_volatile(uart.add(ARM64_UART_DR / 4), c as u32);
    }

    #[cfg(target_arch = "x86_64")]
    {
        // Wait for UART transmit holding register empty (LSR bit 5 at port UART_BASE + 5)
        while (inb((UART_BASE + 5) as u16) & 0x20) == 0 {}
        outb(UART_BASE as u16, c);
    }
}

/// Write a string slice to UART
unsafe fn uart_puts(s: &[u8]) {
    for &byte in s {
        if byte == b'\n' {
            uart_putc(b'\r');
        }
        uart_putc(byte);
    }
}

/// Print string constant (no allocation)
macro_rules! print_str {
    ($s:expr) => {{
        unsafe { uart_puts($s.as_bytes()); }
    }};
}

macro_rules! println_str {
    ($s:expr) => {{
        unsafe {
            uart_puts($s.as_bytes());
            uart_putc(b'\r');
            uart_putc(b'\n');
        }
    }};
}


/// Boot information structure passed from bootloader
#[derive(Debug)]
pub struct BootInfo {
    pub memory_regions: &'static [MemoryRegion],
    pub framebuffer: Option<FramebufferInfo>,
    pub arch_specific: ArchBootInfo,
}

/// Memory region information
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub base: usize,
    pub size: usize,
    pub kind: MemoryRegionKind,
}

/// Memory region types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionKind {
    Usable,
    Reserved,
    AcpiReclaimable,
    Nvs,
    Badram,
    Mmio,
}

/// Framebuffer information
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub address: usize,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
}

/// Pixel formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelFormat {
    Rgb32,
    Bgr32,
}

/// Architecture-specific boot information
#[derive(Debug, Clone, Copy)]
pub struct ArchBootInfo {
    pub rsdp: usize,
    pub bootloader_brand: &'static str,
}

#[cfg(target_arch = "aarch64")]
fn boot_arch_label() -> &'static str {
    "AArch64"
}

#[cfg(target_arch = "x86_64")]
fn boot_arch_label() -> &'static str {
    "x86-64"
}

/// Kernel main function - called from bootloader
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
        serial_init();

        uart_puts(b"[XPARQ OS] Booting on ");
        uart_puts(boot_arch_label().as_bytes());
        uart_puts(b"...\n");
        uart_puts(b"XPARQ OS Kernel v0.1.0\n");
        uart_puts(b"[XPARQ OS] Kernel initialized.\n");
        uart_puts(b"Entering main kernel loop\n");
    }
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("cli; hlt", options(nomem, nostack));
        }
        #[cfg(not(target_arch = "x86_64"))]
        core::hint::spin_loop();
    }
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
#[link_section = ".text.init"]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "adrp x0, {stack}",
        "add x0, x0, :lo12:{stack}",
        "add sp, x0, {stack_top}",
        "bl {kernel_main}",
        stack = sym STACK,
        stack_top = const STACK_TOP_OFFSET,
        kernel_main = sym kernel_main,
    );
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[link_section = ".text.init"]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "lea rsp, [rip + STACK + 16384]",
        "call {kernel_main}",
        kernel_main = sym kernel_main,
    );
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[link_section = ".bss.stack"]
static mut STACK: [u8; 16384] = [0; 16384];

#[cfg(target_arch = "aarch64")]
const STACK_SIZE: usize = 4096;

#[cfg(target_arch = "aarch64")]
const STACK_TOP_OFFSET: usize = STACK_SIZE - 16;

#[cfg(target_arch = "aarch64")]
#[repr(C, align(16))]
struct Arm64Stack([u8; STACK_SIZE]);

#[cfg(target_arch = "aarch64")]
#[no_mangle]
#[link_section = ".bss.stack"]
static mut STACK: Arm64Stack = Arm64Stack([0; STACK_SIZE]);

/// Panic handler for kernel
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        uart_puts("KERNEL PANIC!".as_bytes());
        uart_putc(b'\r'); uart_putc(b'\n');
    }
    loop {
        core::hint::spin_loop();
    }
}


// Global allocator placeholder
#[global_allocator]
static DUMMY_ALLOCATOR: DummyAllocator = DummyAllocator;

/// Dummy allocator for Phase 1
struct DummyAllocator;

unsafe impl core::alloc::GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        // Do nothing
    }
    
    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        let ptr = unsafe { self.alloc(layout) };
        if !ptr.is_null() {
            unsafe { core::ptr::write_bytes(ptr, 0, layout.size()) };
        }
        ptr
    }
    
    unsafe fn realloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout, _new_size: usize) -> *mut u8 {
        core::ptr::null_mut()
    }
}
