// XPARQ OS - x86-64 Local and I/O APIC Driver

use crate::x86_64::acpi;
use core::ptr::{read_volatile, write_volatile};

/// Local APIC register offsets
const LAPIC_ID: usize = 0x0020;
const LAPIC_SPURIOUS: usize = 0x00F0;
const LAPIC_EOI: usize = 0x00B0;

/// Local APIC state
#[derive(Debug, Clone, Copy)]
pub struct LocalApic {
    address: u64,
}

impl LocalApic {
    /// Initialize Local APIC
    pub fn init(address: u64) -> Self {
        let apic = Self { address };

        // Enable Local APIC and set spurious interrupt vector
        let spurious = (1 << 8) | 0xFF; // bit 8: enable, spurious vector 255
        unsafe { apic.write_register(LAPIC_SPURIOUS, spurious as u32); }

        apic
    }

    /// Read Local APIC register
    unsafe fn read_register(&self, offset: usize) -> u32 {
        let ptr = (self.address as usize + offset) as *const u32;
        read_volatile(ptr)
    }

    /// Write Local APIC register
    unsafe fn write_register(&self, offset: usize, value: u32) {
        let ptr = (self.address as usize + offset) as *mut u32;
        write_volatile(ptr, value);
    }

    /// Get Local APIC ID
    pub fn id(&self) -> u32 {
        unsafe { (self.read_register(LAPIC_ID) >> 24) & 0xFF }
    }

    /// Send End of Interrupt
    pub fn eoi(&self) {
        unsafe { self.write_register(LAPIC_EOI, 0); }
    }
}

/// I/O APIC register offsets
const IOAPIC_IDX: usize = 0x0000;
const IOAPIC_WIN: usize = 0x0010;
const IOAPIC_REG_VER: usize = 0x01;
const IOAPIC_REG_TABLE_LOW: usize = 0x10;

/// I/O APIC state
#[derive(Debug, Clone, Copy)]
pub struct IoApic {
    address: u64,
    gsi_base: u32,
    id: u8,
}

impl IoApic {
    /// Create new I/O APIC
    pub fn new(address: u64, gsi_base: u32, id: u8) -> Self {
        Self { address, gsi_base, id }
    }

    /// Read I/O APIC register
    unsafe fn read_register(&self, reg: u32) -> u32 {
        let idx_ptr = (self.address as usize + IOAPIC_IDX) as *mut u32;
        write_volatile(idx_ptr, reg);
        let win_ptr = (self.address as usize + IOAPIC_WIN) as *const u32;
        read_volatile(win_ptr)
    }

    /// Write I/O APIC register
    unsafe fn write_register(&self, reg: u32, value: u32) {
        let idx_ptr = (self.address as usize + IOAPIC_IDX) as *mut u32;
        write_volatile(idx_ptr, reg);
        let win_ptr = (self.address as usize + IOAPIC_WIN) as *mut u32;
        write_volatile(win_ptr, value);
    }

    /// Get maximum redirection entry count
    pub fn max_redirects(&self) -> u32 {
        unsafe {
            let ver = self.read_register(IOAPIC_REG_VER);
            ((ver >> 16) & 0xFF) + 1
        }
    }

    /// Configure redirection entry
    pub fn configure_redirect(&self, gsi: u32, vector: u8, dest_lapic_id: u8) {
        if gsi < self.gsi_base || gsi >= self.gsi_base + self.max_redirects() {
            return;
        }

        let index = (gsi - self.gsi_base) * 2;
        let low_reg = IOAPIC_REG_TABLE_LOW + index;
        let high_reg = low_reg + 1;

        // Configure high DWORD (destination APIC ID)
        unsafe {
            self.write_register(high_reg, (dest_lapic_id as u32) << 24);
        }

        // Configure low DWORD (vector, delivery mode, etc.)
        let mut low = (vector as u32);        // vector
        low |= 0 << 8;                        // delivery mode (fixed)
        low |= 0 << 11;                       // destination mode (physical)
        low &= !(1 << 16);                    // set active high
        low &= !(1 << 17);                    // set edge-triggered
        low &= !(1 << 16);                    // enable

        unsafe {
            self.write_register(low_reg, low);
        }
    }
}

/// Global APIC state
pub static mut LOCAL_APIC: Option<LocalApic> = None;
pub static mut IO_APICS: [Option<IoApic>; 8] = [None; 8];

/// Initialize APIC subsystem
pub fn init() -> Result<(), ()> {
    unsafe {
        let acpi_state = &acpi::ACPI_STATE;
        if !acpi_state.initialized {
            return Err(());
        }
        if let Some(madt) = &acpi_state.madt {
            // Initialize Local APIC
            LOCAL_APIC = Some(LocalApic::init(madt.local_apic_address as u64));

            // Initialize I/O APICs
            for (i, ioapic_entry) in madt.ioapics.iter().enumerate() {
                if i < 8 {
                    IO_APICS[i] = Some(IoApic::new(
                        ioapic_entry.ioapic_address as u64,
                        ioapic_entry.global_system_interrupt_base,
                        ioapic_entry.ioapic_id,
                    ));
                }
            }
        } else {
            return Err(());
        }
    }
    Ok(())
}
