// XPARQ OS - x86-64 Local and I/O APIC Driver

use crate::x86_64::acpi;
use core::ptr::{read_volatile, write_volatile};
use arrayvec::ArrayVec;

/// Local APIC register offsets
const LAPIC_ID: usize = 0x0020;
const LAPIC_SPURIOUS: usize = 0x00F0;
const LAPIC_EOI: usize = 0x00B0;

/// I/O APIC register offsets
const IOAPIC_IDX: usize = 0x0000;
const IOAPIC_WIN: usize = 0x0010;
const IOAPIC_REG_VER: usize = 0x01;
const IOAPIC_REG_TABLE_LOW: usize = 0x10;

/// Local APIC state
#[derive(Debug, Clone, Copy)]
pub struct LocalApic {
    address: u64,
}

impl LocalApic {
    /// Initialize Local APIC
    pub fn init(address: u64) -> Self {
        let apic = Self { address };

        // Enable Local APIC and set spurious interrupt vector (255)
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

    /// Send End of Interrupt (EOI)
    pub fn eoi(&self) {
        unsafe { self.write_register(LAPIC_EOI, 0); }
    }
}

/// I/O APIC state
#[derive(Debug, Clone, Copy)]
pub struct IoApic {
    address: u64,
    gsi_base: u32,
    gsi_count: u32,
    id: u8,
}

impl IoApic {
    /// Create new I/O APIC
    pub fn new(address: u64, gsi_base: u32, id: u8) -> Self {
        // Read I/O APIC version register to find out max redirection entries
        let mut ioapic = Self {
            address,
            gsi_base,
            gsi_count: 24, // Default: 24 redirection entries
            id,
        };
        ioapic.gsi_count = ioapic.max_redirects();
        ioapic
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

    /// Check if this I/O APIC handles a particular GSI
    pub fn handles_gsi(&self, gsi: u32) -> bool {
        gsi >= self.gsi_base && gsi < (self.gsi_base + self.gsi_count)
    }

    /// Configure redirection entry for a GSI
    pub fn configure_gsi(&self, gsi: u32, vector: u8, dest_lapic_id: u8, flags: u16) {
        if !self.handles_gsi(gsi) {
            return;
        }

        let entry_index = (gsi - self.gsi_base) * 2;
        let low_reg = IOAPIC_REG_TABLE_LOW + entry_index;
        let high_reg = low_reg + 1;

        // Configure high DWORD: destination APIC ID (bits 56-63)
        unsafe {
            self.write_register(high_reg, (dest_lapic_id as u32) << 24);
        }

        // Configure low DWORD: vector, delivery mode, dest mode, polarity, trigger mode, mask
        let mut low = vector as u32;
        low |= 0 << 8; // Delivery mode: fixed (0b000)
        low |= 0 << 11; // Destination mode: physical (0)
        // Polarity: flags bit 0 (0=active high, 1=active low)
        if (flags & 0x0002) != 0 {
            low |= 1 << 13; // Active low
        }
        // Trigger mode: flags bit 1 (0=edge, 1=level)
        if (flags & 0x0008) != 0 {
            low |= 1 << 15; // Level triggered
        }
        // Unmask interrupt (clear bit 16)
        low &= !(1 << 16);

        unsafe {
            self.write_register(low_reg, low);
        }
    }
}

/// Global APIC state
pub static mut LOCAL_APIC: Option<LocalApic> = None;
pub static mut IO_APICS: [Option<IoApic>; 8] = [None; 8];
pub static mut IRQ_TO_GSI: [Option<u32>; 16] = [None; 16]; // IRQ 0-15 map to GSI
pub static mut GSI_TO_IRQ_FLAGS: [Option<u16>; 256] = [None; 256]; // Flags for each GSI (polarity/trigger)

/// Initialize interrupt routing with APIC
pub fn init_interrupt_routing() -> Result<(), ()> {
    unsafe {
        // Set up default IRQ → GSI mappings (1:1 for legacy IRQs)
        for irq in 0..16 {
            IRQ_TO_GSI[irq] = Some(irq as u32);
            GSI_TO_IRQ_FLAGS[irq as usize] = Some(0); // Default: active high, edge triggered
        }

        // Apply any MADT interrupt source overrides
        let acpi_state = &acpi::ACPI_STATE;
        if acpi_state.initialized {
            if let Some(madt_info) = &acpi_state.madt {
                for override_entry in &madt_info.int_overrides {
                    let irq = override_entry.source as usize;
                    if irq < 16 {
                        IRQ_TO_GSI[irq] = Some(override_entry.global_system_interrupt);
                        GSI_TO_IRQ_FLAGS[override_entry.global_system_interrupt as usize] = Some(override_entry.flags);
                    }
                }
            }
        }

        Ok(())
    }
}

/// Map an IRQ to an interrupt vector
pub fn map_irq(irq: u8, vector: u8) -> Result<(), ()> {
    unsafe {
        let irq_usize = irq as usize;
        if irq_usize > 15 {
            return Err(());
        }

        // Find the corresponding GSI for this IRQ
        let gsi = match IRQ_TO_GSI[irq_usize] {
            Some(gsi) => gsi,
            None => irq as u32,
        };

        let flags = match GSI_TO_IRQ_FLAGS[gsi as usize] {
            Some(flags) => flags,
            None => 0,
        };

        // Get the Local APIC ID
        let lapic_id = match &LOCAL_APIC {
            Some(lapic) => lapic.id() as u8,
            None => 0,
        };

        // Find the I/O APIC that handles this GSI and configure it
        for ioapic in IO_APICS.iter().flatten() {
            if ioapic.handles_gsi(gsi) {
                ioapic.configure_gsi(gsi, vector, lapic_id, flags);
                return Ok(());
            }
        }

        Err(())
    }
}

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

            // Set up IRQ → GSI routing
            init_interrupt_routing()?;
        } else {
            return Err(());
        }
    }
    Ok(())
}
