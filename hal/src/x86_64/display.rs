// XPARQ OS - x86-64 Display Driver
// Simple VGA/BIOS framebuffer implementation for QEMU x86-64

use crate::display::{DisplayDriver, DisplayInfo, DisplayMode, PixelFormat, DisplayError, Framebuffer, DisplayCapabilities};
use core::ptr::{read_volatile, write_volatile};

/// VGA Text Buffer Display Driver for x86-64
pub struct VgaTextDisplay {
    buffer_addr: usize,
    width: u32,
    height: u32,
    initialized: bool,
}

impl VgaTextDisplay {
    /// Create a new VGA text display driver
    pub fn new() -> Self {
        Self {
            buffer_addr: 0xB8000, // Standard VGA text buffer address
            width: 80,
            height: 25,
            initialized: false,
        }
    }

    /// Write a character to the VGA text buffer
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

    /// Write a string to the VGA text buffer
    pub fn write_string(&mut self, x: u32, y: u32, string: &[u8], color: u8) {
        for (i, &c) in string.iter().enumerate() {
            let current_x = x + i as u32;
            if current_x >= self.width {
                break;
            }
            self.write_char(current_x, y, c, color);
        }
    }

    /// Clear the entire screen
    pub fn clear_screen(&mut self, color: u8) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.write_char(x, y, b' ', color);
            }
        }
    }

    /// Scroll screen up (one line)
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
            for x in 0..line_size {
                let blank = ((color as u16) << 8) | (b' ' as u16);
                write_volatile(buffer.add(last_line + x), blank);
            }
        }
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
