// XPARQ OS - Phase 7: Hardware Paging Structures
// x86_64 4-Level Paging

use core::arch::asm;
use bitflags::bitflags;

bitflags! {
    /// Page Table Entry flags
    #[derive(Clone, Copy)]
    pub struct PageTableFlags: u64 {
        const PRESENT =         1 << 0;
        const WRITABLE =        1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGE_PAGE =       1 << 7;
        const GLOBAL =          1 << 8;
        const NO_EXECUTE =      1 << 63;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn set_addr(&mut self, addr: u64, flags: PageTableFlags) {
        // Address must be 4KB aligned
        debug_assert!(addr & 0xFFF == 0);
        self.0 = addr | flags.bits();
    }

    pub fn addr(&self) -> u64 {
        self.0 & 0x000F_FFFF_FFFF_F000
    }

    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.0)
    }

    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }
}

/// A Page Table (PML4, PDPT, PD, PT) containing 512 entries
#[repr(align(4096))]
#[derive(Debug, Clone, Copy)]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); 512],
        }
    }
}

/// Gets the physical address of the current PML4 table from CR3
pub fn get_cr3() -> u64 {
    let value: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) value);
    }
    value
}

/// Sets the CR3 register to point to a new PML4 table
pub fn set_cr3(pml4_addr: u64) {
    unsafe {
        asm!("mov cr3, {}", in(reg) pml4_addr);
    }
}

/// Flushes a single virtual address from the TLB
pub fn flush_tlb(vaddr: u64) {
    unsafe {
        asm!("invlpg [{}]", in(reg) vaddr);
    }
}

/// Helper functions to extract indices for 4-level paging
pub fn p4_index(vaddr: u64) -> usize {
    ((vaddr >> 39) & 0x1FF) as usize
}

pub fn p3_index(vaddr: u64) -> usize {
    ((vaddr >> 30) & 0x1FF) as usize
}

pub fn p2_index(vaddr: u64) -> usize {
    ((vaddr >> 21) & 0x1FF) as usize
}

pub fn p1_index(vaddr: u64) -> usize {
    ((vaddr >> 12) & 0x1FF) as usize
}
