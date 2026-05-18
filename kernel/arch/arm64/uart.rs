// XPARQ OS - Phase 01: OS & Kernel Foundations
// ARM64 PL011 UART implementation
// Provides serial communication for early debugging

#![no_std]

/// PL011 UART registers
#[repr(C)]
pub struct PL011Uart {
    pub data: volatile::Volatile<u32>,        // 0x00
    pub status: volatile::Volatile<u32>,       // 0x04
    pub ctrl: volatile::Volatile<u32>,        // 0x08
    pub _reserved1: [volatile::Volatile<u32>; 1], // 0x0C
    pub low_divider: volatile::Volatile<u32>,  // 0x10
    pub high_divider: volatile::Volatile<u32>, // 0x14
    pub _reserved2: [volatile::Volatile<u32>; 4], // 0x18-0x28
    pub fifo_config: volatile::Volatile<u32>, // 0x2C
    pub _reserved3: [volatile::Volatile<u32>; 2], // 0x30-0x38
    pub modem_ctrl: volatile::Volatile<u32>,  // 0x3C
    pub _reserved4: [volatile::Volatile<u32>; 1], // 0x40
    pub line_ctrl: volatile::Volatile<u32>,   // 0x44
    pub _reserved5: [volatile::Volatile<u32>; 1], // 0x48
    pub control: volatile::Volatile<u32>,     // 0x4C
    pub int_fifo_level_sel: volatile::Volatile<u32>, // 0x50
    pub int_raw: volatile::Volatile<u32>,      // 0x54
    pub int_mask: volatile::Volatile<u32>,     // 0x58
    pub int_clear: volatile::Volatile<u32>,    // 0x5C
    pub dma_ctrl: volatile::Volatile<u32>,     // 0x60
}

/// UART status register bits
pub mod status_bits {
    pub const TX_EMPTY: u32 = 1 << 7;
    pub const TX_FULL: u32 = 1 << 5;
    pub const RX_FULL: u32 = 1 << 4;
    pub const RX_EMPTY: u32 = 1 << 6;
    pub const UART_BUSY: u32 = 1 << 3;
}

/// UART control register bits
pub mod ctrl_bits {
    pub const UART_ENABLE: u32 = 1 << 0;
    pub const SIREN_ENABLE: u32 = 1 << 1;
    pub const SIRLP_ENABLE: u32 = 1 << 2;
    pub const MST_ENABLE: u32 = 1 << 3;
    pub const RTS_ENABLE: u32 = 1 << 14;
    pub const CTS_ENABLE: u32 = 1 << 15;
}

/// UART line control register bits
pub mod line_ctrl_bits {
    pub const BRK: u32 = 1 << 0;
    pub const PEN: u32 = 1 << 1;
    pub const EPS: u32 = 1 << 2;
    pub const STP2: u32 = 1 << 3;
    pub const FEN: u32 = 1 << 4;
    pub const WLEN_5: u32 = 0 << 5;
    pub const WLEN_6: u32 = 1 << 5;
    pub const WLEN_7: u32 = 2 << 5;
    pub const WLEN_8: u32 = 3 << 5;
}

/// UART control register bits
pub mod control_bits {
    pub const RX_ENABLE: u32 = 1 << 9;
    pub const TX_ENABLE: u32 = 1 << 8;
    pub const DTR_ENABLE: u32 = 1 << 0;
    pub const RTS_ENABLE: u32 = 1 << 1;
    pub const OUT1_ENABLE: u32 = 1 << 2;
    pub const OUT2_ENABLE: u32 = 1 << 3;
    pub const RTS_ENABLE: u32 = 1 << 7;
    pub const CTS_ENABLE: u32 = 1 << 8;
}

/// Global UART instance
static mut UART: Option<PL011Uart> = None;
static mut UART_INITIALIZED: bool = false;

