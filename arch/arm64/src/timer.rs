//! ARM64 Timer - Phase 2: Dev Environment Setup
//! 
//! This module provides ARM64 timer support for XPARQ OS, including:
//! - Generic Timer (ARMv8) configuration
//! - Counter frequency and time keeping
//! - Timer interrupts for scheduling
//! - Per-CPU timer state
//! 
//! Timer Type: ARM Generic Timer (CNTFRQ, CNTVCT, CNTV_CTL)
//! Frequency: Typically 24-50MHz (board-specific)
//! Resolution: Nanosecond precision
//! Interrupt: Virtual Timer (CNTVCT)
//! 
//! Roadmap Phase: Phase 2 - Dev Environment Setup

use super::sysreg;

/// Timer frequency (Hz)
static mut TIMER_FREQUENCY: u64 = 0;

/// Initialize system timer
pub fn init() {
    println!("Initializing ARM64 Generic Timer...");
    
    // Read counter frequency
    let freq = sysreg::mrs("CNTFRQ_EL0");
    unsafe {
        TIMER_FREQUENCY = freq;
    }
    
    println!("Timer frequency: {} Hz", freq);
    
    // Configure virtual timer
    configure_virtual_timer();
}

/// Configure virtual timer
fn configure_virtual_timer() {
    // Disable timer initially
    let mut cntv_ctl = sysreg::mrs("CNTV_CTL_EL0");
    cntv_ctl &= !1; // Clear enable bit
    sysreg::msr("CNTV_CTL_EL0", cntv_ctl);
    
    println!("Virtual timer configured");
}

/// Get current time in nanoseconds
pub fn current_time() -> u64 {
    let freq = unsafe { TIMER_FREQUENCY };
    if freq == 0 {
        return 0;
    }
    
    // Read virtual counter
    let cntvct = sysreg::mrs("CNTVCT_EL0");
    
    // Convert to nanoseconds
    (cntvct * 1_000_000_000) / freq
}

/// Set timer for next interrupt
pub fn set_timer(deadline_ns: u64) {
    let freq = unsafe { TIMER_FREQUENCY };
    if freq == 0 {
        return;
    }
    
    // Convert deadline to counter value
    let deadline_counter = (deadline_ns * freq) / 1_000_000_000;
    
    // Set compare value
    sysreg::msr("CNTV_CVAL_EL0", deadline_counter);
    
    // Enable timer
    let mut cntv_ctl = sysreg::mrs("CNTV_CTL_EL0");
    cntv_ctl |= 1; // Set enable bit
    cntv_ctl &= !2; // Clear interrupt bit
    sysreg::msr("CNTV_CTL_EL0", cntv_ctl);
}

/// Handle timer interrupt
pub fn handle_timer_interrupt() {
    // Clear interrupt flag
    let mut cntv_ctl = sysreg::mrs("CNTV_CTL_EL0");
    cntv_ctl |= 2; // Set interrupt bit to clear
    sysreg::msr("CNTV_CTL_EL0", cntv_ctl);
    
    // Notify scheduler
    crate::scheduler::yield_cpu();
}

/// Get timer frequency
pub fn frequency() -> u64 {
    unsafe { TIMER_FREQUENCY }
}
