// XPARQ OS - x86-64 Display Driver
// Simple VGA/BIOS framebuffer implementation for QEMU x86-64

use crate::display::{DisplayDriver, DisplayInfo, DisplayMode, PixelFormat, DisplayError, Framebuffer, DisplayCapabilities};
use core::ptr::{read_volatile, write_volatile};
use core::fmt;

/// VGA Text Buffer Display Driver for x86-64
pub struct VgaTextDisplay {
    buffer_addr: usize,
    width: u32,
    height: u32,
    initialized: bool,
    cursor_x: u32,
    cursor_y: u32,
    current_color: u8,
    mouse_x: u32,
    mouse_y: u32,
    saved_char: Option<(u8, u8)>, // (character, color)
}

/// VGA Color Codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VgaColor {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xA,
    LightCyan = 0xB,
    LightRed = 0xC,
    LightMagenta = 0xD,
    Yellow = 0xE,
    White = 0xF,
}

impl VgaTextDisplay {
    /// Create a new VGA text display driver
    pub fn new() -> Self {
        Self {
            buffer_addr: 0xB8000, // Standard VGA text buffer address
            width: 80,
            height: 25,
            initialized: false,
            cursor_x: 0,
            cursor_y: 0,
            current_color: VgaTextDisplay::make_color(VgaColor::White, VgaColor::Blue),
            mouse_x: 0,
            mouse_y: 0,
            saved_char: None,
        }
    }
    
    /// Get mouse position
    pub fn get_mouse_pos(&self) -> (u32, u32) {
        (self.mouse_x, self.mouse_y)
    }
    
    /// Set mouse position
    pub fn set_mouse_pos(&mut self, x: u32, y: u32) {
        let new_x = x.clamp(0, self.width - 1);
        let new_y = y.clamp(0, self.height - 1);
        
        if new_x != self.mouse_x || new_y != self.mouse_y {
            // Restore old character first
            self.restore_saved_char();
            
            // Update position
            self.mouse_x = new_x;
            self.mouse_y = new_y;
            
            // Draw new cursor
            self.draw_mouse_cursor();
        }
    }
    
    /// Move mouse relative
    pub fn move_mouse(&mut self, dx: i32, dy: i32) {
        let new_x = (self.mouse_x as i32 + dx).clamp(0, self.width as i32 - 1) as u32;
        let new_y = (self.mouse_y as i32 + dy).clamp(0, self.height as i32 - 1) as u32;
        self.set_mouse_pos(new_x, new_y);
    }
    
    /// Draw mouse cursor
    fn draw_mouse_cursor(&mut self) {
        let offset = (self.mouse_y * self.width + self.mouse_x) as usize * 2;
        let addr = (self.buffer_addr + offset) as *mut u16;
        
        // Save current character and color
        unsafe {
            let value = read_volatile(addr);
            let old_char = (value & 0xFF) as u8;
            let old_color = ((value >> 8) & 0xFF) as u8;
            self.saved_char = Some((old_char, old_color));
            
            // Draw cursor (inverted block)
            let cursor_color = VgaTextDisplay::make_color(VgaColor::Black, VgaColor::White);
            let cursor_value = ((cursor_color as u16) << 8) | (b'*' as u16);
            write_volatile(addr, cursor_value);
        }
    }
    
    /// Restore saved character
    fn restore_saved_char(&mut self) {
        if let Some((saved_char, saved_color)) = self.saved_char.take() {
            let offset = (self.mouse_y * self.width + self.mouse_x) as usize * 2;
            let addr = (self.buffer_addr + offset) as *mut u16;
            let value = ((saved_color as u16) << 8) | (saved_char as u16);
            unsafe {
                write_volatile(addr, value);
            }
        }
    }

    /// Create a VGA color byte from foreground and background colors
    pub const fn make_color(fg: VgaColor, bg: VgaColor) -> u8 {
        (bg as u8) << 4 | (fg as u8)
    }

    /// Set current text color
    pub fn set_color(&mut self, color: u8) {
        self.current_color = color;
    }

    /// Set foreground and background colors
    pub fn set_colors(&mut self, fg: VgaColor, bg: VgaColor) {
        self.current_color = Self::make_color(fg, bg);
    }

