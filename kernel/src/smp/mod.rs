// XPARQ OS - Phase 12: SMP Boot Flow
// AP Startup logic and initialization

use crate::hal::x86_64::apic::LOCAL_APIC;
use crate::hal::x86_64::apic::sleep_ms;
use core::arch::global_asm;

// We will embed a small 16-bit real mode trampoline that the AP starts at.
// The trampoline transitions from Real Mode -> 32-bit Protected Mode -> 64-bit Long Mode
// and then jumps to the `ap_entry` Rust function.
global_asm!(include_str!("trampoline.asm"));

extern "C" {
    // These symbols are provided by trampoline.asm
    static TRAMPOLINE_START: u8;
    static TRAMPOLINE_END: u8;
    static mut AP_STACK_PTR: u64;
    static mut AP_PAGE_TABLE: u64;
    static mut AP_CODE_SEG: u64;
    static mut AP_DATA_SEG: u64;
    static mut AP_READY_FLAG: u32;
}

static mut AP_STACKS: [[u8; 4096]; 8] = [[0; 4096]; 8];
static mut AP_IDLE_STACKS: [[u8; 4096]; 8] = [[0; 4096]; 8];

/// The physical address where the trampoline must be loaded.
/// We usually pick 0x8000 (a free memory area in real mode below 1MB).
pub const TRAMPOLINE_PHYS_ADDR: u64 = 0x8000;

/// Starts all Application Processors (APs) found in the system.
pub fn start_aps() {
    unsafe {
        let acpi_state = &*(&raw const crate::hal::x86_64::acpi::ACPI_STATE);
        if !acpi_state.initialized {
            return;
        }

        if let Some(madt) = &acpi_state.madt {
            if let Some(lapic) = &*(&raw const LOCAL_APIC) {
                let bsp_id = lapic.id();

                // 1. Copy trampoline code to the low memory physical address
                let trampoline_len = (&TRAMPOLINE_END as *const u8 as usize) - (&TRAMPOLINE_START as *const u8 as usize);
                let dest = TRAMPOLINE_PHYS_ADDR as *mut u8;
                core::ptr::copy_nonoverlapping(&TRAMPOLINE_START as *const u8, dest, trampoline_len);

                // Setup shared arguments for APs (Page Tables, Segments)
                let cr3 = crate::hal::x86_64::paging::get_cr3();
                core::ptr::write_volatile(&raw mut AP_PAGE_TABLE, cr3);

                for apic_id_entry in &madt.lapics {
                    let target_apic_id = apic_id_entry.apic_id as u32;
                    if target_apic_id == bsp_id {
                        continue; // Don't start the BSP
                    }

                    // Pre-allocate a kernel stack for this AP
                    // In a real implementation we would allocate this dynamically,
                    // but for Phase 12 we can use static arrays.
                    // Let's pass the pre-allocated stack pointer.
                    let ap_stack = (&raw const AP_STACKS[target_apic_id as usize]) as u64 + 4096;
                    core::ptr::write_volatile(&raw mut AP_STACK_PTR, ap_stack);
                    core::ptr::write_volatile(&raw mut AP_READY_FLAG, 0);

                    // 2. Send INIT IPI
                    // Target, Delivery Mode (INIT = 5), Assert, Level, Vector = 0
                    // Vector must be 0 for INIT
                    lapic.send_ipi(target_apic_id, 0); 
                    
                    sleep_ms(10); // Wait 10ms

                    // 3. Send SIPI
                    // The vector is the page number of the trampoline address
                    let vector = (TRAMPOLINE_PHYS_ADDR >> 12) as u8;
                    // Delivery Mode (SIPI = 6)
                    lapic.send_ipi(target_apic_id, vector);

                    // Wait for AP to set the ready flag
                    let mut timeout = 0;
                    while core::ptr::read_volatile(&raw const AP_READY_FLAG) == 0 {
                        sleep_ms(1);
                        timeout += 1;
                        if timeout > 100 { // 100ms timeout
                            break;
                        }
                    }

                    if core::ptr::read_volatile(&raw const AP_READY_FLAG) == 1 {
                        // Success
                    } else {
                        // Failed
                    }
                }
            }
        }
    }
}

/// The entry point for Application Processors in 64-bit Long Mode.
/// The trampoline jumps here after setting up the stack and page tables.
#[no_mangle]
pub extern "C" fn ap_entry() -> ! {
    unsafe {
        // Signal BSP that we have successfully started
        core::ptr::write_volatile(&raw mut AP_READY_FLAG, 1);
    }
    
    let lapic_addr = 0xFEE00000; // Need to read from MADT really, but assume standard
    let lapic = crate::hal::x86_64::apic::LocalApic::init(lapic_addr);
    
    // Enable interrupt handling for this CPU
    crate::hal::x86_64::idt::init();
    
    // Setup GDT and TSS for this CPU
    crate::hal::x86_64::gdt::init();

    lapic.init_timer();

    let apic_id = lapic.id() as usize;
    let mut manager = crate::task::TASK_MANAGER.lock();
    let idle_stack = unsafe { (&raw const AP_IDLE_STACKS[apic_id]) as u64 };
    let idle_id = manager.spawn_task(crate::task::idle::idle_task_entry, idle_stack, 4096).unwrap();
    
    let mut cpu = crate::cpu::CPUS[apic_id].lock();
    cpu.scheduler.idle_task = Some(idle_id);
    cpu.scheduler.ready_queue.remove(&mut manager.pool, idle_id);
    cpu.current_task = Some(idle_id);
    drop(cpu);
    
    let mut dummy_ptr = 0u64;
    let next_sp = manager.schedule_next_for_cpu(apic_id);
    drop(manager);
    
    unsafe {
        crate::task::switch::switch_context(&mut dummy_ptr, next_sp);
    }
    
    unreachable!();
}
