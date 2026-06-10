// XPARQ OS - Phase 01: OS & Kernel Foundations
// Hardware Abstraction Layer - Main library
// Provides unified hardware interface across ARM and x86 architectures

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

// Simple println macro for no_std debugging
macro_rules! println {
    ($($arg:tt)*) => {
        // Phase 1: No output in no_std
        // Phase 2: Use actual console/serial output
    };
}

/// HAL errors
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HalError {
    #[default]
    /// Hardware failure
    HardwareFailure,
    /// Device not found
    DeviceNotFound,
    /// Unsupported operation
    Unsupported,
    /// Invalid parameter
    InvalidParameter,
    /// Resource exhausted
    ResourceExhausted,
    /// Permission denied
    PermissionDenied,
    /// Timeout
    Timeout,
}

/// Supported architectures
#[derive(Debug, Clone, Copy)]
pub struct SupportedArchitectures {
    pub arm64: bool,
    pub x86_64: bool,
}

impl SupportedArchitectures {
    pub const ARM64: Self = Self { arm64: true, x86_64: false };
    pub const X86_64: Self = Self { arm64: false, x86_64: true };
    pub const BOTH: Self = Self { arm64: true, x86_64: true };
}

impl core::ops::BitOr for SupportedArchitectures {
    type Output = Self;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            arm64: self.arm64 || rhs.arm64,
            x86_64: self.x86_64 || rhs.x86_64,
        }
    }
}

/// HAL features
#[derive(Debug, Clone, Copy)]
pub struct HalFeatures {
    pub display: bool,
    pub input: bool,
    pub power: bool,
    pub storage: bool,
    pub connectivity: bool,
    pub audio: bool,
    pub sensors: bool,
}

impl HalFeatures {
    pub const DISPLAY: Self = Self { display: true, input: false, power: false, storage: false, connectivity: false, audio: false, sensors: false };
    pub const INPUT: Self = Self { display: false, input: true, power: false, storage: false, connectivity: false, audio: false, sensors: false };
    pub const POWER: Self = Self { display: false, input: false, power: true, storage: false, connectivity: false, audio: false, sensors: false };
    pub const STORAGE: Self = Self { display: false, input: false, power: false, storage: true, connectivity: false, audio: false, sensors: false };
    pub const CONNECTIVITY: Self = Self { display: false, input: false, power: false, storage: false, connectivity: true, audio: false, sensors: false };
    pub const AUDIO: Self = Self { display: false, input: false, power: false, storage: false, connectivity: false, audio: true, sensors: false };
    pub const SENSORS: Self = Self { display: false, input: false, power: false, storage: false, connectivity: false, audio: false, sensors: true };
    
    pub const ALL: Self = Self { 
        display: true, 
        input: true, 
        power: true, 
        storage: true, 
        connectivity: true, 
        audio: true, 
        sensors: true 
    };
}

impl core::ops::BitOr for HalFeatures {
    type Output = Self;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            display: self.display || rhs.display,
            input: self.input || rhs.input,
            power: self.power || rhs.power,
            storage: self.storage || rhs.storage,
            connectivity: self.connectivity || rhs.connectivity,
            audio: self.audio || rhs.audio,
            sensors: self.sensors || rhs.sensors,
        }
    }
}

/// HAL information structure
#[derive(Debug, Clone, Copy)]
pub struct HalInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub supported_architectures: SupportedArchitectures,
    pub features: HalFeatures,
}

/// HAL capabilities
#[derive(Debug, Clone, Copy, Default)]
pub struct HalCapabilities {
    pub max_display_resolution: (u32, u32),
    pub max_input_devices: usize,
    pub max_storage_devices: usize,
    pub power_management: bool,
    pub hardware_acceleration: bool,
    pub multi_touch: bool,
    pub gesture_recognition: bool,
}

// Core HAL modules
pub mod display;
pub mod input;
pub mod power;
pub mod storage;

