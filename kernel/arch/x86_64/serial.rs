// XPARQ OS - Phase 01: OS & Kernel Foundations
// x86-64 COM1 serial port implementation
// Provides serial communication for early debugging

#![no_std]

/// COM1 serial port registers
#[repr(C)]
pub struct COM1Port {
    pub data: volatile::Volatile<u8>,        // 0x00
    pub interrupt_enable: volatile::Volatile<u8>, // 0x01
    pub divisor_low: volatile::Volatile<u8>,  // 0x00 (when DLAB set)
    pub divisor_high: volatile::Volatile<u8>, // 0x01 (when DLAB set)
    pub interrupt_id: volatile::Volatile<u8>, // 0x02
    pub fifo_control: volatile::Volatile<u8>, // 0x02 (when DLAB set)
    pub line_control: volatile::Volatile<u8>, // 0x03
    pub modem_control: volatile::Volatile<u8>, // 0x04
    pub line_status: volatile::Volatile<u8>,   // 0x05
    pub modem_status: volatile::Volatile<u8>, // 0x06
    pub scratch: volatile::Volatile<u8>,      // 0x07
}

/// Line control register bits
pub mod line_ctrl_bits {
    pub const DATA_BITS_5: u8 = 0x00;
    pub const DATA_BITS_6: u8 = 0x01;
    pub const DATA_BITS_7: u8 = 0x02;
    pub const DATA_BITS_8: u8 = 0x03;
    pub const STOP_BITS_1: u8 = 0x00;
    pub const STOP_BITS_2: u8 = 0x04;
    pub const PARITY_NONE: u8 = 0x00;
    pub const PARITY_ODD: u8 = 0x08;
    pub const PARITY_EVEN: u8 = 0x18;
    pub const PARITY_MARK: u8 = 0x28;
    pub const PARITY_SPACE: u8 = 0x38;
    pub const DLAB: u8 = 0x80; // Divisor latch access bit
}

/// Line status register bits
pub mod line_status_bits {
    pub const DATA_READY: u8 = 0x01;
    pub const OVERRUN_ERROR: u8 = 0x02;
    pub const PARITY_ERROR: u8 = 0x04;
    pub const FRAMING_ERROR: u8 = 0x08;
    pub const BREAK_INTERRUPT: u8 = 0x10;
    pub const THR_EMPTY: u8 = 0x20;
    pub const TRANSMITTER_EMPTY: u8 = 0x40;
    pub const ERROR_IN_FIFO: u8 = 0x80;
}

/// FIFO control register bits
pub mod fifo_ctrl_bits {
    pub const ENABLE: u8 = 0x01;
    pub const CLEAR_RECEIVER: u8 = 0x02;
    pub const CLEAR_TRANSMITTER: u8 = 0x04;
    pub const DMA_MODE: u8 = 0x08;
    pub const TRIGGER_1: u8 = 0x00;
    pub const TRIGGER_4: u8 = 0x40;
    pub const TRIGGER_8: u8 = 0x80;
    pub const TRIGGER_14: u8 = 0xC0;
}

/// Modem control register bits
pub mod modem_ctrl_bits {
    pub const DTR: u8 = 0x01;
    pub const RTS: u8 = 0x02;
    pub const OUT1: u8 = 0x04;
    pub const OUT2: u8 = 0x08;
    pub const LOOP: u8 = 0x10;
}

/// Global COM1 port instance
static mut COM1: Option<COM1Port> = None;
static mut COM1_INITIALIZED: bool = false;

