// XPARQ OS - Phase 02: Simple Kernel for Boot Verification
// Minimal kernel for Phase 2 boot verification

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

use core::panic::PanicInfo;

/// UART base address per architecture
#[cfg(target_arch = "aarch64")]
const UART_BASE: usize = 0x09000000; // PL011 on QEMU virt
#[cfg(target_arch = "x86_64")]
const UART_BASE: usize = 0x03F8;     // COM1 on x86

/// Write a single byte to UART (blocking)
unsafe fn uart_putc(c: u8) {
    let uart = UART_BASE as *mut u8;
    // Wait for UART transmit holding register empty
    while (*uart.add(5) & 0x20) == 0 {}
    *uart = c;
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
            }
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            // x86_64 serial output - Phase 1: Simple hardcoded messages
            if false {
                // Write to COM1 at 0x3F8
                let com1 = 0x3F8 as *mut u8;
                unsafe {
                    // Wait for UART to be ready
                    while *com1.add(5) & 0x20 == 0 {}
                    *com1 = b'K'; // Simple test output
                }
            }
        }
    };
}

use core::panic::PanicInfo;

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
pub extern "C" fn kernel_main(boot_info: &BootInfo) -> ! {
    // Print architecture-specific boot message
    #[cfg(target_arch = "aarch64")]
    unsafe {
        uart_puts("[XPARQ OS] Booting on AArch64...".as_bytes());
        uart_putc(b'\n'); uart_putc(b'\r');
    }
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        uart_puts("[XPARQ OS] Booting on x86-64...".as_bytes());
        uart_putc(b'\n'); uart_putc(b'\r');
    }
    
    unsafe {
        uart_puts("XPARQ OS Kernel v0.1.0".as_bytes());
        uart_putc(b'\n'); uart_putc(b'\r');
        uart_puts("Bootloader: ".as_bytes());
        uart_puts(boot_info.arch_specific.bootloader_brand.as_bytes());
        uart_putc(b'\n'); uart_putc(b'\r');
        uart_puts("Memory regions: ".as_bytes());
        // Simple number printing (count up to 9)
        let count = boot_info.memory_regions.len();
        if count < 10 {
            uart_putc(b'0' + count as u8);
        }
        uart_putc(b'\n'); uart_putc(b'\r');
        
        if let Some(_fb) = boot_info.framebuffer {
            uart_puts("Framebuffer: 1024x768 @ 0x".as_bytes());
            // simplified - just print marker
        }
        
        uart_puts("[XPARQ OS] Kernel initialized".as_bytes());
        uart_putc(b'\n'); uart_putc(b'\r');
        uart_puts("Entering main kernel loop".as_bytes());
        uart_putc(b'\n'); uart_putc(b'\r');
    }
    
    // Main kernel loop
    loop {
        // Phase 1: Simple infinite loop
        // Phase 2: Implement proper kernel scheduling
    }
}

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

// Dummy allocator for Phase 1
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    unsafe {
        uart_puts("Allocation failed!".as_bytes());
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
