//! ARM64 Memory Management - Phase 2: Dev Environment Setup
//! 
//! This module provides ARM64-specific memory management for XPARQ OS,
//! including:
//! - ARMv8 page table management (4KB pages, 48-bit VA)
//! - LPAE (Large Physical Address Extension) support
//! - Memory attribute configuration (MAIR)
//! - Identity mapping for early boot
//! - Kernel space layout
//! 
//! Page Table Format: 4-level (L0-L3) with 4KB pages
//! Virtual Address Space: 48-bit (256TB)
//! Physical Address Space: Up to 48-bit (256TB)
//! Granule: 4KB (configurable for 16KB/64KB in Phase 2)
//! 
//! Roadmap Phase: Phase 2 - Dev Environment Setup

use super::{sysreg, asm_utils};
use bitflags::bitflags;

/// ARM64 page table entry flags
bitflags! {
    pub struct PageFlags: u64 {
        /// Valid bit (entry is valid)
        const VALID = 1 << 0;
        /// Table entry (points to next level table)
        const TABLE = 1 << 1;
        /// Access flag (has been accessed)
        const ACCESS = 1 << 10;
        /// Shareable attribute
        const SHAREABLE = 1 << 8;
        /// User-mode access
        const USER = 1 << 6;
        /// Read/write access
        const READ_WRITE = 1 << 7;
        /// Execute-never
        const EXECUTE_NEVER = 1 << 54;
        /// Not global (TLB entry not global)
        const NOT_GLOBAL = 1 << 11;
        /// Privileged access only
        const PRIVILEGED = 1 << 6;
    }
}

/// Page table levels
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PageLevel {
    L0 = 0,  // Level 0 (512GB regions)
    L1 = 1,  // Level 1 (1GB regions)
    L2 = 2,  // Level 2 (2MB regions)
    L3 = 3,  // Level 3 (4KB pages)
}

/// Page table entry structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PageTableEntry {
    pub value: u64,
}

impl PageTableEntry {
    /// Create invalid entry
    pub const fn invalid() -> Self {
        Self { value: 0 }
    }
    
    /// Create block entry (maps a region directly)
    pub const fn block(phys_addr: usize, flags: PageFlags) -> Self {
        Self {
            value: (phys_addr as u64) | flags.bits() | PageFlags::VALID.bits(),
        }
    }
    
    /// Create table entry (points to next level table)
    pub const fn table(next_table_addr: usize, flags: PageFlags) -> Self {
        Self {
            value: (next_table_addr as u64) | flags.bits() | PageFlags::TABLE.bits() | PageFlags::VALID.bits(),
        }
    }
    
    /// Check if entry is valid
    pub fn is_valid(&self) -> bool {
        self.value & PageFlags::VALID.bits() != 0
    }
    
    /// Check if entry is a table pointer
    pub fn is_table(&self) -> bool {
        self.value & PageFlags::TABLE.bits() != 0
    }
    
    /// Get physical address from entry
    pub fn phys_addr(&self) -> usize {
        (self.value & 0x0000FFFFFFFFF000) as usize
    }
    
    /// Get flags from entry
    pub fn flags(&self) -> PageFlags {
        PageFlags::from_bits_truncate(self.value)
    }
}

/// Page table structure (512 entries)
#[derive(Debug)]
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    /// Create new empty page table
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::invalid(); 512],
        }
    }
    
    /// Get entry at index
    pub fn entry(&self, index: usize) -> PageTableEntry {
        self.entries[index]
    }
    
    /// Set entry at index
    pub fn set_entry(&mut self, index: usize, entry: PageTableEntry) {
        self.entries[index] = entry;
    }
    
    /// Get physical address of this table
    pub fn phys_addr(&self) -> usize {
        self as *const _ as usize
    }
}

/// Memory manager for ARM64
pub struct MemoryManager {
    /// Root page table (L0)
    root_table: *mut PageTable,
    /// Page table allocator
    page_allocator: PageTableAllocator,
    /// Current memory attributes
    memory_attributes: MemoryAttributes,
}

/// Page table allocator
#[derive(Debug)]
pub struct PageTableAllocator {
    /// Free page table pages
    free_pages: arrayvec::ArrayVec<usize, 64>,
    /// Next page to allocate
    next_page: usize,
}

