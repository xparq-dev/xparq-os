// XPARQ OS - Phase 01: OS & Kernel Foundations
// Kernel entry point - architecture-agnostic main function
// This is the main entry point that calls architecture-specific initialization

#![no_std]

// Core modules
mod capability;
mod memory;
mod ipc;
mod scheduler;

// Architecture-specific modules
#[cfg(target_arch = "aarch64")]
#[path = "arch/arm64/mod.rs"]
mod arch;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
mod arch;

use memory::vmo::{VMO, VMORights};
use capability::{Capability, Handle};
use scheduler::{Thread, ThreadState};

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

/// Kernel main function - called from architecture-specific entry points
/// 
/// This function performs architecture-agnostic kernel initialization:
/// 1. Initialize memory management
/// 2. Set up capability system
/// 3. Start scheduler
/// 4. Initialize IPC system
/// 5. Start system services
#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &BootInfo) -> ! {
    // Early initialization
    early_init(boot_info);
    
    // Initialize memory management
    memory::init(boot_info);
    
    // Initialize capability system
    capability::init();
    
    // Initialize scheduler
    scheduler::init();
    
    // Initialize IPC system
    ipc::init();
    
    // Start system services
    start_system_services();
    
    // Enable interrupts and start scheduling
    arch::interrupts::enable();
    
    // Main kernel loop
    kernel_loop();
}

/// Early initialization before memory management is available
fn early_init(boot_info: &BootInfo) {
    // Print architecture-specific boot message
    #[cfg(target_arch = "aarch64")]
    println!("[XPARQ OS] Booting on AArch64...");
    
    #[cfg(target_arch = "x86_64")]
    println!("[XPARQ OS] Booting on x86-64...");
    
    // Print boot information
    println!("XPARQ OS Kernel v0.1.0");
    println!("Bootloader: {}", boot_info.arch_specific.bootloader_brand);
    println!("Memory regions: {}", boot_info.memory_regions.len());
    
    if let Some(fb) = boot_info.framebuffer {
        println!("Framebuffer: {}x{} @ 0x{:x}", fb.width, fb.height, fb.address);
    }
    
    // Architecture-specific early init
    arch::early_init(boot_info);
}

/// Start essential system services
fn start_system_services() {
    println!("Starting system services...");
    
    // Create initial system job
    let root_job = capability::create_job().expect("Failed to create root job");
    
    // Create kernel process
    let kernel_process = capability::create_process(root_job, "kernel").expect("Failed to create kernel process");
    
    // Start scheduler thread
    let scheduler_thread = scheduler::create_thread(
        kernel_process,
        scheduler_main as usize,
        0,
        0,
    ).expect("Failed to create scheduler thread");
    
    // Start scheduler thread
    scheduler::resume_thread(scheduler_thread);
    
    println!("System services started");
}

/// Scheduler main loop
fn scheduler_main() -> ! {
    println!("Scheduler started");
    
    loop {
        // Schedule next thread
        scheduler::schedule_next();
        
        // Yield CPU if no threads ready
        scheduler::yield_cpu();
    }
}

/// Main kernel loop
fn kernel_loop() -> ! {
    println!("XPARQ OS Kernel initialized");
    println!("Entering main kernel loop");
    
    loop {
        // Handle system events
        handle_system_events();
        
        // Perform maintenance tasks
        perform_maintenance();
        
        // Yield CPU
        scheduler::yield_cpu();
    }
}

/// Handle system events
fn handle_system_events() {
    // Phase 1: Placeholder for event handling
    // Phase 2: Implement interrupt handling, IPC events, etc.
}

/// Perform maintenance tasks
fn perform_maintenance() {
    // Phase 1: Placeholder for maintenance
    // Phase 2: Implement garbage collection, memory compaction, etc.
}

/// Panic handler for kernel
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("KERNEL PANIC!");
    println!("Location: {:?}", info.location());
    println!("Message: {}", info);
    
    // Halt system
    arch::cpu::halt();
    
    loop {
        core::hint::spin_loop();
    }
}

/// Dummy allocator for Phase 1
/// 
/// Phase 2: Replace with proper heap allocator
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    println!("Allocation failed!");
    arch::cpu::halt();
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
        // Phase 1: Return null to trigger allocation failure
        // Phase 2: Implement proper allocation
        core::ptr::null_mut()
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        // Phase 1: Do nothing
        // Phase 2: Implement proper deallocation
    }
    
    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        // Phase 1: Return null
        // Phase 2: Implement proper zeroed allocation
        core::ptr::null_mut()
    }
    
    unsafe fn realloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout, _new_size: usize) -> *mut u8 {
        // Phase 1: Return null
        // Phase 2: Implement proper reallocation
        core::ptr::null_mut()
    }
}
