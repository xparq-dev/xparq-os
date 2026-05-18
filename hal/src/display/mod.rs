// XPARQ OS - Phase 01: OS & Kernel Foundations
// HAL Display module - Phase 3: Hardware Abstraction Layer
// Provides unified display interface across ARM and x86 architectures

#![no_std]

use bitflags::bitflags;
use arrayvec::ArrayVec;

/// Display driver trait
pub trait DisplayDriver {
    /// Get driver name
    fn name(&self) -> &'static str;
    
    /// Initialize display driver
    fn init(&mut self) -> Result<(), DisplayError>;
    
    /// Get display information
    fn get_info(&self) -> DisplayInfo;
    
    /// Set display mode
    fn set_mode(&mut self, mode: &DisplayMode) -> Result<(), DisplayError>;
    
    /// Get current display mode
    fn get_mode(&self) -> DisplayMode;
    
    /// Create framebuffer
    fn create_framebuffer(&mut self, width: u32, height: u32, format: PixelFormat) -> Result<Framebuffer, DisplayError>;
    
    /// Present framebuffer to display
    fn present_framebuffer(&mut self, framebuffer: &Framebuffer) -> Result<(), DisplayError>;
    
    /// Set backlight brightness
    fn set_backlight(&mut self, brightness: u8) -> Result<(), DisplayError>;
    
    /// Get backlight brightness
    fn get_backlight(&self) -> u8;
    
    /// Enable/disable display
    fn set_power(&mut self, enabled: bool) -> Result<(), DisplayError>;
    
    /// Check if display is powered
    fn is_powered(&self) -> bool;
}

/// Display information
#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    pub supported_modes: ArrayVec<DisplayMode, 16>,
    pub supported_formats: ArrayVec<PixelFormat, 8>,
    pub capabilities: DisplayCapabilities,
}

/// Display capabilities
#[derive(Debug, Clone, Copy)]
pub struct DisplayCapabilities {
    pub hardware_acceleration: bool,
    pub vsync: bool,
    pub double_buffering: bool,
    pub gamma_correction: bool,
    pub color_calibration: bool,
    pub touch_support: bool,
    pub hdr_support: bool,
}

/// Display mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub pixel_format: PixelFormat,
    pub flags: DisplayModeFlags,
}

/// Display mode flags
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct DisplayModeFlags: u32 {
        const INTERLACED = 0x0001;
        const DOUBLE_SCAN = 0x0002;
        const HSYNC_NEGATIVE = 0x0004;
        const VSYNC_NEGATIVE = 0x0008;
        const CSYNC = 0x0010;
        const EXTERNAL_SYNC = 0x0020;
        const BROADCAST = 0x0040;
        const PIXEL_MULTIPLEX = 0x0080;
        const DOUBLE_CLOCK = 0x0100;
        const HALVE_CLOCK = 0x0200;
    }
}

/// Pixel format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelFormat {
    Rgb32,
    Bgr32,
    Rgb24,
    Bgr24,
    Rgb16,
    Rgb15,
    Argb32,
    Abgr32,
}

/// Framebuffer
#[derive(Debug)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
    pub address: usize,
    pub size: usize,
    pub flags: FramebufferFlags,
}

/// Framebuffer flags
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct FramebufferFlags: u32 {
        const READ_ONLY = 0x0001;
        const WRITE_ONLY = 0x0002;
        const CACHED = 0x0004;
        const UNCACHED = 0x0008;
        const WRITE_COMBINING = 0x0010;
        const DMA_CAPABLE = 0x0020;
        const PROTECTED = 0x0040;
        const SHARED = 0x0080;
    }
}

/// Display errors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayError {
    /// Invalid mode
    InvalidMode,
    /// Unsupported format
    UnsupportedFormat,
    /// Out of memory
    OutOfMemory,
    /// Hardware failure
    HardwareFailure,
    /// Timeout
    Timeout,
    /// Permission denied
    PermissionDenied,
    /// Invalid parameter
    InvalidParameter,
    /// Device not found
    DeviceNotFound,
}

