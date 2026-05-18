// XPARQ OS - Phase 01: OS & Kernel Foundations
// x86-64 MMU implementation
// Handles x86-64 PML4 page tables and virtual memory management

#![no_std]

/// x86-64 MMU constants
pub const PAGE_SIZE: usize = 4096; // 4KB pages
pub const PAGE_SHIFT: usize = 12;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

/// Page table levels for x86-64
pub const PML4_ENTRIES: usize = 512;
pub const PDPT_ENTRIES: usize = 512;
pub const PDT_ENTRIES: usize = 512;
pub const PT_ENTRIES: usize = 512;

/// Page table entry bits
pub mod pte_bits {
    pub const PRESENT: u64 = 1 << 0;
    pub const READ_WRITE: u64 = 1 << 1;
    pub const USER_SUPERVISOR: u64 = 1 << 2;
    pub const WRITE_THROUGH: u64 = 1 << 3;
    pub const CACHE_DISABLE: u64 = 1 << 4;
    pub const ACCESSED: u64 = 1 << 5;
    pub const DIRTY: u64 = 1 << 6;
    pub const PAT: u64 = 1 << 7;
    pub const GLOBAL: u64 = 1 << 8;
    pub const AVAILABLE1: u64 = 1 << 9;
    pub const AVAILABLE2: u64 = 1 << 10;
    pub const AVAILABLE3: u64 = 1 << 11;
    pub const NX: u64 = 1 << 63; // No-execute bit
}

/// Memory attributes
#[derive(Debug, Clone, Copy)]
pub struct MemoryAttributes {
    pub device: bool,
    pub cacheable: bool,
    pub write_through: bool,
    pub user_accessible: bool,
    pub executable: bool,
    pub writable: bool,
    pub global: bool,
}

/// Page table entry
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
    pub value: u64,
}

impl PageTableEntry {
    /// Create an invalid entry
    pub const fn invalid() -> Self {
        Self { value: 0 }
    }
    
    /// Create a table entry pointing to next level
    pub fn table(address: usize, attributes: MemoryAttributes) -> Self {
        let mut value = (address as u64) & !0xFFF; // Clear lower 12 bits
        value |= pte_bits::PRESENT;
        
        if attributes.writable {
            value |= pte_bits::READ_WRITE;
        }
        
        if attributes.user_accessible {
            value |= pte_bits::USER_SUPERVISOR;
        }
        
        if attributes.write_through {
            value |= pte_bits::WRITE_THROUGH;
        }
        
        if !attributes.cacheable {
            value |= pte_bits::CACHE_DISABLE;
        }
        
        Self { value }
    }
    
    /// Create a page entry
    pub fn page(address: usize, attributes: MemoryAttributes) -> Self {
        let mut value = (address as u64) & !0xFFF; // Clear lower 12 bits
        value |= pte_bits::PRESENT;
        
        if attributes.writable {
            value |= pte_bits::READ_WRITE;
        }
        
        if attributes.user_accessible {
            value |= pte_bits::USER_SUPERVISOR;
        }
        
        if attributes.write_through {
            value |= pte_bits::WRITE_THROUGH;
        }
        
        if !attributes.cacheable {
            value |= pte_bits::CACHE_DISABLE;
        }
        
        if attributes.global {
            value |= pte_bits::GLOBAL;
        }
        
        // Set NX bit if not executable
        if !attributes.executable {
            value |= pte_bits::NX;
        }
        
        Self { value }
    }
    
    /// Check if entry is present
    pub fn is_present(&self) -> bool {
        self.value & pte_bits::PRESENT != 0
    }
    
    /// Check if entry is a table (not a huge page)
    pub fn is_table(&self) -> bool {
        self.is_present() && (self.value & (1 << 7) == 0)
    }
    
    /// Check if entry is a huge page
    pub fn is_huge_page(&self) -> bool {
        self.is_present() && (self.value & (1 << 7) != 0)
    }
    
    /// Get physical address from entry
    pub fn get_address(&self) -> usize {
        (self.value & !0xFFF) as usize
    }
    
    /// Set accessed flag
    pub fn set_accessed(&mut self) {
        self.value |= pte_bits::ACCESSED;
    }
    
    /// Set dirty flag
    pub fn set_dirty(&mut self) {
        self.value |= pte_bits::DIRTY;
    }
}

/// Page table
#[derive(Debug)]
pub struct PageTable {
    entries: [PageTableEntry; PML4_ENTRIES],
}

impl PageTable {
    /// Create a new empty page table
    pub fn new() -> Self {
        Self {
            entries: [PageTableEntry::invalid(); PML4_ENTRIES],
        }
    }
    
    /// Get entry at index
    pub fn get_entry(&self, index: usize) -> PageTableEntry {
        self.entries[index]
    }
    
    /// Set entry at index
    pub fn set_entry(&mut self, index: usize, entry: PageTableEntry) {
        self.entries[index] = entry;
    }
    
    /// Clear all entries
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = PageTableEntry::invalid();
        }
    }
}

