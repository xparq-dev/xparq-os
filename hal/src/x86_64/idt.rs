// XPARQ OS - x86-64 Interrupt Descriptor Table (IDT)

use core::mem;
use core::ptr::write_volatile;

/// IDT entry flags
bitflags::bitflags! {
    pub struct IdtFlags: u8 {
        const PRESENT = 1 << 7;
        const RING_0 = 0 << 5;
        const INTERRUPT_GATE = 0b1110;
    }
}

/// IDT entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    flags: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    /// Create an empty IDT entry
    pub const fn empty() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            flags: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    /// Set handler address
    pub fn set_handler(&mut self, handler: u64, flags: IdtFlags) {
        self.selector = 0x08; // kernel code segment selector
        self.flags = flags.bits();
        self.offset_low = (handler & 0xFFFF) as u16;
        self.offset_mid = ((handler >> 16) & 0xFFFF) as u16;
        self.offset_high = (handler >> 32) as u32;
    }
}

/// IDT pointer (for lidt)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

/// IDT (256 entries)
static mut IDT: [IdtEntry; 256] = [IdtEntry::empty(); 256];

/// Initialize the IDT
pub fn init() {
    let idt_ptr = IdtPointer {
        limit: (mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
        base: &IDT as *const _ as u64,
    };

    unsafe {
        // Load the IDT
        asm!("lidt [{}]", in(reg) &idt_ptr, options(nostack));

        // Enable interrupts later once we have handlers setup
    }
}
