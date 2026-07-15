// XPARQ OS - x86-64 ACPI (Advanced Configuration and Power Interface)
// ACPI driver

use core::ptr::read_volatile;
use arrayvec::ArrayVec;

/// ACPI RSDP (Root System Description Pointer) structure
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct Rsdp {
    signature: [u8; 8],       // "RSD PTR "
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,       // valid if revision == 0
    length: u32,             // valid if revision >= 2
    xsdt_address: u64,       // valid if revision >= 2
    extended_checksum: u8,
    reserved: [u8; 3],
}

/// ACPI common header for all tables
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct AcpiHeader {
    signature: [u8; 4],    // e.g., "RSDT", "XSDT", "FACP", "MADT"
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    asl_compiler_id: [u8; 4],
    asl_compiler_revision: u32,
}

impl AcpiHeader {
    unsafe fn is_valid(&self, expected_signature: &[u8; 4]) -> bool {
        if &self.signature != expected_signature {
            return false;
        }

        // Verify checksum
        let mut sum: u8 = 0;
        let ptr = self as *const AcpiHeader as *const u8;
        for i in 0..self.length as usize {
            sum = sum.wrapping_add(*ptr.add(i));
        }

        sum == 0
    }
}

impl Rsdp {
    unsafe fn is_valid(&self) -> bool {
        if &self.signature != b"RSD PTR " {
            return false;
        }

        // Verify first checksum (for rev 0 and up)
        let mut sum: u8 = 0;
        let ptr = self as *const Rsdp as *const u8;
        let first_part_len = if self.revision == 0 { 20 } else { 36 };
        for i in 0..first_part_len {
            sum = sum.wrapping_add(*ptr.add(i));
        }
        if sum != 0 {
            return false;
        }

        // Verify extended checksum if revision >= 2
        if self.revision >= 2 {
            let mut ext_sum: u8 = 0;
            for i in 0..self.length as usize {
                ext_sum = ext_sum.wrapping_add(*ptr.add(i));
            }
            if ext_sum != 0 {
                return false;
            }
        }

        true
    }
}

/// Find RSDP in BIOS area
pub unsafe fn find_rsdp() -> Option<&'static Rsdp> {
    // Search 0x000E_0000 to 0x000F_FFFF (128 KB)
    let start = 0xE0000;
    let end = 0x100000;
    let mut addr = start;

    while addr < end {
        let rsdp = &*(addr as *const Rsdp);
        if rsdp.is_valid() {
            return Some(rsdp);
        }
        addr += 16; // RSDP is always 16‑byte aligned
    }

    None
}

/// Find table with specific signature via RSDT/XSDT
pub unsafe fn find_table(rsdp: &Rsdp, signature: &[u8; 4]) -> Option<u64> {
    if rsdp.revision >= 2 {
        // Use XSDT
        let xsdt_ptr = rsdp.xsdt_address as *const AcpiHeader;
        let xsdt = &*xsdt_ptr;
        if xsdt.is_valid(b"XSDT") {
            let entry_count = (xsdt.length as usize - core::mem::size_of::<AcpiHeader>()) / 8;
            let entries_ptr = (rsdp.xsdt_address as usize + core::mem::size_of::<AcpiHeader>()) as *const u64;
            for i in 0..entry_count {
                let table_addr = *entries_ptr.add(i);
                let header = &*(table_addr as *const AcpiHeader);
                if header.is_valid(signature) {
                    return Some(table_addr);
                }
            }
        }
    } else {
        // Use RSDT
        let rsdt_ptr = rsdp.rsdt_address as u64 as *const AcpiHeader;
        let rsdt = &*rsdt_ptr;
        if rsdt.is_valid(b"RSDT") {
            let entry_count = (rsdt.length as usize - core::mem::size_of::<AcpiHeader>()) / 4;
            let entries_ptr = (rsdp.rsdt_address as usize + core::mem::size_of::<AcpiHeader>()) as *const u32;
            for i in 0..entry_count {
                let table_addr = *entries_ptr.add(i) as u64;
                let header = &*(table_addr as *const AcpiHeader);
                if header.is_valid(signature) {
                    return Some(table_addr);
                }
            }
        }
    }
    None
}

/// MADT (Multiple APIC Description Table)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct Madt {
    header: AcpiHeader,
    local_apic_address: u32,
    flags: u32,
    // Followed by variable-length MADT structures
}

/// MADT structure types
const MADT_TYPE_LAPIC: u8 = 0;     // Processor Local APIC
const MADT_TYPE_IOAPIC: u8 = 1;    // I/O APIC
const MADT_TYPE_INT_OVERRIDE: u8 = 2; // Interrupt Source Override

/// Processor Local APIC MADT entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MadtEntryLapic {
    pub type_: u8,
    pub length: u8,
    pub processor_id: u8,
    pub apic_id: u8,
    pub flags: u32,
}

/// I/O APIC MADT entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MadtEntryIoapic {
    pub type_: u8,
    pub length: u8,
    pub ioapic_id: u8,
    pub reserved: u8,
    pub ioapic_address: u32,
    pub global_system_interrupt_base: u32,
}