/// MMU manager
pub struct MMU {
    /// PML4 (root) page table
    pml4_table: PageTable,
    /// Page table allocator
    page_tables: spin::Mutex<PageTableAllocator>,
}

/// Page table allocator
#[derive(Debug)]
struct PageTableAllocator {
    /// Allocated page tables
    tables: arrayvec::ArrayVec<*mut PageTable, 64>,
    /// Next free table
    next_free: usize,
}

impl MMU {
    /// Create new MMU instance
    pub fn new() -> Self {
        Self {
            pml4_table: PageTable::new(),
            page_tables: spin::Mutex::new(PageTableAllocator {
                tables: arrayvec::ArrayVec::new(),
                next_free: 0,
            }),
        }
    }
    
    /// Initialize MMU with identity mapping
    pub fn init_identity_mapping(&mut self) -> Result<(), &'static str> {
        println!("Initializing x86-64 MMU with identity mapping...");
        
        // Phase 1: Identity map first 1GB
        // Phase 2: Full memory mapping
        
        // Identity map first 1GB (0x00000000 - 0x40000000)
        for i in 0..512 {
            let virt_addr = i * PAGE_SIZE * 512 * 512; // 1GB blocks
            let phys_addr = virt_addr;
            
            let attributes = MemoryAttributes {
                device: false,
                cacheable: true,
                write_through: false,
                user_accessible: false,
                executable: true,
                writable: true,
                global: false,
            };
            
            // Create PML4 entry pointing to PDPT table
            let pdpt_table = self.allocate_page_table()?;
            let pml4_entry = PageTableEntry::table(pdpt_table as usize, attributes);
            self.pml4_table.set_entry(i, pml4_entry);
            
            // For Phase 1, we'll skip creating PDPT/PDT/PT tables
            // Phase 2: Create full 4-level page tables
        }
        
        println!("x86-64 MMU identity mapping initialized");
        Ok(())
    }
    
    /// Allocate a page table
    fn allocate_page_table(&self) -> Result<*mut PageTable, &'static str> {
        let mut allocator = self.page_tables.lock();
        
        if allocator.tables.is_full() {
            return Err("Out of page tables");
        }
        
        // Phase 1: Use static allocation
        // Phase 2: Use proper memory allocator
        
        static mut PAGE_TABLES: [[PageTable; 64]; 1] = [[PageTable::new(); 64]; 1];
        
        let table = unsafe {
            &mut PAGE_TABLES[0][allocator.next_free] as *mut PageTable
        };
        
        allocator.tables.push(table);
        allocator.next_free += 1;
        
        Ok(table)
    }
    
    /// Enable MMU
    pub fn enable(&self) {
        println!("Enabling x86-64 MMU...");
        
        unsafe {
            // Get PML4 table address
            let pml4_addr = &self.pml4_table as *const PageTable as u64;
            
            // Load CR3 with PML4 address
            super::boot::regs::write_cr3(pml4_addr);
            
            // Enable paging in CR0
            let mut cr0 = super::boot::regs::read_cr0();
            cr0 |= 1 << 31; // PG bit
            super::boot::regs::write_cr0(cr0);
            
            // Ensure MMU enable takes effect
            super::boot::regs::memory_barrier();
        }
        
        println!("x86-64 MMU enabled");
    }
    
    /// Map a virtual address to physical address
    pub fn map_page(&mut self, virt_addr: usize, phys_addr: usize, attributes: MemoryAttributes) -> Result<(), &'static str> {
        // Phase 1: Dummy implementation
        // Phase 2: Full page table management
        
        println!("Mapping 0x{:x} -> 0x{:x}", virt_addr, phys_addr);
        
        // For Phase 1, we'll just return success
        Ok(())
    }
    
    /// Unmap a virtual address
    pub fn unmap_page(&mut self, virt_addr: usize) -> Result<(), &'static str> {
        // Phase 1: Dummy implementation
        // Phase 2: Full page table management
        
        println!("Unmapping 0x{:x}", virt_addr);
        
        Ok(())
    }
    
    /// Get physical address from virtual address
    pub fn get_physical_address(&self, virt_addr: usize) -> Option<usize> {
        // Phase 1: Dummy implementation
        // Phase 2: Walk page tables
        
        // For Phase 1, return identity mapping
        Some(virt_addr)
    }
    
    /// Invalidate TLB entry
    pub fn invalidate_tlb_entry(&self, virt_addr: usize) {
        unsafe {
            // Use INVLPG instruction
            core::arch::asm!("invlpg [{}]", in(reg) virt_addr);
        }
    }
    
    /// Invalidate entire TLB
    pub fn invalidate_tlb(&self) {
        unsafe {
            // Reload CR3 to invalidate TLB
            let cr3 = super::boot::regs::read_cr3();
            super::boot::regs::write_cr3(cr3);
        }
    }
}