/// Initialize COM1 serial port
pub fn init() {
    println!("Initializing COM1 serial port...");
    
    // Phase 1: Use standard COM1 address
    // Phase 2: Get address from BIOS data area
    
    const COM1_BASE: usize = 0x3F8;
    
    unsafe {
        // Create COM1 instance
        COM1 = Some(&mut *(COM1_BASE as *mut COM1Port));
        
        if let Some(com1) = &mut COM1 {
            // Disable interrupts
            com1.interrupt_enable.write(0);
            
            // Set DLAB to access divisor latch
            com1.line_control.write(line_ctrl_bits::DLAB);
            
            // Set divisor for 115200 baud (assuming 1.8432MHz crystal)
            // Divisor = 1843200 / (16 * 115200) = 1
            com1.divisor_low.write(1);
            com1.divisor_high.write(0);
            
            // Clear DLAB and set line control: 8 data bits, 1 stop bit, no parity
            com1.line_control.write(line_ctrl_bits::DATA_BITS_8 | line_ctrl_bits::STOP_BITS_1 | line_ctrl_bits::PARITY_NONE);
            
            // Enable FIFO, clear them, set trigger level to 14 bytes
            com1.fifo_control.write(fifo_ctrl_bits::ENABLE | fifo_ctrl_bits::CLEAR_RECEIVER | fifo_ctrl_bits::CLEAR_TRANSMITTER | fifo_ctrl_bits::TRIGGER_14);
            
            // Set modem control: DTR and RTS enabled
            com1.modem_control.write(modem_ctrl_bits::DTR | modem_ctrl_bits::RTS);
            
            // Enable interrupts (Phase 1: disabled, Phase 2: enable as needed)
            com1.interrupt_enable.write(0);
            
            COM1_INITIALIZED = true;
        }
    }
    
    println!("COM1 serial port initialized");
}

/// Write a single character to COM1
pub fn write_char(c: u8) {
    unsafe {
        if let Some(com1) = &mut COM1 {
            // Wait for transmitter holding register to be empty
            while com1.line_status.read() & line_status_bits::THR_EMPTY == 0 {}
            
            // Write character
            com1.data.write(c);
        }
    }
}