/// Memory attributes configuration
#[derive(Debug, Clone, Copy)]
pub struct MemoryAttributes {
    /// Memory Attribute Indirection Register value
    pub mair: u64,
}

impl MemoryManager {
    /// Initialize memory management
    pub fn init() {
        println!("Initializing ARM64 memory management...");
        
        // Create root page table
        let root_table = alloc_page_table();
        
        // Initialize page table allocator
        let page_allocator = PageTableAllocator {
            free_pages: arrayvec::ArrayVec::new(),
            next_page: 0x40100000, // Start after kernel
        };
        
        // Set up memory attributes
        let memory_attributes = setup_memory_attributes();
        
        let manager = MemoryManager {
            root_table,
            page_allocator,
            memory_attributes,
        };
        
        // Store global instance
        unsafe {
            MEMORY_MANAGER = Some(manager);
        }
        
        println!("ARM64 memory management initialized");
    }
    
    /// Set up identity mapping for early boot
    pub fn setup_identity_mapping() {
        println!("Setting up identity mapping...");
        
        let manager = unsafe { MEMORY_MANAGER.as_mut().unwrap() };
        
        // Map first 4GB identity (for early boot)
        for addr in (0..4 * 1024 * 1024 * 1024).step_by(2 * 1024 * 1024) {
            map_identity_2mb(manager.root_table, addr);
        }
        
        // Map kernel space (0xFFFF800000000000)
        map_kernel_space(manager.root_table);
        
        // Set TTBR0_EL1 and TTBR1_EL1
        let root_table_phys = unsafe { (*manager.root_table).phys_addr() };
        sysreg::msr("TTBR0_EL1", root_table_phys as u64);
        sysreg::msr("TTBR1_EL1", root_table_phys as u64);
        
        // Set MAIR_EL1
        sysreg::msr("MAIR_EL1", manager.memory_attributes.mair);
        
        // Set TCR_EL1 (Translation Control Register)
        let tcr = (1 << 31) | // TBI1 (Top Byte Ignore for TTBR1)
                  (1 << 23) | // TBI0 (Top Byte Ignore for TTBR0)
                  (0b00 << 12) | // TG0 (Translation Granule 0) = 4KB
                  (0b00 << 8) | // SH0 (Shareability 0) = Inner Shareable
                  (0b01 << 6) | // ORGN0 (Outer RGN 0) = Normal memory
                  (0b01 << 4) | // IRGN0 (Inner RGN 0) = Normal memory
                  (0b10 << 0) | // T0SZ (Translation size 0) = 48-bit VA
                  (0b00 << 28) | // TG1 (Translation Granule 1) = 4KB
                  (0b11 << 24) | // SH1 (Shareability 1) = Inner Shareable
                  (0b01 << 22) | // ORGN1 (Outer RGN 1) = Normal memory
                  (0b01 << 20);  // IRGN1 (Inner RGN 1) = Normal memory
        
        sysreg::msr("TCR_EL1", tcr);
        
        println!("Identity mapping setup complete");
    }
    
    /// Enable virtual memory
    pub fn enable_vm() {
        println!("Enabling virtual memory...");
        
        // Ensure memory barriers
        asm_utils::dsb();
        asm_utils::isb();
        
        // Enable MMU
        let mut sctlr = sysreg::mrs("SCTLR_EL1");
        sctlr |= 1; // Set M bit
        sysreg::msr("SCTLR_EL1", sctlr);
        
        // Invalidate TLB
        sysreg::tlbialle1is();
        
        // Ensure memory barriers
        asm_utils::dsb();
        asm_utils::isb();
        
        println!("Virtual memory enabled");
    }
    
