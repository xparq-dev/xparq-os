// XPARQ OS - Phase 7: Physical Frame Allocator
// Static Bump Allocator for 4KB physical frames
// Currently operates in a hardcoded region for bootstrapping (e.g. 16MB to 32MB)

use spin::Mutex;

pub const PAGE_SIZE: u64 = 4096;

pub struct FrameAllocator {
    next_free: u64,
    limit: u64,
}

impl FrameAllocator {
    pub const fn new(start: u64, end: u64) -> Self {
        Self {
            next_free: start,
            limit: end,
        }
    }

    /// Allocates a 4KB physical frame and returns its physical address
    /// The frame is zeroed out before returning
    pub fn allocate_frame(&mut self) -> Option<u64> {
        if self.next_free + PAGE_SIZE <= self.limit {
            let addr = self.next_free;
            self.next_free += PAGE_SIZE;

            // Zero out the frame
            unsafe {
                core::ptr::write_bytes(addr as *mut u8, 0, PAGE_SIZE as usize);
            }

            Some(addr)
        } else {
            None
        }
    }

    pub fn allocate_frames(&mut self, count: u64) -> Option<u64> {
        let size = count * PAGE_SIZE;
        if self.next_free + size <= self.limit {
            let addr = self.next_free;
            self.next_free += size;
            unsafe {
                core::ptr::write_bytes(addr as *mut u8, 0, size as usize);
            }
            Some(addr)
        } else {
            None
        }
    }
}

// We allocate from 16MB to 32MB physical memory for testing page tables
pub static FRAME_ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator::new(0x0100_0000, 0x0200_0000));