/// Initialize the PL011 UART
pub fn init() {
    println!("Initializing PL011 UART...");
    
    // Phase 1: Use standard QEMU PL011 address
    // Phase 2: Get address from device tree
    
    const UART_BASE: usize = 0x9000000;
    
    unsafe {
        // Create UART instance
        UART = Some(&mut *(UART_BASE as *mut PL011Uart));
        
        if let Some(uart) = &mut UART {
            // Disable UART
            uart.ctrl.write(0);
            
            // Wait for UART to be ready
            while uart.status.read() & status_bits::UART_BUSY != 0 {}
            
            // Set baud rate to 115200
            // UART clock is 24MHz in QEMU
            // BAUD_RATE = UART_CLOCK / (16 * DIVISOR)
            // DIVISOR = UART_CLOCK / (16 * BAUD_RATE) = 24000000 / (16 * 115200) = 13.0208
            // Integer part = 13, Fractional part = 0.0208
            // Fractional part * 64 = 1.333, so use 1
            
            uart.high_divider.write(0);
            uart.low_divider.write((13 << 6) | 1);
            
            // Set line control: 8 data bits, 1 stop bit, no parity, FIFO enabled
            uart.line_ctrl.write(line_ctrl_bits::WLEN_8 | line_ctrl_bits::FEN);
            
            // Enable UART, TX, and RX
            uart.ctrl.write(ctrl_bits::UART_ENABLE | ctrl_bits::TX_ENABLE | ctrl_bits::RX_ENABLE);
            
            // Clear FIFOs
            uart.fifo_config.write(1);
            
            UART_INITIALIZED = true;
        }
    }
    
    println!("PL011 UART initialized");
}

/// Write a single character to UART
pub fn write_char(c: u8) {
    unsafe {
        if let Some(uart) = &mut UART {
            // Wait for TX FIFO to have space
            while uart.status.read() & status_bits::TX_FULL != 0 {}
            
            // Write character
            uart.data.write(c as u32);
        }
    }
}

