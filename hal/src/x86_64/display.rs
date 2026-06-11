// XPARQ OS - x86-64 Display Driver
// Supports both VGA Text Mode and VBE Framebuffer
use crate::display::{DisplayDriver, DisplayInfo, DisplayMode, PixelFormat, DisplayError, Framebuffer, DisplayCapabilities};
use core::ptr::{read_volatile, write_volatile};
use core::fmt;

/// VBE Mode Info Structure (simplified)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct VbeModeInfo {
    _attributes: u16,
    _win_a: u8,
    _win_b: u8,
    _granularity: u16,
    _win_size: u16,
    _segment_a: u16,
    _segment_b: u16,
    _win_func_ptr: u32,
    pitch: u16,
    width: u16,
    height: u16,
    _w_char: u8,
    _y_char: u8,
    _planes: u8,
    bits_per_pixel: u8,
    _banks: u8,
    _memory_model: u8,
    _bank_size: u8,
    _image_pages: u8,
    _reserved0: u8,
    _red_mask: u8,
    _red_position: u8,
    _green_mask: u8,
    _green_position: u8,
    _blue_mask: u8,
    _blue_position: u8,
    _rsvd_mask: u8,
    _rsvd_position: u8,
    _direct_color_info: u8,
    framebuffer_base: u32,
    _reserved1: [u8; 212],
}

/// Unified display driver that uses either VBE framebuffer or VGA text
pub struct X86Display {
    inner: X86DisplayInner,
}

enum X86DisplayInner {
    Vbe {
        framebuffer: *mut u32,
        width: u32,
        height: u32,
        pitch: u32,
        cursor_x: u32,
        cursor_y: u32,
    },
    Text(VgaTextDisplay),
}

// Include the VGA font data
const VGA_FONT: &[u8; 4096] = include_bytes!("vga-font.bin");

/// VGA Text Buffer Display Driver (fallback)
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

impl X86Display {
    /// Create new display driver (tries VBE first, falls back to text mode)
    pub fn new() -> Self {
        let vbe_info_ptr = 0x7E00 as *const VbeModeInfo;
        let vbe_info = unsafe { read_volatile(vbe_info_ptr) };

        if vbe_info.width > 0 
            && vbe_info.height > 0 
            && vbe_info.bits_per_pixel == 32 
            && vbe_info.framebuffer_base != 0 
        {
            // VBE mode is available
            X86Display {
                inner: X86DisplayInner::Vbe {
                    framebuffer: vbe_info.framebuffer_base as *mut u32,
                    width: vbe_info.width as u32,
                    height: vbe_info.height as u32,
                    pitch: vbe_info.pitch as u32,
                    cursor_x: 0,
                    cursor_y: 0,
                }
            }
        } else {
            // Fall back to VGA text mode
            X86Display {
                inner: X86DisplayInner::Text(VgaTextDisplay::new())
            }
        }
    }

    /// Create pixel value (RGBA → BGRX for VBE)
    #[inline(always)]
    pub fn make_pixel(r: u8, g: u8, b: u8) -> u32 {
        (b as u32) | ((g as u32) << 8) | ((r as u32) << 16)
    }

    /// Set pixel at (x,y)
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        match &mut self.inner {
            X86DisplayInner::Vbe { framebuffer, width, height, pitch, .. } => {
                if x < *width && y < *height {
                    let offset = (y * pitch / 4 + x) as usize;
                    unsafe {
                        write_volatile(framebuffer.add(offset), color);
                    }
                }
            },
            X86DisplayInner::Text(_) => { /* No pixel support in text mode */ }
        }
    }

    /// Draw simple character (for testing)
    pub fn draw_char(&mut self, x: u32, y: u32, c: u8, fg: u32, bg: u32) {
        match &mut self.inner {
            X86DisplayInner::Vbe { framebuffer, width, height, pitch, .. } => {
                let glyph_offset = (c as usize) * 16;
                let glyph = &VGA_FONT[glyph_offset..glyph_offset + 16];
                
                for row in 0..16 {
                    for col in 0..8 {
                        let bit = (glyph[row] >> (7 - col)) & 1;
                        let px = x + col;
                        let py = y + row;
                        
                        if px < *width && py < *height {
                            let offset = (py * pitch / 4 + px) as usize;
                            unsafe {
                                write_volatile(framebuffer.add(offset), if bit != 0 { fg } else { bg });
                            }
                        }
                    }
                }
            },
            X86DisplayInner::Text(text) => {
                let vga_color = VgaTextDisplay::make_color(VgaColor::White, VgaColor::Blue);
                text.write_char(x / 8, y / 16, c, vga_color);
            }
        }
    }

    /// Clear screen
    pub fn clear_screen(&mut self, color: u32) {
        match &mut self.inner {
            X86DisplayInner::Vbe { framebuffer, width, height, pitch, .. } => {
                for y in 0..*height {
                    for x in 0..*width {
                        let offset = (y * pitch / 4 + x) as usize;
                        unsafe {
                            write_volatile(framebuffer.add(offset), color);
                        }
                    }
                }
            },
            X86DisplayInner::Text(text) => {
                text.clear_screen(0x1F);
            }
        }
    }
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
    
    /// Create a VGA color byte from foreground and background colors
    pub const fn make_color(fg: VgaColor, bg: VgaColor) -> u8 {
        (bg as u8) << 4 | (fg as u8)
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

    /// Clear the entire screen
    pub fn clear_screen(&mut self, color: u8) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.write_char(x, y, b' ', color);
            }
        }
    }
}