/// Interrupt Source Override MADT entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MadtEntryIntOverride {
    pub type_: u8,
    pub length: u8,
    pub bus: u8,
    pub source: u8,
    pub global_system_interrupt: u32,
    pub flags: u16,
}

/// Parsed MADT information
#[derive(Debug, Clone, Default)]
pub struct MadtInfo {
    pub local_apic_address: u32,
    pub lapics: ArrayVec<MadtEntryLapic, 32>,
    pub ioapics: ArrayVec<MadtEntryIoapic, 8>,
    pub int_overrides: ArrayVec<MadtEntryIntOverride, 32>,
}

pub unsafe fn parse_madt(madt_addr: u64) -> Result<MadtInfo, ()> {
    let header_ptr = madt_addr as *const AcpiHeader;
    let header = &*header_ptr;

    if !header.is_valid(b"APIC") {
        return Err(());
    }

    let madt_ptr = madt_addr as *const Madt;
    let madt = &*madt_ptr;

    let mut info = MadtInfo {
        local_apic_address: madt.local_apic_address,
        lapics: ArrayVec::new(),
        ioapics: ArrayVec::new(),
        int_overrides: ArrayVec::new(),
    };

    // Parse MADT entries
    let mut offset = core::mem::size_of::<Madt>();
    let madt_len = header.length as usize;

    while offset < madt_len {
        let entry_ptr = (madt_addr as usize + offset) as *const u8;
        let type_ = *entry_ptr;
        let length = *entry_ptr.add(1) as usize;

        match type_ {
            MADT_TYPE_LAPIC => {
                let lapic = &*(entry_ptr as *const MadtEntryLapic);
                if (lapic.flags & 0x01) != 0 {
                    let _ = info.lapics.try_push(*lapic);
                }
            }
            MADT_TYPE_IOAPIC => {
                let ioapic = &*(entry_ptr as *const MadtEntryIoapic);
                let _ = info.ioapics.try_push(*ioapic);
            }
            MADT_TYPE_INT_OVERRIDE => {
                let int_override = &*(entry_ptr as *const MadtEntryIntOverride);
                let _ = info.int_overrides.try_push(*int_override);
            }
            _ => {}
        }

        offset += length;
    }

    Ok(info)
}
/// FADT (Fixed ACPI Description Table)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Fadt {
    pub header: AcpiHeader,
    pub firmware_ctrl: u32,
    pub dsdt: u32,
    _reserved1: u8,
    pub preferred_power_management_profile: u8,
    pub sci_interrupt: u16,
    pub smi_command_port: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_control: u8,
    pub pm1a_event_block: u32,
    pub pm1b_event_block: u32,
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pub pm2_control_block: u32,
    pub pm_timer_block: u32,
    pub gpe0_block: u32,
    pub gpe1_block: u32,
    pub pm1_event_length: u8,
    pub pm1_control_length: u8,
    pub pm2_control_length: u8,
    pub pm_timer_length: u8,
    pub gpe0_length: u8,
    pub gpe1_length: u8,
    pub gpe1_base: u8,
    pub cstate_control: u8,
    pub worst_c2_latency: u16,
    pub worst_c3_latency: u16,
    pub flush_size: u16,
    pub flush_stride: u16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alarm: u8,
    pub month_alarm: u8,
    pub century: u8,
    pub boot_architecture_flags: u16,
    _reserved2: u8,
    pub flags: u32,
}

pub unsafe fn parse_fadt(fadt_addr: u64) -> Result<u32, ()> {
    let header_ptr = fadt_addr as *const AcpiHeader;
    let header = &*header_ptr;

    if !header.is_valid(b"FACP") {
        return Err(());
    }

    let fadt_ptr = fadt_addr as *const Fadt;
    let fadt = &*fadt_ptr;

    Ok(fadt.pm1a_control_block)
}

/// ACPI global state
pub static mut ACPI_STATE: AcpiState = AcpiState::new();

#[derive(Debug, Clone, Default)]
pub struct AcpiState {
    pub initialized: bool,
    pub madt: Option<MadtInfo>,
    pub pm1a_control_block: Option<u32>,
}

impl AcpiState {
    const fn new() -> Self {
        Self {
            initialized: false,
            madt: None,
            pm1a_control_block: None,
        }
    }
}

/// Initialize ACPI subsystem
pub fn init() -> Result<(), ()> {
    unsafe {
        if let Some(rsdp) = find_rsdp() {
            // Find MADT
            if let Some(madt_addr) = find_table(rsdp, b"APIC") {
                if let Ok(madt_info) = parse_madt(madt_addr) {
                    ACPI_STATE.madt = Some(madt_info);
                }
            }
            // Find FADT
            if let Some(fadt_addr) = find_table(rsdp, b"FACP") {
                if let Ok(pm1a_cnt) = parse_fadt(fadt_addr) {
                    ACPI_STATE.pm1a_control_block = Some(pm1a_cnt);
                }
            }
            ACPI_STATE.initialized = true;
        }
    }
    Ok(())
}
