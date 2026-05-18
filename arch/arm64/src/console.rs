//! ARM64 Console - Phase 2: Dev Environment Setup
//! 
//! This module provides ARM64 console output for XPARQ OS, including:
//! - PL011 UART support for early debug output
//! - Serial communication configuration
//! - Kernel message printing
//! - Early console initialization
//! 
//! UART Type: PL011 (PrimeCell UART)
//! Base Address: 0x9000000 (QEMU virt platform)
//! Baud Rate: 115200 (standard)
//! Data Format: 8N1 (8 data bits, no parity, 1 stop bit)
//! 
//! Roadmap Phase: Phase 2 - Dev Environment Setup

/// UART base address
const UART_BASE: usize = 0x9000000;

/// UART register offsets
const UART_DR: usize = 0x00;   // Data register
const UART_FR: usize = 0x18;   // Flag register
const UART_CR: usize = 0x30;   // Control register
const UART_IMSC: usize = 0x38; // Interrupt mask set/clear
const UART_ICR: usize = 0x44;  // Interrupt clear

/// UART flag register bits
const FR_TXFF: u32 = 1 << 5;    // Transmit FIFO full
const FR_RXFE: u32 = 1 << 4;    // Receive FIFO empty

/// Initialize early console
pub fn early_init() {
    // Phase 1: Basic UART initialization
    // Phase 2: Full UART configuration with interrupts
    
    // Disable UART
    unsafe {
        core::ptr::write_volatile((UART_BASE + UART_CR) as *mut u32, 0);
    }
    
    // Configure UART (115200 baud, 8N1)
    // Phase 2: Calculate proper divisor for 115200 baud
    unsafe {
        // Set line control (8N1)
        core::ptr::write_volatile((UART_BASE + 0x2C) as *mut u32, 0x60); // LCR_H
        
        // Set baud rate divisor (assuming 24MHz clock)
        let divisor = 13; // 24MHz / (16 * 115200) = 13.02
        core::ptr::write_volatile((UART_BASE + 0x24) as *mut u32, divisor & 0xFFFF); // IBRD
        core::ptr::write_volatile((UART_BASE + 0x28) as *mut u32, ((divisor >> 16) & 0x0F) << 6); // FBRD
        
        // Enable UART
        core::ptr::write_volatile((UART_BASE + UART_CR) as *mut u32, 0x301); // Enable TX, RX, UART
    }
    
    println!("Early console initialized via PL011 UART");
}

/// Initialize console
pub fn init() {
    println!("Initializing ARM64 console...");
    
    // Phase 2: Enable UART interrupts
    unsafe {
        // Clear all interrupts
        core::ptr::write_volatile((UART_BASE + UART_ICR) as *mut u32, 0x7FF);
        
        // Enable RX interrupt
        core::ptr::write_volatile((UART_BASE + UART_IMSC) as *mut u32, 0x10);
    }
    
    println!("ARM64 console initialized");
}

/// Write string to console
pub fn write_str(s: &str) {
    for byte in s.bytes() {
        write_byte(byte);
    }
}

/// Write byte to console
pub fn write_byte(byte: u8) {
    // Wait for transmit FIFO to not be full
    while unsafe { core::ptr::read_volatile((UART_BASE + UART_FR) as *const u32) } & FR_TXFF != 0 {
        core::hint::spin_loop();
    }
    
    // Write byte to data register
    unsafe {
        core::ptr::write_volatile((UART_BASE + UART_DR) as *mut u32, byte as u32);
    }
}

/// Read byte from console (non-blocking)
pub fn read_byte() -> Option<u8> {
    // Check if receive FIFO is not empty
    if unsafe { core::ptr::read_volatile((UART_BASE + UART_FR) as *const u32) } & FR_RXFE != 0 {
        return None;
    }
    
    // Read byte from data register
    let byte = unsafe { core::ptr::read_volatile((UART_BASE + UART_DR) as *const u32) } as u8;
    Some(byte)
}
