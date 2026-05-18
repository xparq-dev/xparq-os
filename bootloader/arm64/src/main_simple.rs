// XPARQ OS - Phase 02: Simple ARM64 Bootloader
// Minimal bootloader for Phase 2 boot verification

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

// Simple println macro for no_std debugging
macro_rules! println {
    ($($arg:tt)*) => {
        // Phase 1: No output in no_std
        // Phase 2: Use actual console/serial output
        // For now, we'll use a simple serial output implementation
        #[cfg(target_arch = "aarch64")]
        {
            // ARM64 serial output - Phase 1: Simple hardcoded messages
            if false {
                // Write to PL011 UART at 0x9000000
                let uart = 0x9000000 as *mut u8;
                unsafe {
                    // Wait for UART to be ready
                    while *uart.add(5) & 0x20 == 0 {}
                    *uart = b'X'; // Simple test output
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
                    *com1 = b'X'; // Simple test output
                }
            }
        }
    };
}

use core::panic::PanicInfo;

/// Boot information structure passed to kernel
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

/// Kernel arguments structure
#[derive(Debug, Clone, Copy)]
pub struct KernelArgs {
    pub boot_info_ptr: u64,
    pub cpu_count: u32,
    pub current_cpu: u32,
}

/// Bootloader entry point for ARM64
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Print architecture-specific boot message
    #[cfg(target_arch = "aarch64")]
    println!("[XPARQ OS] Booting on AArch64...");
    
    #[cfg(target_arch = "x86_64")]
    println!("[XPARQ OS] Booting on x86-64...");
    
    println!("XPARQ OS Bootloader v0.1.0");
    println!("Initializing minimal bootloader...");
    
    // Create simple boot information
    let boot_info = BootInfo {
        memory_regions: &[
            MemoryRegion {
                base: 0x40000000,
                size: 512 * 1024 * 1024, // 512MB for kernel
                kind: MemoryRegionKind::Usable,
            },
            MemoryRegion {
                base: 0x00000000,
                size: 0x40000000, // 1GB for devices
                kind: MemoryRegionKind::Mmio,
            },
        ],
        framebuffer: None, // Phase 1: No framebuffer
        arch_specific: ArchBootInfo {
            rsdp: 0,
            bootloader_brand: "XPARQ OS Bootloader",
        },
    };
    
    println!("Boot information created");
    
    // Prepare kernel arguments
    let kernel_args = KernelArgs {
        boot_info_ptr: &boot_info as *const BootInfo as u64,
        cpu_count: 1,
        current_cpu: 0,
    };
    
    println!("Jumping to kernel...");
    
    // Jump to kernel
    unsafe {
        // For Phase 2, we'll use a dummy kernel entry point
        // In a real bootloader, this would be the actual kernel entry point
        let kernel_entry: extern "C" fn(KernelArgs) -> ! = 
            core::mem::transmute(0x40000000usize); // Dummy kernel address
        
        kernel_entry(kernel_args);
    }
}

/// Kernel main function - called from bootloader
#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &BootInfo) -> ! {
    println!("XPARQ OS Kernel v0.1.0");
    println!("Bootloader: {}", boot_info.arch_specific.bootloader_brand);
    println!("Memory regions: {}", boot_info.memory_regions.len());
    
    if let Some(fb) = boot_info.framebuffer {
        println!("Framebuffer: {}x{} @ 0x{:x}", fb.width, fb.height, fb.address);
    }
    
    println!("XPARQ OS Kernel initialized");
    println!("Entering main kernel loop");
    
    // Main kernel loop
    loop {
        // Phase 1: Simple infinite loop
        // Phase 2: Implement proper kernel scheduling
    }
}

/// Panic handler for bootloader
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Bootloader panic: {}", info);
    
    // Halt system
    loop {
        core::hint::spin_loop();
    }
}

// Only one panic handler allowed per crate

// Dummy allocator for Phase 1
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    println!("Allocation failed!");
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