/// Display manager
pub struct DisplayManager {
    /// Registered display drivers - simplified for no_std
    drivers: ArrayVec<*const (), 8>,
    /// Active displays
    displays: ArrayVec<DisplayHandle, 4>,
    /// Next display ID
    next_id: u32,
}

/// Display handle
#[derive(Debug, Clone)]
pub struct DisplayHandle {
    pub id: u32,
    pub driver_name: &'static str,
    pub info: DisplayInfo,
    pub current_mode: DisplayMode,
    pub powered: bool,
}

impl DisplayManager {
    /// Create new display manager
    pub fn new() -> Self {
        Self {
            drivers: ArrayVec::new(),
            displays: ArrayVec::new(),
            next_id: 1,
        }
    }
    
    /// Register display driver - simplified for no_std
    pub fn register_driver(&mut self, _driver: *const ()) -> Result<(), DisplayError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Initialize all drivers - simplified for no_std
    pub fn init_all(&mut self) -> Result<(), DisplayError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Enumerate displays - simplified for no_std
    pub fn enumerate_displays(&mut self) -> Result<(), DisplayError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Get display by ID
    pub fn get_display(&self, id: u32) -> Option<&DisplayHandle> {
        self.displays.iter().find(|display| display.id == id)
    }
    
    /// Get display by name
    pub fn get_display_by_name(&self, name: &str) -> Option<&DisplayHandle> {
        self.displays.iter().find(|display| display.info.name == name)
    }
    
    /// Set display mode - simplified for no_std
    pub fn set_mode(&mut self, _id: u32, _mode: &DisplayMode) -> Result<(), DisplayError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Create framebuffer - simplified for no_std
    pub fn create_framebuffer(&mut self, _id: u32, _width: u32, _height: u32, _format: PixelFormat) -> Result<Framebuffer, DisplayError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(Framebuffer {
            width: 1024,
            height: 768,
            stride: 1024,
            format: PixelFormat::Rgb24,
            address: 0,
            size: 1024 * 768 * 3,
            flags: FramebufferFlags::empty(),
        })
    }
    
    /// Present framebuffer - simplified for no_std
    pub fn present_framebuffer(&mut self, _id: u32, _framebuffer: &Framebuffer) -> Result<(), DisplayError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Set backlight - simplified for no_std
    pub fn set_backlight(&mut self, _id: u32, _brightness: u8) -> Result<(), DisplayError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Get backlight - simplified for no_std
    pub fn get_backlight(&self, _id: u32) -> Result<u8, DisplayError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(128)
    }
    
    /// Set power - simplified for no_std
    pub fn set_power(&mut self, _id: u32, _enabled: bool) -> Result<(), DisplayError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// List all displays
    pub fn list_displays(&self) -> ArrayVec<&DisplayHandle, 4> {
        self.displays.iter().collect()
    }
}

/// Global display manager
static mut DISPLAY_MANAGER: Option<DisplayManager> = None;
static mut DISPLAY_MANAGER_INITIALIZED: bool = false;

/// Initialize display subsystem
pub fn init() -> Result<(), super::HalError> {
    println!("Initializing display subsystem...");
    
    unsafe {
        if DISPLAY_MANAGER_INITIALIZED {
            return Ok(());
        }
        
        DISPLAY_MANAGER = Some(DisplayManager::new());
        DISPLAY_MANAGER_INITIALIZED = true;
        
        // Initialize architecture-specific display drivers
        #[cfg(target_arch = "aarch64")]
        {
            // Phase 1: Dummy ARM64 display driver
            // Phase 2: Real ARM64 Mali GPU driver
            println!("Initializing ARM64 display drivers");
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            // Phase 1: Dummy x86-64 display driver
            // Phase 2: Real x86-64 Intel/AMD GPU driver
            println!("Initializing x86-64 display drivers");
        }
        
        if let Some(manager) = &mut DISPLAY_MANAGER {
            manager.init_all()?;
            manager.enumerate_displays()?;
        }
    }
    
    println!("Display subsystem initialized");
    Ok(())
}

