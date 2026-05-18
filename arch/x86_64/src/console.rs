//! x86-64 Console - Phase 2: Dev Environment Setup
//! 
//! This module provides x86-64 console output for XPARQ OS, including:
//! - VGA text mode support for early debug output
//! - Serial port (COM1) support
//! - UEFI console integration
//! - Kernel message printing
//! 
//! Console Types: VGA text mode, Serial port, UEFI console
//! VGA Address: 0xB8000 (text mode)
//! Serial Port: COM1 (0x3F8)
//! Baud Rate: 115200 (standard)
//! Data Format: 8N1 (8 data bits, no parity, 1 stop bit)
//! 
//! Roadmap Phase: Phase 2 - Dev Environment Setup

/// VGA text mode base address
const VGA_BASE: usize = 0xB8000;

/// Serial port base addresses
const COM1_BASE: usize = 0x3F8;
const COM2_BASE: usize = 0x2F8;

/// VGA dimensions
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

/// VGA colors
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum VgaColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

/// Console state
static mut CONSOLE_STATE: Option<ConsoleState> = None;

/// Console state structure
#[derive(Debug)]
pub struct ConsoleState {
    /// Current cursor position
    pub cursor_x: usize,
    pub cursor_y: usize,
    /// Current color
    pub color: u8,
    /// Console type
    pub console_type: ConsoleType,
}

/// Console types
#[derive(Debug, Clone, Copy)]
pub enum ConsoleType {
    Vga,
    Serial,
    Uefi,
}

/// Initialize early console
pub fn early_init() {
    // Phase 1: Try VGA first, then serial
    // Phase 2: Use UEFI console if available
    
    if is_vga_available() {
        init_vga_console();
    } else if is_serial_available() {
        init_serial_console();
    }
    
    println!("Early console initialized");
}

/// Initialize console
pub fn init() {
    println!("Initializing x86-64 console...");
    
    // Phase 2: Full console initialization
    // Phase 3: Graphics console support
    
    println!("x86-64 console initialized");
}

/// Write string to console
pub fn write_str(s: &str) {
    let state = unsafe { CONSOLE_STATE.as_ref() };
    
    match state.map(|s| s.console_type) {
        Some(ConsoleType::Vga) => vga_write_str(s),
        Some(ConsoleType::Serial) => serial_write_str(s),
        Some(ConsoleType::Uefi) => uefi_write_str(s),
        None => {
            // No console initialized, try to initialize
            early_init();
            write_str(s);
        }
    }
}

/// Write byte to console
pub fn write_byte(byte: u8) {
    let state = unsafe { CONSOLE_STATE.as_ref() };
    
    match state.map(|s| s.console_type) {
        Some(ConsoleType::Vga) => vga_write_byte(byte),
        Some(ConsoleType::Serial) => serial_write_byte(byte),
        Some(ConsoleType::Uefi) => uefi_write_byte(byte),
        None => {
            // No console initialized, try to initialize
            early_init();
            write_byte(byte);
        }
    }
}

/// Check if VGA is available
fn is_vga_available() -> bool {
    // Phase 1: Assume VGA is available
    // Phase 2: Check for VGA BIOS and hardware
    true
}

/// Check if serial is available
fn is_serial_available() -> bool {
    // Phase 1: Check COM1 port
    // Phase 2: Proper serial port detection
    
    // Check if COM1 base port is accessible
    unsafe {
        let com1_data = COM1_BASE as *mut u8;
        let test_value = 0xAA;
        core::ptr::write_volatile(com1_data, test_value);
        let read_value = core::ptr::read_volatile(com1_data);
        read_value == test_value
    }
}

/// Initialize VGA console
fn init_vga_console() {
    println!("Initializing VGA console...");
    
    let state = ConsoleState {
        cursor_x: 0,
        cursor_y: 0,
        color: make_color(VgaColor::LightGrey, VgaColor::Black),
        console_type: ConsoleType::Vga,
    };
    
    unsafe {
        CONSOLE_STATE = Some(state);
    }
    
    // Clear VGA screen
    vga_clear_screen();
    
    println!("VGA console initialized");
}

/// Initialize serial console
fn init_serial_console() {
    println!("Initializing serial console...");
    
    // Configure serial port
    serial_configure(COM1_BASE, 115200);
    
    let state = ConsoleState {
        cursor_x: 0,
        cursor_y: 0,
        color: 0,
        console_type: ConsoleType::Serial,
    };
    
    unsafe {
        CONSOLE_STATE = Some(state);
    }
    
    println!("Serial console initialized");
}

/// Make VGA color
fn make_color(fg: VgaColor, bg: VgaColor) -> u8 {
    (bg as u8) << 4 | (fg as u8)
}

/// Clear VGA screen
fn vga_clear_screen() {
    let vga_buffer = unsafe { core::ptr::slice_from_raw_parts_mut(VGA_BASE as *mut u16, VGA_WIDTH * VGA_HEIGHT) };
    
    for i in 0..VGA_WIDTH * VGA_HEIGHT {
        unsafe {
            vga_buffer[i] = make_color(VgaColor::LightGrey, VgaColor::Black) as u16;
        }
    }
}

/// Write string to VGA
fn vga_write_str(s: &str) {
    for byte in s.bytes() {
        vga_write_byte(byte);
    }
}