/// Read a single character from UART (non-blocking)
pub fn read_char() -> Option<u8> {
    unsafe {
        if let Some(uart) = &mut UART {
            // Check if RX FIFO has data
            if uart.status.read() & status_bits::RX_EMPTY == 0 {
                Some(uart.data.read() as u8)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Write a string to UART
pub fn write_string(s: &str) {
    for byte in s.bytes() {
        write_char(byte);
    }
}

/// Check if UART is initialized
pub fn is_initialized() -> bool {
    unsafe { UART_INITIALIZED }
}

/// Flush TX buffer
pub fn flush() {
    unsafe {
        if let Some(uart) = &mut UART {
            // Wait for TX FIFO to be empty
            while uart.status.read() & status_bits::TX_EMPTY == 0 {}
        }
    }
}

/// Enable/disable UART interrupts
pub fn set_interrupts(enable: bool) {
    unsafe {
        if let Some(uart) = &mut UART {
            if enable {
                // Enable RX interrupt
                uart.int_mask.write(0);
            } else {
                // Disable all interrupts
                uart.int_mask.write(0xFFFFFFFF);
            }
        }
    }
}

/// Get UART status
pub fn get_status() -> u32 {
    unsafe {
        if let Some(uart) = &mut UART {
            uart.status.read()
        } else {
            0
        }
    }
}

/// UART interrupt handling
pub mod interrupts {
    use super::*;
    
    /// UART interrupt types
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum UartInterrupt {
        Modem,
        TxEmpty,
        Rx,
        RxTimeout,
        Framing,
        Parity,
        Break,
        Overrun,
    }
    
    /// Get pending UART interrupts
    pub fn get_pending_interrupts() -> arrayvec::ArrayVec<UartInterrupt, 8> {
        let mut interrupts = arrayvec::ArrayVec::new();
        
        unsafe {
            if let Some(uart) = &mut UART {
                let raw_ints = uart.int_raw.read();
                
                if raw_ints & (1 << 0) != 0 {
                    interrupts.push(UartInterrupt::Modem);
                }
                if raw_ints & (1 << 1) != 0 {
                    interrupts.push(UartInterrupt::TxEmpty);
                }
                if raw_ints & (1 << 2) != 0 {
                    interrupts.push(UartInterrupt::Rx);
                }
                if raw_ints & (1 << 3) != 0 {
                    interrupts.push(UartInterrupt::RxTimeout);
                }
                if raw_ints & (1 << 4) != 0 {
                    interrupts.push(UartInterrupt::Framing);
                }
                if raw_ints & (1 << 5) != 0 {
                    interrupts.push(UartInterrupt::Parity);
                }
                if raw_ints & (1 << 6) != 0 {
                    interrupts.push(UartInterrupt::Break);
                }
                if raw_ints & (1 << 7) != 0 {
                    interrupts.push(UartInterrupt::Overrun);
                }
            }
        }
        
        interrupts
    }
    
    /// Clear UART interrupt
    pub fn clear_interrupt(interrupt: UartInterrupt) {
        unsafe {
            if let Some(uart) = &mut UART {
                let bit = match interrupt {
                    UartInterrupt::Modem => 1 << 0,
                    UartInterrupt::TxEmpty => 1 << 1,
                    UartInterrupt::Rx => 1 << 2,
                    UartInterrupt::RxTimeout => 1 << 3,
                    UartInterrupt::Framing => 1 << 4,
                    UartInterrupt::Parity => 1 << 5,
                    UartInterrupt::Break => 1 << 6,
                    UartInterrupt::Overrun => 1 << 7,
                };
                
                uart.int_clear.write(bit);
            }
        }
    }
}

/// UART configuration utilities
pub mod config {
    use super::*;
    
    /// Set baud rate
    pub fn set_baud_rate(baud_rate: u32) {
        // UART clock is 24MHz in QEMU
        const UART_CLOCK: u32 = 24_000_000;
        
        let divisor = UART_CLOCK / (16 * baud_rate);
        let fractional = ((UART_CLOCK % (16 * baud_rate)) * 64) / (16 * baud_rate);
        
        unsafe {
            if let Some(uart) = &mut UART {
                uart.high_divider.write(divider >> 6);
                uart.low_divider.write((divisor & 0x3F) << 6 | (fractional & 0x3F));
            }
        }
    }
    
    /// Set data bits
    pub fn set_data_bits(bits: u8) {
        let wlen = match bits {
            5 => line_ctrl_bits::WLEN_5,
            6 => line_ctrl_bits::WLEN_6,
            7 => line_ctrl_bits::WLEN_7,
            8 => line_ctrl_bits::WLEN_8,
            _ => line_ctrl_bits::WLEN_8, // Default to 8 bits
        };
        
        unsafe {
            if let Some(uart) = &mut UART {
                let current = uart.line_ctrl.read();
                uart.line_ctrl.write((current & !0x60) | wlen);
            }
        }
    }
    
    /// Set parity
    pub fn set_parity(parity: Parity) {
        let parity_bits = match parity {
            Parity::None => 0,
            Parity::Even => line_ctrl_bits::PEN,
            Parity::Odd => line_ctrl_bits::PEN | line_ctrl_bits::EPS,
        };
        
        unsafe {
            if let Some(uart) = &mut UART {
                let current = uart.line_ctrl.read();
                uart.line_ctrl.write((current & !(line_ctrl_bits::PEN | line_ctrl_bits::EPS)) | parity_bits);
            }
        }
    }
    
    /// Set stop bits
    pub fn set_stop_bits(stop_bits: StopBits) {
        let stop_bit = match stop_bits {
            StopBits::One => 0,
            StopBits::Two => line_ctrl_bits::STP2,
        };
        
        unsafe {
            if let Some(uart) = &mut UART {
                let current = uart.line_ctrl.read();
                uart.line_ctrl.write((current & !line_ctrl_bits::STP2) | stop_bit);
            }
        }
    }
}

/// Parity settings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Parity {
    None,
    Even,
    Odd,
}

/// Stop bit settings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StopBits {
    One,
    Two,
}

/// Implement the standard print macro for UART output
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut uart_writer = $crate::arch::arm64::uart::UartWriter;
            write!(uart_writer, $($arg)*).unwrap();
        }
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
    };
}

/// UART writer for formatting
pub struct UartWriter;

impl core::fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write_string(s);
        Ok(())
    }
}