/// Read a single character from COM1 (non-blocking)
pub fn read_char() -> Option<u8> {
    unsafe {
        if let Some(com1) = &mut COM1 {
            // Check if data is ready
            if com1.line_status.read() & line_status_bits::DATA_READY != 0 {
                Some(com1.data.read())
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Write a string to COM1
pub fn write_string(s: &str) {
    for byte in s.bytes() {
        write_char(byte);
    }
}

/// Check if COM1 is initialized
pub fn is_initialized() -> bool {
    unsafe { COM1_INITIALIZED }
}

/// Flush TX buffer
pub fn flush() {
    unsafe {
        if let Some(com1) = &mut COM1 {
            // Wait for transmitter to be empty
            while com1.line_status.read() & line_status_bits::TRANSMITTER_EMPTY == 0 {}
        }
    }
}

/// Enable/disable COM1 interrupts
pub fn set_interrupts(enable: bool) {
    unsafe {
        if let Some(com1) = &mut COM1 {
            if enable {
                // Enable data available interrupt
                com1.interrupt_enable.write(0x01);
            } else {
                // Disable all interrupts
                com1.interrupt_enable.write(0);
            }
        }
    }
}

/// Get COM1 status
pub fn get_status() -> u8 {
    unsafe {
        if let Some(com1) = &mut COM1 {
            com1.line_status.read()
        } else {
            0
        }
    }
}

/// COM1 interrupt handling
pub mod interrupts {
    use super::*;
    
    /// COM1 interrupt types
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Com1Interrupt {
        ModemStatus,
        TransmitterEmpty,
        DataAvailable,
        LineStatus,
        Timeout,
    }
    
    /// Get pending COM1 interrupts
    pub fn get_pending_interrupts() -> arrayvec::ArrayVec<Com1Interrupt, 5> {
        let mut interrupts = arrayvec::ArrayVec::new();
        
        unsafe {
            if let Some(com1) = &mut COM1 {
                let int_id = com1.interrupt_id.read();
                
                // Check if there's an interrupt pending
                if int_id & 0x01 == 0 {
                    let int_type = (int_id >> 1) & 0x03;
                    
                    match int_type {
                        0 => interrupts.push(Com1Interrupt::ModemStatus),
                        1 => interrupts.push(Com1Interrupt::TransmitterEmpty),
                        2 => interrupts.push(Com1Interrupt::DataAvailable),
                        3 => {
                            // Line status or timeout
                            if int_id & 0x08 != 0 {
                                interrupts.push(Com1Interrupt::Timeout);
                            } else {
                                interrupts.push(Com1Interrupt::LineStatus);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        interrupts
    }
    
    /// Clear COM1 interrupt
    pub fn clear_interrupt(interrupt: Com1Interrupt) {
        // COM1 interrupts are cleared by reading appropriate registers
        match interrupt {
            Com1Interrupt::ModemStatus => {
                unsafe {
                    if let Some(com1) = &mut COM1 {
                        let _ = com1.modem_status.read();
                    }
                }
            }
            Com1Interrupt::DataAvailable => {
                // Read data register to clear
                let _ = read_char();
            }
            Com1Interrupt::LineStatus => {
                unsafe {
                    if let Some(com1) = &mut COM1 {
                        let _ = com1.line_status.read();
                    }
                }
            }
            _ => {
                // Other interrupts are cleared automatically
            }
        }
    }
}

/// COM1 configuration utilities
pub mod config {
    use super::*;
    
    /// Set baud rate
    pub fn set_baud_rate(baud_rate: u32) {
        // Assuming 1.8432MHz crystal
        const CRYSTAL_FREQ: u32 = 1_843_200;
        
        let divisor = CRYSTAL_FREQ / (16 * baud_rate);
        
        unsafe {
            if let Some(com1) = &mut COM1 {
                // Set DLAB to access divisor latch
                let current_lcr = com1.line_control.read();
                com1.line_control.write(current_lcr | line_ctrl_bits::DLAB);
                
                // Set divisor
                com1.divisor_low.write((divisor & 0xFF) as u8);
                com1.divisor_high.write((divisor >> 8) as u8);
                
                // Restore original LCR
                com1.line_control.write(current_lcr);
            }
        }
    }
    
    /// Set data bits
    pub fn set_data_bits(bits: u8) {
        unsafe {
            if let Some(com1) = &mut COM1 {
                let current_lcr = com1.line_control.read();
                let data_bits = match bits {
                    5 => line_ctrl_bits::DATA_BITS_5,
                    6 => line_ctrl_bits::DATA_BITS_6,
                    7 => line_ctrl_bits::DATA_BITS_7,
                    8 => line_ctrl_bits::DATA_BITS_8,
                    _ => line_ctrl_bits::DATA_BITS_8, // Default to 8 bits
                };
                
                com1.line_control.write((current_lcr & !0x03) | data_bits);
            }
        }
    }
    
    /// Set parity
    pub fn set_parity(parity: Parity) {
        unsafe {
            if let Some(com1) = &mut COM1 {
                let current_lcr = com1.line_control.read();
                let parity_bits = match parity {
                    Parity::None => line_ctrl_bits::PARITY_NONE,
                    Parity::Odd => line_ctrl_bits::PARITY_ODD,
                    Parity::Even => line_ctrl_bits::PARITY_EVEN,
                    Parity::Mark => line_ctrl_bits::PARITY_MARK,
                    Parity::Space => line_ctrl_bits::PARITY_SPACE,
                };
                
                com1.line_control.write((current_lcr & !0x38) | parity_bits);
            }
        }
    }
    
    /// Set stop bits
    pub fn set_stop_bits(stop_bits: StopBits) {
        unsafe {
            if let Some(com1) = &mut COM1 {
                let current_lcr = com1.line_control.read();
                let stop_bit = match stop_bits {
                    StopBits::One => line_ctrl_bits::STOP_BITS_1,
                    StopBits::Two => line_ctrl_bits::STOP_BITS_2,
                };
                
                com1.line_control.write((current_lcr & !line_ctrl_bits::STOP_BITS_2) | stop_bit);
            }
        }
    }
    
    /// Enable/disable FIFO
    pub fn set_fifo(enable: bool) {
        unsafe {
            if let Some(com1) = &mut COM1 {
                if enable {
                    // Enable FIFO, clear them, set trigger level to 14 bytes
                    com1.fifo_control.write(fifo_ctrl_bits::ENABLE | fifo_ctrl_bits::CLEAR_RECEIVER | fifo_ctrl_bits::CLEAR_TRANSMITTER | fifo_ctrl_bits::TRIGGER_14);
                } else {
                    // Disable FIFO
                    com1.fifo_control.write(0);
                }
            }
        }
    }
    
    /// Set FIFO trigger level
    pub fn set_fifo_trigger_level(level: FifoTriggerLevel) {
        unsafe {
            if let Some(com1) = &mut COM1 {
                let current_fcr = com1.fifo_control.read();
                let trigger_bits = match level {
                    FifoTriggerLevel::Bytes1 => fifo_ctrl_bits::TRIGGER_1,
                    FifoTriggerLevel::Bytes4 => fifo_ctrl_bits::TRIGGER_4,
                    FifoTriggerLevel::Bytes8 => fifo_ctrl_bits::TRIGGER_8,
                    FifoTriggerLevel::Bytes14 => fifo_ctrl_bits::TRIGGER_14,
                };
                
                com1.fifo_control.write((current_fcr & !0xC0) | trigger_bits);
            }
        }
    }
}

/// Parity settings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Parity {
    None,
    Odd,
    Even,
    Mark,
    Space,
}

/// Stop bit settings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StopBits {
    One,
    Two,
}

/// FIFO trigger levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FifoTriggerLevel {
    Bytes1,
    Bytes4,
    Bytes8,
    Bytes14,
}

/// Implement the standard print macro for COM1 output
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut com1_writer = $crate::arch::x86_64::serial::Com1Writer;
            write!(com1_writer, $($arg)*).unwrap();
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

/// COM1 writer for formatting
pub struct Com1Writer;

impl core::fmt::Write for Com1Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write_string(s);
        Ok(())
    }
}

/// COM1 statistics
pub mod stats {
    use super::*;
    
    static BYTES_SENT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
    static BYTES_RECEIVED: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
    static ERRORS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
    
    /// Get bytes sent
    pub fn get_bytes_sent() -> u64 {
        BYTES_SENT.load(core::sync::atomic::Ordering::SeqCst)
    }
    
    /// Get bytes received
    pub fn get_bytes_received() -> u64 {
        BYTES_RECEIVED.load(core::sync::atomic::Ordering::SeqCst)
    }
    
    /// Get error count
    pub fn get_error_count() -> u64 {
        ERRORS.load(core::sync::atomic::Ordering::SeqCst)
    }
    
    /// Increment bytes sent
    pub fn increment_bytes_sent(count: u64) {
        BYTES_SENT.fetch_add(count, core::sync::atomic::Ordering::SeqCst);
    }
    
    /// Increment bytes received
    pub fn increment_bytes_received(count: u64) {
        BYTES_RECEIVED.fetch_add(count, core::sync::atomic::Ordering::SeqCst);
    }
    
    /// Increment error count
    pub fn increment_errors(count: u64) {
        ERRORS.fetch_add(count, core::sync::atomic::Ordering::SeqCst);
    }
    
    /// Reset statistics
    pub fn reset_stats() {
        BYTES_SENT.store(0, core::sync::atomic::Ordering::SeqCst);
        BYTES_RECEIVED.store(0, core::sync::atomic::Ordering::SeqCst);
        ERRORS.store(0, core::sync::atomic::Ordering::SeqCst);
    }
}
