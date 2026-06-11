// XPARQ OS - Phase 03: Hardware Abstraction Layer
// HAL USB module - Phase 3: Hardware Abstraction Layer
// Provides unified USB interface across ARM and x86 architectures

use bitflags::bitflags;
use arrayvec::ArrayVec;

/// USB driver trait
pub trait UsbDriver {
    /// Get driver name
    fn name(&self) -> &'static str;
    
    /// Initialize USB driver
    fn init(&mut self) -> Result<(), UsbError>;
    
    /// Get USB host controller information
    fn get_info(&self) -> UsbHostInfo;
    
    /// Enumerate USB devices
    fn enumerate_devices(&mut self) -> ArrayVec<UsbDeviceInfo, 32>;
    
    /// Enable/disable host controller
    fn set_enabled(&mut self, enabled: bool) -> Result<(), UsbError>;
    
    /// Check if host controller is enabled
    fn is_enabled(&self) -> bool;
}

/// USB error type
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum UsbError {
    #[default]
    /// Hardware failure
    HardwareFailure,
    /// Device not found
    DeviceNotFound,
    /// Unsupported operation
    Unsupported,
    /// Invalid parameter
    InvalidParameter,
    /// Transfer failed
    TransferFailed,
    /// Timeout
    Timeout,
}

/// USB device classes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UsbDeviceClass {
    /// Audio device
    Audio,
    /// HID device (keyboard, mouse)
    HID,
    /// Mass storage
    MassStorage,
    /// Network adapter
    Network,
    /// Hub
    Hub,
    /// Vendor-specific
    VendorSpecific,
    /// Unknown
    Unknown,
}

/// USB host controller types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UsbHostControllerType {
    /// UHCI
    UHCI,
    /// OHCI
    OHCI,
    /// EHCI (USB 2.0)
    EHCI,
    /// xHCI (USB 3.x)
    XHCI,
}

/// USB host controller information
#[derive(Debug, Clone, Copy)]
pub struct UsbHostInfo {
    pub controller_type: UsbHostControllerType,
    pub vendor_id: u16,
    pub product_id: u16,
    pub model: &'static str,
    pub capabilities: UsbCapabilities,
}

bitflags! {
    /// USB capabilities
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct UsbCapabilities: u32 {
        /// Supports USB 1.x
        const USB_1 = 1 << 0;
        /// Supports USB 2.0
        const USB_2 = 1 << 1;
        /// Supports USB 3.x
        const USB_3 = 1 << 2;
    }
}

/// USB device information
#[derive(Debug, Clone, Copy)]
pub struct UsbDeviceInfo {
    pub device_class: UsbDeviceClass,
    pub vendor_id: u16,
    pub product_id: u16,
    pub bus_number: u8,
    pub device_address: u8,
    pub speed: UsbSpeed,
}

/// USB device speeds
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UsbSpeed {
    /// Low speed (1.5 Mbps)
    Low,
    /// Full speed (12 Mbps)
    Full,
    /// High speed (480 Mbps)
    High,
    /// Super speed (5 Gbps)
    Super,
    /// Super speed plus (10 Gbps)
    SuperPlus,
}

/// USB manager
pub struct UsbManager {
    /// Registered USB host drivers - simplified for no_std
    drivers: ArrayVec<*const (), 8>,
    /// Active USB host controllers
    controllers: ArrayVec<UsbHostHandle, 8>,
    /// Next host controller ID
    next_id: u32,
}

/// USB host controller handle
#[derive(Debug, Clone)]
pub struct UsbHostHandle {
    pub id: u32,
    pub driver_name: &'static str,
    pub info: UsbHostInfo,
    pub enabled: bool,
}

impl UsbManager {
    /// Create new USB manager
    pub fn new() -> Self {
        Self {
            drivers: ArrayVec::new(),
            controllers: ArrayVec::new(),
            next_id: 1,
        }
    }
    
    /// Initialize all drivers - simplified for no_std
    pub fn init_all(&mut self) -> Result<(), UsbError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Get host controller by ID
    pub fn get_controller(&self, id: u32) -> Option<&UsbHostHandle> {
        self.controllers.iter().find(|controller| controller.id == id)
    }
}

impl Default for UsbManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize USB subsystem
pub fn init() -> Result<(), UsbError> {
    println!("Initializing USB subsystem...");
    // Phase 3: Initialize USB drivers
    Ok(())
}