    /// Map a virtual address to physical address
    pub fn map_page(virt_addr: usize, phys_addr: usize, flags: PageFlags) -> Result<(), MemoryError> {
        let manager = unsafe { MEMORY_MANAGER.as_mut().unwrap() };
        
        // Extract page table indices
        let l0_index = (virt_addr >> 39) & 0x1FF;
        let l1_index = (virt_addr >> 30) & 0x1FF;
        let l2_index = (virt_addr >> 21) & 0x1FF;
        let l3_index = (virt_addr >> 12) & 0x1FF;
        
        // Ensure L0 entry exists
        let l0_table = unsafe { &mut *manager.root_table };
        if !l0_table.entry(l0_index).is_valid() {
            let l1_table = manager.page_allocator.alloc_page_table()?;
            let entry = PageTableEntry::table(l1_table, PageFlags::VALID);
            l0_table.set_entry(l0_index, entry);
        }
        
        // Ensure L1 entry exists
        let l1_addr = l0_table.entry(l0_index).phys_addr();
        let l1_table = unsafe { &mut *(l1_addr as *mut PageTable) };
        if !l1_table.entry(l1_index).is_valid() {
            let l2_table = manager.page_allocator.alloc_page_table()?;
            let entry = PageTableEntry::table(l2_table, PageFlags::VALID);
            l1_table.set_entry(l1_index, entry);
        }
        
        // Ensure L2 entry exists
        let l2_addr = l1_table.entry(l1_index).phys_addr();
        let l2_table = unsafe { &mut *(l2_addr as *mut PageTable) };
        if !l2_table.entry(l2_index).is_valid() {
            let l3_table = manager.page_allocator.alloc_page_table()?;
            let entry = PageTableEntry::table(l3_table, PageFlags::VALID);
            l2_table.set_entry(l2_index, entry);
        }
        
        // Set L3 entry (actual page mapping)
        let l3_addr = l2_table.entry(l2_index).phys_addr();
        let l3_table = unsafe { &mut *(l3_addr as *mut PageTable) };
        let entry = PageTableEntry::block(phys_addr, flags);
        l3_table.set_entry(l3_index, entry);
        
        // Invalidate TLB for this page
        asm_utils::dsb();
        unsafe {
            core::arch::asm!("tlbi vae1is, {}", in(reg) virt_addr as u64);
        }
        asm_utils::dsb();
        asm_utils::isb();
        
        Ok(())
    }
    
    /// Get physical address from virtual address
    pub fn virt_to_phys(virt_addr: usize) -> Option<usize> {
        let manager = unsafe { MEMORY_MANAGER.as_ref().unwrap() };
        
        // Extract page table indices
        let l0_index = (virt_addr >> 39) & 0x1FF;
        let l1_index = (virt_addr >> 30) & 0x1FF;
        let l2_index = (virt_addr >> 21) & 0x1FF;
        let l3_index = (virt_addr >> 12) & 0x1FF;
        
        // Walk page tables
        let l0_table = unsafe { &*manager.root_table };
        if !l0_table.entry(l0_index).is_valid() {
            return None;
        }
        
        let l1_addr = l0_table.entry(l0_index).phys_addr();
        let l1_table = unsafe { &*(l1_addr as *const PageTable) };
        if !l1_table.entry(l1_index).is_valid() {
            return None;
        }
        
        let l2_addr = l1_table.entry(l1_index).phys_addr();
        let l2_table = unsafe { &*(l2_addr as *const PageTable) };
        if !l2_table.entry(l2_index).is_valid() {
            return None;
        }
        
        let l3_addr = l2_table.entry(l2_index).phys_addr();
        let l3_table = unsafe { &*(l3_addr as *const PageTable) };
        if !l3_table.entry(l3_index).is_valid() {
            return None;
        }
        
        let page_offset = virt_addr & 0xFFF;
        let page_phys = l3_table.entry(l3_index).phys_addr();
        Some(page_phys + page_offset)
    }
}

/// Memory management errors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryError {
    OutOfMemory,
    InvalidAddress,
    AlreadyMapped,
    PermissionDenied,
}

/// Setup memory attributes
fn setup_memory_attributes() -> MemoryAttributes {
    // MAIR_EL1 configuration
    // Attr0: Device-nGnRnE memory
    // Attr1: Normal memory, Write-Back, Read-Allocate, Write-Allocate
    let mair = (0b00000000 << 0) |  // Attr0: Device memory
               (0b11111111 << 8);   // Attr1: Normal memory
    
    MemoryAttributes { mair }
}

