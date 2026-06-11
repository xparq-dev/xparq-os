// XPARQ OS - x86-64 8259 PIC driver
// Used only to disable the PIC completely when switching to APIC

use core::ptr::write_volatile;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

/// Initialize and disable both PICs
pub fn disable_pic() {
    // Mask all interrupts first
    unsafe {
        write_volatile(PIC1_DATA as *mut u8, 0xFF);
        write_volatile(PIC2_DATA as *mut u8, 0xFF);

        // Send initialization command (ICW1 - start initialization sequence)
        write_volatile(PIC1_COMMAND as *mut u8, 0x11);
        write_volatile(PIC2_COMMAND as *mut u8, 0x11);

        // ICW2 - map IRQs to vectors (we don't care, just any offset)
        write_volatile(PIC1_DATA as *mut u8, 0x20); // IRQ0 → 32
        write_volatile(PIC2_DATA as *mut u8, 0x28); // IRQ8 → 40

        // ICW3 - configure PIC2 at IRQ2 of PIC1
        write_volatile(PIC1_DATA as *mut u8, 0x04);
        write_volatile(PIC2_DATA as *mut u8, 0x02);

        // ICW4 - 8086/88 mode, no special features
        write_volatile(PIC1_DATA as *mut u8, 0x01);
        write_volatile(PIC2_DATA as *mut u8, 0x01);

        // Mask all interrupts again (to fully disable them)
        write_volatile(PIC1_DATA as *mut u8, 0xFF);
        write_volatile(PIC2_DATA as *mut u8, 0xFF);
    }
}