/// Write byte to VGA
fn vga_write_byte(byte: u8) {
    let state = unsafe { CONSOLE_STATE.as_mut().unwrap() };
    
    match byte {
        b'\n' => {
            state.cursor_x = 0;
            state.cursor_y += 1;
            if state.cursor_y >= VGA_HEIGHT {
                vga_scroll_up();
                state.cursor_y = VGA_HEIGHT - 1;
            }
        }
        b'\r' => {
            state.cursor_x = 0;
        }
        b'\t' => {
            state.cursor_x = (state.cursor_x + 8) & !7;
            if state.cursor_x >= VGA_WIDTH {
                vga_write_byte(b'\n');
            }
        }
        0x20..=0x7E => {
            // Printable ASCII
            let vga_buffer = unsafe { core::ptr::slice_from_raw_parts_mut(VGA_BASE as *mut u16, VGA_WIDTH * VGA_HEIGHT) };
            let index = state.cursor_y * VGA_WIDTH + state.cursor_x;
            
            unsafe {
                vga_buffer[index] = (byte as u16) | ((state.color as u16) << 8);
            }
            
            state.cursor_x += 1;
            if state.cursor_x >= VGA_WIDTH {
                vga_write_byte(b'\n');
            }
        }
        _ => {
            // Non-printable character, ignore for now
        }
    }
    
    vga_update_cursor();
}

/// Scroll VGA screen up
fn vga_scroll_up() {
    let vga_buffer = unsafe { core::ptr::slice_from_raw_parts_mut(VGA_BASE as *mut u16, VGA_WIDTH * VGA_HEIGHT) };
    
    // Move everything up one line
    for y in 1..VGA_HEIGHT {
        for x in 0..VGA_WIDTH {
            let src_index = y * VGA_WIDTH + x;
            let dst_index = (y - 1) * VGA_WIDTH + x;
            
            unsafe {
                vga_buffer[dst_index] = vga_buffer[src_index];
            }
        }
    }
    
    // Clear last line
    let last_line_start = (VGA_HEIGHT - 1) * VGA_WIDTH;
    for x in 0..VGA_WIDTH {
        unsafe {
            vga_buffer[last_line_start + x] = make_color(VgaColor::LightGrey, VgaColor::Black) as u16;
        }
    }
}

/// Update VGA cursor position
fn vga_update_cursor() {
    let state = unsafe { CONSOLE_STATE.as_ref().unwrap() };
    
    let position = state.cursor_y * VGA_WIDTH + state.cursor_x;
    
    // Send cursor position to VGA hardware
    unsafe {
        // Set cursor location low byte
        core::ptr::write_volatile(0x3D4 as *mut u8, 0x0F);
        core::ptr::write_volatile(0x3D5 as *mut u8, (position & 0xFF) as u8);
        
        // Set cursor location high byte
        core::ptr::write_volatile(0x3D4 as *mut u8, 0x0E);
        core::ptr::write_volatile(0x3D5 as *mut u8, ((position >> 8) & 0xFF) as u8);
    }
}

/// Configure serial port
fn serial_configure(base: usize, baud_rate: u32) {
    // Disable interrupts
    unsafe {
        core::ptr::write_volatile((base + 1) as *mut u8, 0x00);
    }
    
    // Set baud rate divisor
    let divisor = 115200 / baud_rate;
    unsafe {
        // Enable DLAB (set bit 7 of line control register)
        core::ptr::write_volatile((base + 3) as *mut u8, 0x80);
        
        // Set divisor latch low byte
        core::ptr::write_volatile((base + 0) as *mut u8, (divisor & 0xFF) as u8);
        
        // Set divisor latch high byte
        core::ptr::write_volatile((base + 1) as *mut u8, ((divisor >> 8) & 0xFF) as u8);
        
        // Disable DLAB and set 8N1 format
        core::ptr::write_volatile((base + 3) as *mut u8, 0x03);
    }
    
    // Enable FIFO, clear them, with 14-byte threshold
    unsafe {
        core::ptr::write_volatile((base + 2) as *mut u8, 0xC7);
    }
    
    // Enable interrupts
    unsafe {
        core::ptr::write_volatile((base + 1) as *mut u8, 0x01);
    }
}

/// Write string to serial port
fn serial_write_str(s: &str) {
    for byte in s.bytes() {
        serial_write_byte(byte);
    }
}

/// Write byte to serial port
fn serial_write_byte(byte: u8) {
    // Wait for transmit buffer to be empty
    while unsafe { core::ptr::read_volatile((COM1_BASE + 5) as *const u8) } & 0x20 == 0 {
        core::hint::spin_loop();
    }
    
    // Write byte
    unsafe {
        core::ptr::write_volatile(COM1_BASE as *mut u8, byte);
    }
}

/// Write string to UEFI console
fn uefi_write_str(s: &str) {
    // Phase 2: Use UEFI console protocol
    // Phase 3: Proper UEFI console integration
    println!("UEFI console: {}", s);
}

/// Write byte to UEFI console
fn uefi_write_byte(byte: u8) {
    // Phase 2: Use UEFI console protocol
    // Phase 3: Proper UEFI console integration
    if byte == b'\n' {
        println!();
    } else if byte >= 0x20 && byte <= 0x7E {
        print!("{}", byte as char);
    }
}

/// Read byte from serial port (non-blocking)
pub fn read_byte() -> Option<u8> {
    // Check if data is available
    if unsafe { core::ptr::read_volatile((COM1_BASE + 5) as *const u8) } & 0x01 != 0 {
        Some(unsafe { core::ptr::read_volatile(COM1_BASE as *const u8) })
    } else {
        None
    }
}
