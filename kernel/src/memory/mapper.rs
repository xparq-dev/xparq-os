// XPARQ OS - Phase 7: Virtual Memory Mapper
// Maps virtual addresses to physical addresses

use crate::memory::frame::FRAME_ALLOCATOR;
use xparq_hal::x86_64::paging::{PageTable, PageTableFlags, p4_index, p3_index, p2_index, p1_index};

pub struct Mapper {
    pml4_addr: u64,
}

impl Mapper {
    pub const fn new(pml4_addr: u64) -> Self {
        Self { pml4_addr }
    }

    /// Translates a virtual address to a physical address if it's mapped
    pub fn translate(&self, vaddr: u64) -> Option<u64> {
        let pml4 = unsafe { &*(self.pml4_addr as *const PageTable) };
        let p4_entry = &pml4.entries[p4_index(vaddr)];
        if !p4_entry.flags().contains(PageTableFlags::PRESENT) { return None; }

        let pdpt = unsafe { &*(p4_entry.addr() as *const PageTable) };
        let p3_entry = &pdpt.entries[p3_index(vaddr)];
        if !p3_entry.flags().contains(PageTableFlags::PRESENT) { return None; }
        // Note: We don't support 1GB huge pages yet, so we assume it points to PD

        let pd = unsafe { &*(p3_entry.addr() as *const PageTable) };
        let p2_entry = &pd.entries[p2_index(vaddr)];
        if !p2_entry.flags().contains(PageTableFlags::PRESENT) { return None; }
        // Note: We don't support 2MB huge pages yet, so we assume it points to PT

        let pt = unsafe { &*(p2_entry.addr() as *const PageTable) };
        let p1_entry = &pt.entries[p1_index(vaddr)];
        if !p1_entry.flags().contains(PageTableFlags::PRESENT) { return None; }

        Some(p1_entry.addr() | (vaddr & 0xFFF))
    }

    /// Maps a 4KB virtual page to a physical frame
    pub fn map_page(&mut self, vpn: u64, ppn: u64, flags: PageTableFlags) -> Result<(), &'static str> {
        let vaddr = vpn * 4096;
        let paddr = ppn * 4096;

        let pml4 = unsafe { &mut *(self.pml4_addr as *mut PageTable) };
        let p4_entry = &mut pml4.entries[p4_index(vaddr)];
        
        // Allocate PDPT if not present
        if !p4_entry.flags().contains(PageTableFlags::PRESENT) {
            let mut alloc = FRAME_ALLOCATOR.lock();
            let new_frame = alloc.allocate_frame().ok_or("Out of memory for PDPT")?;
            p4_entry.set_addr(new_frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE);
        }

        let pdpt = unsafe { &mut *(p4_entry.addr() as *mut PageTable) };
        let p3_entry = &mut pdpt.entries[p3_index(vaddr)];

        // Allocate PD if not present
        if !p3_entry.flags().contains(PageTableFlags::PRESENT) {
            let mut alloc = FRAME_ALLOCATOR.lock();
            let new_frame = alloc.allocate_frame().ok_or("Out of memory for PD")?;
            p3_entry.set_addr(new_frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE);
        }

        let pd = unsafe { &mut *(p3_entry.addr() as *mut PageTable) };
        let p2_entry = &mut pd.entries[p2_index(vaddr)];

        // Allocate PT if not present
        if !p2_entry.flags().contains(PageTableFlags::PRESENT) {
            let mut alloc = FRAME_ALLOCATOR.lock();
            let new_frame = alloc.allocate_frame().ok_or("Out of memory for PT")?;
            p2_entry.set_addr(new_frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE);
        }

        let pt = unsafe { &mut *(p2_entry.addr() as *mut PageTable) };
        let p1_entry = &mut pt.entries[p1_index(vaddr)];

        // Finally, set the physical address in the Page Table
        p1_entry.set_addr(paddr, flags | PageTableFlags::PRESENT);

        Ok(())
    }

    /// Maps a range of contiguous virtual pages to contiguous physical frames
    pub fn map_range(&mut self, vpn_start: u64, ppn_start: u64, count: u64, flags: PageTableFlags) -> Result<(), &'static str> {
        for i in 0..count {
            self.map_page(vpn_start + i, ppn_start + i, flags)?;
        }
        Ok(())
    }

    pub fn get_pml4_addr(&self) -> u64 {
        self.pml4_addr
    }
}

/// Creates a new Page Table (PML4) for a user process.
/// It copies the higher half mappings (kernel) from the current CR3.
pub fn clone_kernel_pml4() -> Option<u64> {
    let mut alloc = FRAME_ALLOCATOR.lock();
    let new_pml4_addr = alloc.allocate_frame()?;
    drop(alloc);

    unsafe {
        // Zero out the new PML4
        core::ptr::write_bytes(new_pml4_addr as *mut u8, 0, 4096);

        let current_cr3 = xparq_hal::x86_64::paging::get_cr3();
        let current_pml4 = &*(current_cr3 as *const PageTable);
        let new_pml4 = &mut *(new_pml4_addr as *mut PageTable);

        // Copy indices 0..512 because our kernel is identity mapped in the lower half!
        // (In a true higher-half kernel, we would only copy 256..512)
        for i in 0..512 {
            new_pml4.entries[i] = current_pml4.entries[i].clone();
        }
    }

    Some(new_pml4_addr)
}
