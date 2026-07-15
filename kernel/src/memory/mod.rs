// XPARQ OS - Phase 7: Memory Management
// Virtual Memory & Paging

pub mod frame;
pub mod mapper;
pub mod vmo;
pub mod vmar;
pub mod user;

use spin::Mutex;
pub use mapper::Mapper;
pub use vmar::VMAR as Vmar;
pub use vmo::VMO as Vmo;

// Memory error types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryError {
    OutOfMemory,
    InvalidAddress,
    PermissionDenied,
    AlreadyMapped,
    NotMapped,
    InvalidSize,
    AlignmentError,
    ResourceExhausted,
    NotFound,
}

// The global master page table for the Kernel
pub static KERNEL_MAPPER: Mutex<Option<Mapper>> = Mutex::new(None);

pub fn init() {
    unsafe { crate::uart_puts(b"  -> Allocating KERNEL PML4\n"); }
    let mut alloc = frame::FRAME_ALLOCATOR.lock();
    let pml4_phys = alloc.allocate_frame().expect("Failed to allocate KERNEL PML4");
    drop(alloc);

    unsafe { crate::uart_puts(b"  -> Initializing Mapper\n"); }
    // 2. Initialize the Mapper
    let mut mapper = Mapper::new(pml4_phys);

    unsafe { crate::uart_puts(b"  -> Mapping 16MB kernel\n"); }
    // 3. Identity map the kernel code and data (0x0 to 0x100_0000 -> 16MB)
    use xparq_hal::x86_64::paging::PageTableFlags;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    mapper.map_range(0, 0, 4096, flags.clone()).expect("Failed to identity map Kernel"); // 4096 pages = 16MB

    unsafe { crate::uart_puts(b"  -> Mapping 256MB kernel and basic RAM\n"); }
    // 4. Identity map the lower memory (0 to 256MB)
    mapper.map_range(0, 0, 65536, flags.clone()).expect("Failed to identity map 256MB");

    // 4.5. Map VBE Framebuffer dynamically
    let fb_base = unsafe { core::ptr::read_volatile((0x7E00 + 40) as *const u32) };
    if fb_base != 0 {
        unsafe { crate::uart_puts(b"  -> Mapping VBE Framebuffer\n"); }
        let fb_start_page = (fb_base as u64) / 4096;
        // Map 16MB for the framebuffer (4096 pages)
        mapper.map_range(fb_start_page, fb_start_page, 4096, flags.clone()).expect("Failed to map framebuffer");
    }

    // 4.6. Map APIC and IOAPIC (typically at 0xFEE00000 and 0xFEC00000)
    unsafe { crate::uart_puts(b"  -> Mapping APIC and IOAPIC\n"); }
    let apic_page = 0xFEE00000 / 4096;
    mapper.map_range(apic_page, apic_page, 1, flags.clone()).expect("Failed to map APIC");
    let ioapic_page = 0xFEC00000 / 4096;
    mapper.map_range(ioapic_page, ioapic_page, 1, flags.clone()).expect("Failed to map IOAPIC");

    // 4.7. Map PCIe ECAM Space (typically at 0xE0000000, size 256MB)
    unsafe { crate::uart_puts(b"  -> Mapping PCIe ECAM\n"); }
    let ecam_page = 0xE0000000 / 4096;
    mapper.map_range(ecam_page, ecam_page, 65536, flags.clone()).expect("Failed to map ECAM");

    *KERNEL_MAPPER.lock() = Some(mapper);

    // 5. Load CR3 to activate Virtual Memory!
    xparq_hal::x86_64::paging::set_cr3(pml4_phys);
}