// Architecture-specific modules
#[cfg(target_arch = "aarch64")]
pub mod arm64;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

// Re-export main traits for easy access
pub use display::{DisplayDriver, DisplayInfo, DisplayMode, PixelFormat};
pub use input::{InputDriver, InputEvent, InputDeviceType, InputEventKind};
pub use power::{PowerDriver, PowerState, BatteryInfo, PowerPolicy};
pub use storage::{StorageDriver, StorageDevice, StorageInfo, StorageError};

// Error type conversions
impl From<crate::display::DisplayError> for HalError {
    fn from(_error: crate::display::DisplayError) -> Self {
        HalError::HardwareFailure
    }
}

impl From<crate::input::InputError> for HalError {
    fn from(_error: crate::input::InputError) -> Self {
        HalError::HardwareFailure
    }
}

impl From<crate::power::PowerError> for HalError {
    fn from(_error: crate::power::PowerError) -> Self {
        HalError::HardwareFailure
    }
}

impl From<crate::storage::StorageError> for HalError {
    fn from(_error: crate::storage::StorageError) -> Self {
        HalError::HardwareFailure
    }
}

/// HAL version information
pub const HAL_VERSION: &str = "0.1.0";
pub const HAL_NAME: &str = "XPARQ Hardware Abstraction Layer";

/// Get HAL information
pub fn get_hal_info() -> HalInfo {
    HalInfo {
        name: HAL_NAME,
        version: HAL_VERSION,
        supported_architectures: SupportedArchitectures::ARM64 | SupportedArchitectures::X86_64,
        features: HalFeatures::DISPLAY | HalFeatures::INPUT | HalFeatures::POWER | HalFeatures::STORAGE,
    }
}

/// HAL initialization
pub fn init() -> Result<(), HalError> {
    println!("Initializing HAL...");
    
    // Initialize display subsystem
    display::init()?;
    
    // Initialize input subsystem
    input::init()?;
    
    // Initialize power subsystem
    power::init()?;
    
    // Initialize storage subsystem
    storage::init()?;
    
    println!("HAL initialized");
    Ok(())
}

/// Device manager for HAL
pub struct DeviceManager {
    /// Registered display drivers - simplified for no_std
    display_drivers: arrayvec::ArrayVec<*const (), 8>,
    /// Registered input drivers - simplified for no_std
    input_drivers: arrayvec::ArrayVec<*const (), 16>,
    /// Registered power drivers - simplified for no_std
    power_drivers: arrayvec::ArrayVec<*const (), 4>,
    /// Registered storage drivers - simplified for no_std
    storage_drivers: arrayvec::ArrayVec<*const (), 8>,
}

impl DeviceManager {
    /// Create new device manager
    pub fn new() -> Self {
        Self {
            display_drivers: arrayvec::ArrayVec::new(),
            input_drivers: arrayvec::ArrayVec::new(),
            power_drivers: arrayvec::ArrayVec::new(),
            storage_drivers: arrayvec::ArrayVec::new(),
        }
    }
    
