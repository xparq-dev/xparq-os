// XPARQ OS - Phase 01: OS & Kernel Foundations
// ARM64 MMU implementation
// Handles ARM LPAE page tables and virtual memory management

#![no_std]

/// ARM64 MMU constants
pub const PAGE_SIZE: usize = 4096; // 4KB pages
pub const PAGE_SHIFT: usize = 12;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

/// Page table levels for ARM64 LPAE
pub const L0_ENTRIES: usize = 512;
pub const L1_ENTRIES: usize = 512;
pub const L2_ENTRIES: usize = 512;
pub const L3_ENTRIES: usize = 512;

/// Page table entry bits
pub mod pte_bits {
    pub const VALID: u64 = 1 << 0;
    pub const TABLE: u64 = 1 << 1;
    pub const PAGE: u64 = 1 << 1; // Same bit as TABLE, different level
    pub const AF: u64 = 1 << 10; // Access flag
    pub const SH: u64 = 3 << 8; // Shareability bits
    pub const AP: u64 = 3 << 6; // Access permission bits
    pub const NS: u64 = 1 << 5; // Non-secure bit
    pub const UXN: u64 = 1 << 54; // User-mode execute-never
    pub const PXN: u64 = 1 << 53; // Privileged execute-never
    pub const NG: u64 = 1 << 11; // Not-global bit
    pub const CONTIGUOUS: u64 = 1 << 52; // Contiguous hint
    pub const PXN_TABLE: u64 = 1 << 59; // PXN for table entries
    pub const UXN_TABLE: u64 = 1 << 60; // UXN for table entries
}

/// Memory attributes
#[derive(Debug, Clone, Copy)]
pub struct MemoryAttributes {
    pub device: bool,
    pub cacheable: bool,
    pub shareable: bool,
    pub user_accessible: bool,
    pub executable: bool,
    pub writable: bool,
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
        value |= pte_bits::VALID | pte_bits::TABLE;
        
        // Set shareability
        if attributes.shareable {
            value |= 0b11 << 8; // Inner shareable
        }
        
        // Set access permissions
        if !attributes.user_accessible {
            value |= 0b01 << 6; // EL0 access denied
        }
        
        // Set PXN/UXN for table
        if !attributes.executable {
            value |= pte_bits::PXN_TABLE;
        }
        
        Self { value }
    }
    
    /// Create a page entry
    pub fn page(address: usize, attributes: MemoryAttributes) -> Self {
        let mut value = (address as u64) & !0xFFF; // Clear lower 12 bits
        value |= pte_bits::VALID | pte_bits::PAGE;
        
        // Set access flag
        value |= pte_bits::AF;
        
        // Set shareability
        if attributes.shareable {
            value |= 0b11 << 8; // Inner shareable
        } else {
            value |= 0b10 << 8; // Outer shareable
        }
        
        // Set memory attributes
        if attributes.device {
            value |= 0b00 << 2; // Device memory
        } else if attributes.cacheable {
            value |= 0b01 << 2; // Normal memory, write-back cacheable
        } else {
            value |= 0b10 << 2; // Normal memory, non-cacheable
        }
        
        // Set access permissions
        if attributes.user_accessible {
            if attributes.writable {
                value |= 0b11 << 6; // Read/write EL0
            } else {
                value |= 0b01 << 6; // Read-only EL0
            }
        } else {
            if attributes.writable {
                value |= 0b01 << 6; // Read/write EL1 only
            } else {
                value |= 0b11 << 6; // Read-only EL1 only
            }
        }
        
        // Set execute-never bits
        if !attributes.executable {
            value |= pte_bits::UXN | pte_bits::PXN;
        }
        
        Self { value }
    }
    
    /// Check if entry is valid
    pub fn is_valid(&self) -> bool {
        self.value & pte_bits::VALID != 0
    }
    
    /// Check if entry is a table
    pub fn is_table(&self) -> bool {
        self.value & (pte_bits::VALID | pte_bits::TABLE) == (pte_bits::VALID | pte_bits::TABLE)
    }
    
    /// Check if entry is a page
    pub fn is_page(&self) -> bool {
        self.value & (pte_bits::VALID | pte_bits::PAGE) == (pte_bits::VALID | pte_bits::PAGE)
    }
    
    /// Get physical address from entry
    pub fn get_address(&self) -> usize {
        (self.value & !0xFFF) as usize
    }
}

/// Page table
#[derive(Debug)]
pub struct PageTable {
    entries: [PageTableEntry; L0_ENTRIES],
}

impl PageTable {
    /// Create a new empty page table
    pub fn new() -> Self {
        Self {
            entries: [PageTableEntry::invalid(); L0_ENTRIES],
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
    /// Root page table (L0)
    root_table: PageTable,
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
            root_table: PageTable::new(),
            page_tables: spin::Mutex::new(PageTableAllocator {
                tables: arrayvec::ArrayVec::new(),
                next_free: 0,
            }),
        }
    }
    