/// Get global display manager
pub fn get_display_manager() -> Option<&'static DisplayManager> {
    unsafe { DISPLAY_MANAGER.as_ref() }
}

/// Get mutable global display manager
pub fn get_display_manager_mut() -> Option<&'static mut DisplayManager> {
    unsafe { DISPLAY_MANAGER.as_mut() }
}

/// Display utilities
pub mod utils {
    use super::*;
    
    /// Calculate framebuffer size
    pub fn calculate_framebuffer_size(width: u32, height: u32, format: PixelFormat) -> usize {
        let bytes_per_pixel = match format {
            PixelFormat::Rgb32 | PixelFormat::Bgr32 | PixelFormat::Argb32 | PixelFormat::Abgr32 => 4,
            PixelFormat::Rgb24 | PixelFormat::Bgr24 => 3,
            PixelFormat::Rgb16 => 2,
            PixelFormat::Rgb15 => 2,
        };
        
        (width * height * bytes_per_pixel) as usize
    }
    
    /// Calculate stride
    pub fn calculate_stride(width: u32, format: PixelFormat) -> u32 {
        let bytes_per_pixel = match format {
            PixelFormat::Rgb32 | PixelFormat::Bgr32 | PixelFormat::Argb32 | PixelFormat::Abgr32 => 4,
            PixelFormat::Rgb24 | PixelFormat::Bgr24 => 3,
            PixelFormat::Rgb16 => 2,
            PixelFormat::Rgb15 => 2,
        };
        
        width * bytes_per_pixel
    }
    
    /// Check if mode is supported
    pub fn is_mode_supported(info: &DisplayInfo, mode: &DisplayMode) -> bool {
        // Check if resolution is supported
        if mode.width > info.width || mode.height > info.height {
            return false;
        }
        
        // Check if format is supported
        if !info.supported_formats.contains(&mode.pixel_format) {
            return false;
        }
        
        // Check if exact mode is supported
        info.supported_modes.contains(mode)
    }
    
    /// Find best matching mode
    pub fn find_best_mode(info: &DisplayInfo, target_width: u32, target_height: u32, format: PixelFormat) -> Option<DisplayMode> {
        let mut best_mode = None;
        let mut best_score = 0;
        
        for mode in &info.supported_modes {
            if mode.pixel_format != format {
                continue;
            }
            
            let score = calculate_mode_score(mode, target_width, target_height);
            if score > best_score {
                best_score = score;
                best_mode = Some(*mode);
            }
        }
        
        best_mode
    }
    
    /// Calculate mode score (higher is better)
    fn calculate_mode_score(mode: &DisplayMode, target_width: u32, target_height: u32) -> u32 {
        let width_diff = if mode.width >= target_width { mode.width - target_width } else { target_width - mode.width };
        let height_diff = if mode.height >= target_height { mode.height - target_height } else { target_height - mode.height };
        
        // Prefer modes that meet or exceed the target
        let size_bonus = if mode.width >= target_width && mode.height >= target_height { 1000 } else { 0 };
        
        // Prefer higher refresh rates
        let refresh_bonus = mode.refresh_rate * 10;
        
        // Penalize size differences
        let size_penalty = width_diff + height_diff;
        
        size_bonus + refresh_bonus - size_penalty
    }
}

impl Default for DisplayCapabilities {
    fn default() -> Self {
        Self {
            hardware_acceleration: false,
            vsync: true,
            double_buffering: true,
            gamma_correction: false,
            color_calibration: false,
            touch_support: true,
            hdr_support: false,
        }
    }
}

impl Default for DisplayModeFlags {
    fn default() -> Self {
        Self::empty()
    }
}

impl Default for FramebufferFlags {
    fn default() -> Self {
        Self::WRITE_COMBINING | Self::DMA_CAPABLE
    }
}