    /// Register display driver - simplified for no_std
    pub fn register_display_driver(&mut self, _driver: *const ()) -> Result<(), HalError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Register input driver - simplified for no_std
    pub fn register_input_driver(&mut self, _driver: *const ()) -> Result<(), HalError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Register power driver - simplified for no_std
    pub fn register_power_driver(&mut self, _driver: *const ()) -> Result<(), HalError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Register storage driver - simplified for no_std
    pub fn register_storage_driver(&mut self, _driver: *const ()) -> Result<(), HalError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Get display driver by name - simplified for no_std
    pub fn get_display_driver(&self, _name: &str) -> Option<&dyn DisplayDriver> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        None
    }
    
    /// Get input driver by name - simplified for no_std
    pub fn get_input_driver(&self, _name: &str) -> Option<&dyn InputDriver> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        None
    }
    
    /// Get power driver by name - simplified for no_std
    pub fn get_power_driver(&self, _name: &str) -> Option<&dyn PowerDriver> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        None
    }
    
    /// Get storage driver by name - simplified for no_std
    pub fn get_storage_driver(&self, _name: &str) -> Option<&dyn StorageDriver> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        None
    }
    
    /// List all display drivers - simplified for no_std
    pub fn list_display_drivers(&self) -> arrayvec::ArrayVec<&str, 8> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        arrayvec::ArrayVec::new()
    }
    
    /// List all input drivers - simplified for no_std
    pub fn list_input_drivers(&self) -> arrayvec::ArrayVec<&str, 16> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        arrayvec::ArrayVec::new()
    }
    
    /// List all power drivers - simplified for no_std
    pub fn list_power_drivers(&self) -> arrayvec::ArrayVec<&str, 4> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        arrayvec::ArrayVec::new()
    }
    
    /// List all storage drivers - simplified for no_std
    pub fn list_storage_drivers(&self) -> arrayvec::ArrayVec<&str, 8> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        arrayvec::ArrayVec::new()
    }
}

/// Global device manager
static mut DEVICE_MANAGER: Option<DeviceManager> = None;
static mut DEVICE_MANAGER_INITIALIZED: bool = false;

/// Initialize global device manager
pub fn init_device_manager() -> Result<(), HalError> {
    unsafe {
        if DEVICE_MANAGER_INITIALIZED {
            return Ok(());
        }
        
        DEVICE_MANAGER = Some(DeviceManager::new());
        DEVICE_MANAGER_INITIALIZED = true;
        
        Ok(())
    }
}

/// Get global device manager
pub fn get_device_manager() -> Option<&'static DeviceManager> {
    unsafe { DEVICE_MANAGER.as_ref() }
}

/// Get mutable global device manager
pub fn get_device_manager_mut() -> Option<&'static mut DeviceManager> {
    unsafe { DEVICE_MANAGER.as_mut() }
}

/// HAL utilities
pub mod utils {
    use super::*;
    
    /// Check if architecture is supported
    pub fn is_architecture_supported() -> bool {
        #[cfg(target_arch = "aarch64")]
        return true;
        #[cfg(target_arch = "x86_64")]
        return true;
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        return false;
    }
    
    /// Get current architecture
    pub fn get_current_architecture() -> &'static str {
        #[cfg(target_arch = "aarch64")]
        return "ARM64";
        #[cfg(target_arch = "x86_64")]
        return "x86-64";
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        return "Unknown";
    }
    
    /// Check if feature is supported
    pub fn is_feature_supported(feature: HalFeatures) -> bool {
        let hal_info = get_hal_info();
        
        // Check if all requested features are supported
        if feature.display && !hal_info.features.display { return false; }
        if feature.input && !hal_info.features.input { return false; }
        if feature.power && !hal_info.features.power { return false; }
        if feature.storage && !hal_info.features.storage { return false; }
        if feature.connectivity && !hal_info.features.connectivity { return false; }
        if feature.audio && !hal_info.features.audio { return false; }
        if feature.sensors && !hal_info.features.sensors { return false; }
        
        true
    }
    
    /// Get HAL capabilities
    pub fn get_capabilities() -> HalCapabilities {
        HalCapabilities {
            max_display_resolution: (1920, 1080), // Phase 1: 1080p
            max_input_devices: 16,
            max_storage_devices: 8,
            power_management: true,
            hardware_acceleration: false, // Phase 1: No GPU acceleration
            multi_touch: true,
            gesture_recognition: false, // Phase 1: No gestures
        }
    }
}

/// HAL capabilities
#[derive(Debug, Clone, Copy)]
pub struct HalCapabilities {
    pub max_display_resolution: (u32, u32),
    pub max_input_devices: usize,
    pub max_storage_devices: usize,
    pub power_management: bool,
    pub hardware_acceleration: bool,
    pub multi_touch: bool,
    pub gesture_recognition: bool,
}



use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}


