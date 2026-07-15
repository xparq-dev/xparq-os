// XPARQ OS - x86-64 8259 PIC driver
// Used only to disable the PIC completely when switching to APIC

use core::ptr::write_volatile;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

/// Initialize and disable both PICs
pub fn disable_pic() {
    unsafe {
        // Mask all interrupts
        core::arch::asm!(
            "out dx, al",
            in("dx") PIC1_DATA,
            in("al") 0xFFu8,
            options(nomem, nostack)
        );
        core::arch::asm!(
            "out dx, al",
            in("dx") PIC2_DATA,
            in("al") 0xFFu8,
            options(nomem, nostack)
        );

        // Send initialization command (ICW1 - start initialization sequence)
        core::arch::asm!("out dx, al", in("dx") PIC1_COMMAND, in("al") 0x11u8, options(nomem, nostack));
        core::arch::asm!("out dx, al", in("dx") PIC2_COMMAND, in("al") 0x11u8, options(nomem, nostack));

        // ICW2 - map IRQs to vectors (we don't care, just any offset)
        core::arch::asm!("out dx, al", in("dx") PIC1_DATA, in("al") 0x20u8, options(nomem, nostack)); // IRQ0 → 32
        core::arch::asm!("out dx, al", in("dx") PIC2_DATA, in("al") 0x28u8, options(nomem, nostack)); // IRQ8 → 40

        // ICW3 - configure PIC2 at IRQ2 of PIC1
        core::arch::asm!("out dx, al", in("dx") PIC1_DATA, in("al") 0x04u8, options(nomem, nostack));
        core::arch::asm!("out dx, al", in("dx") PIC2_DATA, in("al") 0x02u8, options(nomem, nostack));

        // ICW4 - 8086 mode
        core::arch::asm!("out dx, al", in("dx") PIC1_DATA, in("al") 0x01u8, options(nomem, nostack));
        core::arch::asm!("out dx, al", in("dx") PIC2_DATA, in("al") 0x01u8, options(nomem, nostack));

        // Restore masks (mask all)
        core::arch::asm!("out dx, al", in("dx") PIC1_DATA, in("al") 0xFFu8, options(nomem, nostack));
        core::arch::asm!("out dx, al", in("dx") PIC2_DATA, in("al") 0xFFu8, options(nomem, nostack));
    }
}

