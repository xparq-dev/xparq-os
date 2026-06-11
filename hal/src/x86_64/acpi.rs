// XPARQ OS - x86-64 ACPI (Advanced Configuration and Power Interface)
// ACPI driver

use core::ptr::read_volatile;

/// ACPI RSDP (Root System Description Pointer) structure
#[repr(C, packed)]
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

impl Rsdp {
    unsafe fn is_valid(&self) -> bool {
        if &self.signature != b"RSD PTR " {
            return false;
        }

        // Verify first checksum (for rev 0 and up)
        let mut sum: u8 = 0;
        let ptr = self as *const u8;
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

/// RSDT (Root System Description Table)
struct Rsdt {
    header: AcpiHeader,
    // Followed by array of u32 addresses
}

/// XSDT (Extended Root System Description Table)
struct Xsdt {
    header: AcpiHeader,
    // Followed by array of u64 addresses
}

/// Initialize ACPI subsystem
pub fn init() -> Result<(), ()> {
    unsafe {
        if let Some(rsdp) = find_rsdp() {
            // TODO: parse RSDT/XSDT, find important tables (FACP, MADT, etc.)
        }
    }
    Ok(())
}