/// Map 2MB identity region
fn map_identity_2mb(root_table: *mut PageTable, addr: usize) {
    let l0_index = (addr >> 39) & 0x1FF;
    let l1_index = (addr >> 30) & 0x1FF;
    
    let l0_table = unsafe { &mut *root_table };
    
    // Create L1 block entry (2MB mapping)
    let flags = PageFlags::VALID | 
               PageFlags::ACCESS | 
               PageFlags::SHAREABLE | 
               PageFlags::READ_WRITE;
    
    let entry = PageTableEntry::block(addr, flags);
    l0_table.set_entry(l0_index * 512 + l1_index, entry);
}

/// Map kernel space
fn map_kernel_space(root_table: *mut PageTable) {
    let l0_index = (0xFFFF800000000000 >> 39) & 0x1FF;
    
    let l0_table = unsafe { &mut *root_table };
    
    // Map kernel space to same physical memory (identity mapping for now)
    // Phase 2: Proper kernel space layout with high memory mapping
    let kernel_phys = 0x40080000; // Kernel load address
    let flags = PageFlags::VALID | 
               PageFlags::ACCESS | 
               PageFlags::SHAREABLE | 
               PageFlags::READ_WRITE | 
               PageFlags::EXECUTE_NEVER; // Kernel memory non-executable for data
    
    let entry = PageTableEntry::table(kernel_phys, flags);
    l0_table.set_entry(l0_index, entry);
}

/// Page table allocator implementation
impl PageTableAllocator {
    /// Allocate a page table page
    pub fn alloc_page_table(&mut self) -> Result<usize, MemoryError> {
        if let Some(page) = self.free_pages.pop() {
            return Ok(page);
        }
        
        let page = self.next_page;
        self.next_page += 4096;
        
        if self.next_page > 0x80000000 {
            return Err(MemoryError::OutOfMemory);
        }
        
        // Zero the page
        let page_ptr = page as *mut u8;
        unsafe {
            core::ptr::write_bytes(page_ptr, 0, 4096);
        }
        
        Ok(page)
    }
    
    /// Free a page table page
    pub fn free_page_table(&mut self, page: usize) {
        if self.free_pages.len() < 64 {
            self.free_pages.push(page);
        }
    }
}

/// Allocate a page table
fn alloc_page_table() -> *mut PageTable {
    extern "C" {
        static mut __bss_end: u8;
    }
    
    static mut PAGE_TABLE_PTR: Option<*mut u8> = None;
    
    let ptr = if let Some(ptr) = PAGE_TABLE_PTR {
        ptr
    } else {
        PAGE_TABLE_PTR = Some(&mut __bss_end as *mut _ as *mut u8);
        PAGE_TABLE_PTR.unwrap()
    };
    
    let aligned_ptr = (ptr as usize + 4095) & !4095;
    let result = aligned_ptr as *mut PageTable;
    
    // Update pointer for next allocation
    PAGE_TABLE_PTR = Some((aligned_ptr as *mut u8).add(4096));
    
    // Zero the page table
    unsafe {
        core::ptr::write_bytes(result as *mut u8, 0, 4096);
    }
    
    result
}

/// Global memory manager instance
static mut MEMORY_MANAGER: Option<MemoryManager> = None;

/// Get global memory manager
pub fn get_memory_manager() -> &'static MemoryManager {
    unsafe { MEMORY_MANAGER.as_ref().unwrap() }
}

/// Public API for memory management
pub mod api {
    use super::*;
    
    /// Map a page
    pub fn map_page(virt_addr: usize, phys_addr: usize, flags: PageFlags) -> Result<(), MemoryError> {
        MemoryManager::map_page(virt_addr, phys_addr, flags)
    }
    
    /// Get physical address
    pub fn virt_to_phys(virt_addr: usize) -> Option<usize> {
        MemoryManager::virt_to_phys(virt_addr)
    }
    
    /// Flush TLB entry
    pub fn flush_tlb(virt_addr: usize) {
        asm_utils::dsb();
        unsafe {
            core::arch::asm!("tlbi vae1is, {}", in(reg) virt_addr as u64);
        }
        asm_utils::dsb();
        asm_utils::isb();
    }
}
