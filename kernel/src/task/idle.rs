// XPARQ OS - Phase 6: Idle Task
// Executes when no other tasks are ready

pub fn idle_task_entry() {
    loop {
        // Halt the CPU until the next interrupt (timer or otherwise)
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