    /// Initialize MMU with identity mapping
    pub fn init_identity_mapping(&mut self) -> Result<(), &'static str> {
        println!("Initializing ARM64 MMU with identity mapping...");
        
        // Phase 1: Identity map first 1GB
        // Phase 2: Full memory mapping
        
        // Identity map first 1GB (0x00000000 - 0x40000000)
        for i in 0..256 {
            let virt_addr = i * PAGE_SIZE * 512 * 512; // 1GB blocks
            let phys_addr = virt_addr;
            
            let attributes = MemoryAttributes {
                device: false,
                cacheable: true,
                shareable: true,
                user_accessible: false,
                executable: true,
                writable: true,
            };
            
            // Create L0 entry pointing to L1 table
            let l1_table = self.allocate_page_table()?;
            let l0_entry = PageTableEntry::table(l1_table as usize, attributes);
            self.root_table.set_entry(i, l0_entry);
            
            // For Phase 1, we'll skip creating L1/L2/L3 tables
            // Phase 2: Create full 4-level page tables
        }
        
        println!("ARM64 MMU identity mapping initialized");
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
        println!("Enabling ARM64 MMU...");
        
        unsafe {
            // Set TCR_EL1 (Translation Control Register)
            let tcr = (1 << 31) | // Disable translation for TTBR0
                     (16 << 0) | // TTBR1 T0SZ = 16 (48-bit virtual addresses)
                     (0 << 12) | // TTBR1 T1SZ = 0 (not used)
                     (0b00 << 14) | // TTBR1 TG = 4KB granule
                     (0b00 << 22) | // TTBR1 SH = Inner shareable
                     (0b01 << 24) | // TTBR1 ORGN = Write-back cacheable
                     (0b01 << 26) | // TTBR1 IRGN = Write-back cacheable
                     (1 << 23) | // TTBR1 EPD = Disable TTBR0
                     (0 << 7);   // TBI = No tag inclusion
            
            core::arch::asm!("msr TCR_EL1, {}", in(reg) tcr);
            
            // Set MAIR_EL1 (Memory Attribute Indirection Register)
            let mair = (0b00001111 << 0) | // Device-nGnRnE memory
                       (0b11111111 << 8) | // Normal memory, write-back cacheable
                       (0b01000100 << 16); // Normal memory, non-cacheable
            
            core::arch::asm!("msr MAIR_EL1, {}", in(reg) mair);
            
            // Set TTBR1_EL1 (Translation Table Base Register 1)
            let ttbr1 = &self.root_table as *const PageTable as u64;
            core::arch::asm!("msr TTBR1_EL1, {}", in(reg) ttbr1);
            
            // Ensure all memory accesses complete before enabling MMU
            crate::arch::arm64::boot::regs::dsb();
            crate::arch::arm64::boot::regs::isb();
            
            // Enable MMU by setting M bit in SCTLR_EL1
            let mut sctlr: u64;
            core::arch::asm!("mrs {}, SCTLR_EL1", out(reg) sctlr);
            sctlr |= 1; // Set M bit
            core::arch::asm!("msr SCTLR_EL1, {}", in(reg) sctlr);
            
            // Ensure MMU enable takes effect
            crate::arch::arm64::boot::regs::isb();
        }
        
        println!("ARM64 MMU enabled");
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
}

/// Initialize the MMU system
pub fn init() {
    println!("Initializing ARM64 MMU system...");
    
    // Phase 1: Create MMU with identity mapping
    // Phase 2: Full MMU initialization
    
    let mut mmu = MMU::new();
    
    // Initialize identity mapping
    if let Err(e) = mmu.init_identity_mapping() {
        panic!("Failed to initialize MMU: {}", e);
    }
    
    // Enable MMU
    mmu.enable();
    
    println!("ARM64 MMU system initialized");
}

/// Create default memory attributes for different types
impl Default for MemoryAttributes {
    fn default() -> Self {
        Self {
            device: false,
            cacheable: true,
            shareable: true,
            user_accessible: false,
            executable: true,
            writable: true,
        }
    }
}

/// Memory attributes for device memory
pub const DEVICE_MEMORY: MemoryAttributes = MemoryAttributes {
    device: true,
    cacheable: false,
    shareable: true,
    user_accessible: false,
    executable: false,
    writable: true,
};

/// Memory attributes for kernel code
pub const KERNEL_CODE: MemoryAttributes = MemoryAttributes {
    device: false,
    cacheable: true,
    shareable: true,
    user_accessible: false,
    executable: true,
    writable: false,
};

/// Memory attributes for kernel data
pub const KERNEL_DATA: MemoryAttributes = MemoryAttributes {
    device: false,
    cacheable: true,
    shareable: true,
    user_accessible: false,
    executable: false,
    writable: true,
};