    /// Set cursor position
    pub fn set_cursor(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.cursor_x = x;
            self.cursor_y = y;
        }
    }

    /// Get current cursor position
    pub fn get_cursor(&self) -> (u32, u32) {
        (self.cursor_x, self.cursor_y)
    }

    /// Write a character to the VGA text buffer at cursor position and advance
    pub fn write_char_at_cursor(&mut self, character: u8) {
        match character {
            b'\n' => {
                self.cursor_x = 0;
                self.cursor_y += 1;
                if self.cursor_y >= self.height {
                    self.scroll_up(self.current_color);
                    self.cursor_y = self.height - 1;
                }
            }
            b'\r' => {
                self.cursor_x = 0;
            }
            b'\t' => {
                self.cursor_x += 8;
                if self.cursor_x >= self.width {
                    self.cursor_x = self.width - 1;
                }
            }
            0x08 => { // Backspace
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                    self.write_char(self.cursor_x, self.cursor_y, b' ', self.current_color);
                }
            }
            _ => {
                self.write_char(self.cursor_x, self.cursor_y, character, self.current_color);
                self.cursor_x += 1;
                if self.cursor_x >= self.width {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                    if self.cursor_y >= self.height {
                        self.scroll_up(self.current_color);
                        self.cursor_y = self.height - 1;
                    }
                }
            }
        }
    }

    /// Write a character to the VGA text buffer at specific position
    pub fn write_char(&mut self, x: u32, y: u32, character: u8, color: u8) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = (y * self.width + x) as usize * 2;
        let addr = (self.buffer_addr + offset) as *mut u16;
        let value = ((color as u16) << 8) | (character as u16);
        unsafe {
            write_volatile(addr, value);
        }
    }

    /// Write a string to the VGA text buffer at specific position
    pub fn write_string(&mut self, x: u32, y: u32, string: &[u8], color: u8) {
        for (i, &c) in string.iter().enumerate() {
            let current_x = x + i as u32;
            if current_x >= self.width {
                break;
            }
            self.write_char(current_x, y, c, color);
        }
    }

    /// Print a string at current cursor position
    pub fn print(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_char_at_cursor(byte);
        }
    }

    /// Clear the entire screen
    pub fn clear_screen(&mut self, color: u8) {
        // Restore saved character first
        self.restore_saved_char();
        
        for y in 0..self.height {
            for x in 0..self.width {
                self.write_char(x, y, b' ', color);
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
        
        // Redraw mouse cursor
        self.draw_mouse_cursor();
    }

    /// Clear a single line
    pub fn clear_line(&mut self, y: u32, color: u8) {
        if y < self.height {
            for x in 0..self.width {
                self.write_char(x, y, b' ', color);
            }
        }
    }

    /// Scroll screen up by one line
    pub fn scroll_up(&mut self, color: u8) {
        unsafe {
            let buffer = self.buffer_addr as *mut u16;
            let size = (self.width * self.height) as usize;
            let line_size = self.width as usize;
            
            // Shift all lines up by one
            for i in 0..size - line_size {
                write_volatile(buffer.add(i), read_volatile(buffer.add(i + line_size)));
            }
            
            // Clear last line
            let last_line = size - line_size;
            let blank = ((color as u16) << 8) | (b' ' as u16);
            for x in 0..line_size {
                write_volatile(buffer.add(last_line + x), blank);
            }
        }
    }

    /// Scroll screen down by one line
    pub fn scroll_down(&mut self, color: u8) {
        unsafe {
            let buffer = self.buffer_addr as *mut u16;
            let size = (self.width * self.height) as usize;
            let line_size = self.width as usize;
            
            // Shift all lines down by one
            for i in (line_size..size).rev() {
                write_volatile(buffer.add(i), read_volatile(buffer.add(i - line_size)));
            }
            
            // Clear first line
            let blank = ((color as u16) << 8) | (b' ' as u16);
            for x in 0..line_size {
                write_volatile(buffer.add(x), blank);
            }
        }
    }
}

impl fmt::Write for VgaTextDisplay {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
}

impl Default for VgaTextDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayDriver for VgaTextDisplay {
    fn name(&self) -> &'static str {
        "VGA Text Mode Display"
    }

    fn init(&mut self) -> Result<(), DisplayError> {
        self.clear_screen(0x1F); // Blue background, white text
        self.initialized = true;
        Ok(())
    }

    fn get_info(&self) -> DisplayInfo {
        DisplayInfo {
            name: "VGA Text Mode",
            width: self.width,
            height: self.height,
            supported_modes: {
                let mut modes = arrayvec::ArrayVec::new();
                modes.push(DisplayMode {
                    width: self.width,
                    height: self.height,
                    refresh_rate: 60,
                    pixel_format: PixelFormat::Rgb32, // Just for compatibility
                    flags: Default::default(),
                });
                modes
            },
            supported_formats: {
                let mut formats = arrayvec::ArrayVec::new();
                formats.push(PixelFormat::Rgb32);
                formats
            },
            capabilities: DisplayCapabilities::default(),
        }
    }

    fn set_mode(&mut self, _mode: &DisplayMode) -> Result<(), DisplayError> {
        Ok(())
    }

    fn get_mode(&self) -> DisplayMode {
        DisplayMode {
            width: self.width,
            height: self.height,
            refresh_rate: 60,
            pixel_format: PixelFormat::Rgb32,
            flags: Default::default(),
        }
    }

    fn create_framebuffer(&mut self, _width: u32, _height: u32, _format: PixelFormat) -> Result<Framebuffer, DisplayError> {
        Ok(Framebuffer {
            width: self.width,
            height: self.height,
            stride: self.width * 2,
            format: PixelFormat::Rgb32,
            address: self.buffer_addr,
            size: (self.width * self.height * 2) as usize,
            flags: Default::default(),
        })
    }

    fn present_framebuffer(&mut self, _framebuffer: &Framebuffer) -> Result<(), DisplayError> {
        Ok(())
    }

    fn set_backlight(&mut self, _brightness: u8) -> Result<(), DisplayError> {
        Ok(())
    }

    fn get_backlight(&self) -> u8 {
        100
    }

    fn set_power(&mut self, _enabled: bool) -> Result<(), DisplayError> {
        Ok(())
    }

    fn is_powered(&self) -> bool {
        true
    }
}
