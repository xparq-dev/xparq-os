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

/// Write a single byte to UART (blocking)
unsafe fn uart_putc(c: u8) {
    #[cfg(target_arch = "aarch64")]
    {
        let uart = UART_BASE as *mut u8;
        // Wait for UART transmit holding register empty
        while (*uart.add(5) & 0x20) == 0 {}
        *uart = c;
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
            uart_putc(b'\n');
            uart_putc(b'\r');
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

/// Kernel main function - called from bootloader
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
        uart_puts(b"[XPARQ OS] Booting on x86-64...\n\r");
        uart_puts(b"XPARQ OS Kernel v0.1.0\n\r");
        uart_puts(b"[XPARQ OS] Kernel initialized\n\r");
        uart_puts(b"Entering main kernel loop\n\r");
    }
    loop {
        core::hint::spin_loop();
    }
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

/// Panic handler for kernel
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        uart_puts("KERNEL PANIC!".as_bytes());
        uart_putc(b'\n'); uart_putc(b'\r');
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