/// Initialize the MMU system
pub fn init() {
    println!("Initializing x86-64 MMU system...");
    
    // Phase 1: Create MMU with identity mapping
    // Phase 2: Full MMU initialization
    
    let mut mmu = MMU::new();
    
    // Initialize identity mapping
    if let Err(e) = mmu.init_identity_mapping() {
        panic!("Failed to initialize MMU: {}", e);
    }
    
    // Enable MMU
    mmu.enable();
    
    println!("x86-64 MMU system initialized");
}

/// Create default memory attributes for different types
impl Default for MemoryAttributes {
    fn default() -> Self {
        Self {
            device: false,
            cacheable: true,
            write_through: false,
            user_accessible: false,
            executable: true,
            writable: true,
            global: false,
        }
    }
}

/// Memory attributes for device memory
pub const DEVICE_MEMORY: MemoryAttributes = MemoryAttributes {
    device: true,
    cacheable: false,
    write_through: false,
    user_accessible: false,
    executable: false,
    writable: true,
    global: false,
};

/// Memory attributes for kernel code
pub const KERNEL_CODE: MemoryAttributes = MemoryAttributes {
    device: false,
    cacheable: true,
    write_through: false,
    user_accessible: false,
    executable: true,
    writable: false,
    global: false,
};

/// Memory attributes for kernel data
pub const KERNEL_DATA: MemoryAttributes = MemoryAttributes {
    device: false,
    cacheable: true,
    write_through: false,
    user_accessible: false,
    executable: false,
    writable: true,
    global: false,
};

/// Memory attributes for user code
pub const USER_CODE: MemoryAttributes = MemoryAttributes {
    device: false,
    cacheable: true,
    write_through: false,
    user_accessible: true,
    executable: true,
    writable: false,
    global: false,
};

/// Memory attributes for user data
pub const USER_DATA: MemoryAttributes = MemoryAttributes {
    device: false,
    cacheable: true,
    write_through: false,
    user_accessible: true,
    executable: false,
    writable: true,
    global: false,
};

/// Page table utilities
pub mod utils {
    use super::*;
    
    /// Extract PML4 index from virtual address
    #[inline(always)]
    pub fn pml4_index(virt_addr: usize) -> usize {
        (virt_addr >> 39) & 0x1FF
    }
    
    /// Extract PDPT index from virtual address
    #[inline(always)]
    pub fn pdpt_index(virt_addr: usize) -> usize {
        (virt_addr >> 30) & 0x1FF
    }
    
    /// Extract PDT index from virtual address
    #[inline(always)]
    pub fn pdt_index(virt_addr: usize) -> usize {
        (virt_addr >> 21) & 0x1FF
    }
    
    /// Extract PT index from virtual address
    #[inline(always)]
    pub fn pt_index(virt_addr: usize) -> usize {
        (virt_addr >> 12) & 0x1FF
    }
    
    /// Extract page offset from virtual address
    #[inline(always)]
    pub fn page_offset(virt_addr: usize) -> usize {
        virt_addr & PAGE_MASK
    }
    
    /// Align address to page boundary
    #[inline(always)]
    pub fn page_align(addr: usize) -> usize {
        addr & !PAGE_MASK
    }
    
    /// Check if address is page-aligned
    #[inline(always)]
    pub fn is_page_aligned(addr: usize) -> bool {
        (addr & PAGE_MASK) == 0
    }
    
    /// Round up to page boundary
    #[inline(always)]
    pub fn page_align_up(addr: usize) -> usize {
        (addr + PAGE_MASK) & !PAGE_MASK
    }
}

/// Memory mapping utilities
pub mod mapping {
    use super::*;
    
    /// Map a range of pages
    pub fn map_range(mmu: &mut MMU, start_virt: usize, start_phys: usize, size: usize, attributes: MemoryAttributes) -> Result<(), &'static str> {
        if !utils::is_page_aligned(start_virt) || !utils::is_page_aligned(start_phys) {
            return Err("Addresses must be page-aligned");
        }
        
        let page_count = size / PAGE_SIZE;
        
        for i in 0..page_count {
            let virt_addr = start_virt + i * PAGE_SIZE;
            let phys_addr = start_phys + i * PAGE_SIZE;
            
            mmu.map_page(virt_addr, phys_addr, attributes)?;
        }
        
        Ok(())
    }
    
    /// Unmap a range of pages
    pub fn unmap_range(mmu: &mut MMU, start_virt: usize, size: usize) -> Result<(), &'static str> {
        if !utils::is_page_aligned(start_virt) {
            return Err("Address must be page-aligned");
        }
        
        let page_count = size / PAGE_SIZE;
        
        for i in 0..page_count {
            let virt_addr = start_virt + i * PAGE_SIZE;
            mmu.unmap_page(virt_addr)?;
        }
        
        Ok(())
    }
    
    /// Change page attributes for a range
    pub fn change_attributes(mmu: &mut MMU, start_virt: usize, size: usize, new_attributes: MemoryAttributes) -> Result<(), &'static str> {
        // Phase 1: Dummy implementation
        // Phase 2: Walk page tables and update attributes
        
        println!("Changing attributes for 0x{:x} - 0x{:x}", start_virt, start_virt + size);
        
        Ok(())
    }
}