impl fmt::Write for X86Display {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let fg = Self::make_pixel(255, 255, 255);
        let bg = Self::make_pixel(0, 0, 128);

        match &mut self.inner {
            X86DisplayInner::Vbe { width, height, pitch, cursor_x, cursor_y, framebuffer, .. } => {
                const CHAR_WIDTH: u32 = 8;
                const CHAR_HEIGHT: u32 = 16;

                for byte in s.bytes() {
                    match byte {
                        b'\n' => {
                            *cursor_x = 0;
                            *cursor_y += CHAR_HEIGHT;
                            if *cursor_y + CHAR_HEIGHT > *height {
                                // Simple scroll: clear last line
                                for y in *height - CHAR_HEIGHT..*height {
                                    for x in 0..*width {
                                        let offset = (y * pitch / 4 + x) as usize;
                                        unsafe {
                                            write_volatile(framebuffer.add(offset), bg);
                                        }
                                    }
                                }
                                *cursor_y = *height - CHAR_HEIGHT;
                            }
                        },
                        _ => {
                            self.draw_char(*cursor_x, *cursor_y, byte, fg, bg);
                            *cursor_x += CHAR_WIDTH;
                            if *cursor_x + CHAR_WIDTH > *width {
                                *cursor_x = 0;
                                *cursor_y += CHAR_HEIGHT;
                            }
                        }
                    }
                }
            },
            X86DisplayInner::Text(text) => {
                text.print(s);
            }
        }
        Ok(())
    }
}

impl Default for X86Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for VgaTextDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayDriver for X86Display {
    fn name(&self) -> &'static str {
        match &self.inner {
            X86DisplayInner::Vbe { .. } => "VBE Framebuffer Display",
            X86DisplayInner::Text(_) => "VGA Text Mode Display (Fallback)",
        }
    }

    fn init(&mut self) -> Result<(), DisplayError> {
        match &mut self.inner {
            X86DisplayInner::Vbe { .. } => {
                let blue = Self::make_pixel(0, 0, 128);
                self.clear_screen(blue);
            },
            X86DisplayInner::Text(text) => {
                text.init()?;
            }
        }
        Ok(())
    }

    fn get_info(&self) -> DisplayInfo {
        match &self.inner {
            X86DisplayInner::Vbe { width, height, .. } => {
                DisplayInfo {
                    name: "VBE Framebuffer",
                    width: *width,
                    height: *height,
                    supported_modes: {
                        let mut modes = arrayvec::ArrayVec::new();
                        modes.push(DisplayMode {
                            width: *width,
                            height: *height,
                            refresh_rate: 60,
                            pixel_format: PixelFormat::Bgr32,
                            flags: Default::default(),
                        });
                        modes
                    },
                    supported_formats: {
                        let mut formats = arrayvec::ArrayVec::new();
                        formats.push(PixelFormat::Bgr32);
                        formats
                    },
                    capabilities: DisplayCapabilities::default(),
                }
            },
            X86DisplayInner::Text(text) => {
                text.get_info()
            }
        }
    }

    fn set_mode(&mut self, _mode: &DisplayMode) -> Result<(), DisplayError> {
        Ok(())
    }

    fn get_mode(&self) -> DisplayMode {
        match &self.inner {
            X86DisplayInner::Vbe { width, height, .. } => {
                DisplayMode {
                    width: *width,
                    height: *height,
                    refresh_rate: 60,
                    pixel_format: PixelFormat::Bgr32,
                    flags: Default::default(),
                }
            },
            X86DisplayInner::Text(text) => {
                text.get_mode()
            }
        }
    }

    fn create_framebuffer(&mut self, _width: u32, _height: u32, _format: PixelFormat) -> Result<Framebuffer, DisplayError> {
        Ok(Framebuffer {
            width: self.get_info().width,
            height: self.get_info().height,
            stride: match &self.inner {
                X86DisplayInner::Vbe { pitch, .. } => *pitch,
                X86DisplayInner::Text(_) => self.get_info().width * 2,
            },
            format: match &self.inner {
                X86DisplayInner::Vbe { .. } => PixelFormat::Bgr32,
                X86DisplayInner::Text(_) => PixelFormat::Rgb32,
            },
            address: match &self.inner {
                X86DisplayInner::Vbe { framebuffer, .. } => *framebuffer as usize,
                X86DisplayInner::Text(text) => text.buffer_addr,
            },
            size: match &self.inner {
                X86DisplayInner::Vbe { width, height, pitch, .. } => (pitch * height) as usize,
                X86DisplayInner::Text(text) => (text.width * text.height * 2) as usize,
            },
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

// Helper for VGA text mode
impl VgaTextDisplay {
    pub fn print(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_char_at_cursor(byte);
        }
    }

    fn write_char_at_cursor(&mut self, character: u8) {
        match character {
            b'\n' => {
                self.cursor_x = 0;
                self.cursor_y += 1;
                if self.cursor_y >= self.height {
                    self.cursor_y = self.height - 1;
                }
            },
            _ => {
                self.write_char(self.cursor_x, self.cursor_y, character, self.current_color);
                self.cursor_x += 1;
                if self.cursor_x >= self.width {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                }
            }
        }
    }
}
