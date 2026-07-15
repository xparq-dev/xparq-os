// XPARQ OS - Phase 12: CPU Identification

pub fn current_cpu_id() -> usize {
    // Read the Local APIC ID directly.
    let apic_base = 0xFEE00000 as *mut u32;
    unsafe {
        let id_reg = core::ptr::read_volatile(apic_base.add(0x20 / 4));
        (id_reg >> 24) as usize
    }
}
