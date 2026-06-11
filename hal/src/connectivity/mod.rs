// XPARQ OS - Phase 03: Hardware Abstraction Layer
// HAL Connectivity module - Phase 3: Hardware Abstraction Layer
// Provides unified connectivity interface across ARM and x86 architectures

use bitflags::bitflags;
use arrayvec::ArrayVec;

/// Connectivity driver trait
pub trait ConnectivityDriver {
    /// Get driver name
    fn name(&self) -> &'static str;
    
    /// Initialize connectivity driver
    fn init(&mut self) -> Result<(), ConnectivityError>;
    
    /// Get connectivity device information
    fn get_info(&self) -> ConnectivityDeviceInfo;
    
    /// Check if device is connected
    fn is_connected(&self) -> bool;
    
    /// Connect to network/device
    fn connect(&mut self) -> Result<(), ConnectivityError>;
    
    /// Disconnect
    fn disconnect(&mut self) -> Result<(), ConnectivityError>;
    
    /// Send data
    fn send(&mut self, data: &[u8]) -> Result<usize, ConnectivityError>;
    
    /// Receive data
    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, ConnectivityError>;
    
    /// Enable/disable device
    fn set_enabled(&mut self, enabled: bool) -> Result<(), ConnectivityError>;
    
    /// Check if device is enabled
    fn is_enabled(&self) -> bool;
}

/// Connectivity error type
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ConnectivityError {
    #[default]
    /// Hardware failure
    HardwareFailure,
    /// Device not found
    DeviceNotFound,
    /// Unsupported operation
    Unsupported,
    /// Invalid parameter
    InvalidParameter,
    /// Connection failed
    ConnectionFailed,
    /// Disconnected
    Disconnected,
    /// Timeout
    Timeout,
}

/// Connectivity device types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectivityDeviceType {
    /// WiFi adapter
    WiFi,
    /// Bluetooth adapter
    Bluetooth,
    /// Ethernet adapter
    Ethernet,
    /// UWB adapter
    UWB,
    /// Cellular modem
    Cellular,
}

/// Connectivity interface types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectivityInterface {
    PCIe,
    USB,
    SDIO,
    UART,
    I2C,
    SPI,
    Integrated,
}

/// Connectivity device information
#[derive(Debug, Clone, Copy)]
pub struct ConnectivityDeviceInfo {
    pub device_type: ConnectivityDeviceType,
    pub interface: ConnectivityInterface,
    pub vendor_id: u16,
    pub product_id: u16,
    pub model: &'static str,
    pub serial: &'static str,
    pub mac_address: [u8; 6],
    pub capabilities: ConnectivityCapabilities,
}

bitflags! {
    /// Connectivity capabilities
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct ConnectivityCapabilities: u32 {
        /// Supports WiFi 6/6E
        const WIFI_6 = 1 << 0;
        /// Supports Bluetooth 5.x
        const BLUETOOTH_5 = 1 << 1;
        /// Supports Ethernet 1G/10G
        const GIGABIT_ETHERNET = 1 << 2;
        /// Supports UWB
        const UWB = 1 << 3;
        /// Supports 5G cellular
        const FIVE_G = 1 << 4;
    }
}

/// Connectivity manager
pub struct ConnectivityManager {
    /// Registered connectivity drivers - simplified for no_std
    drivers: ArrayVec<*const (), 8>,
    /// Active connectivity devices
    devices: ArrayVec<ConnectivityDeviceHandle, 8>,
    /// Next device ID
    next_id: u32,
}

/// Connectivity device handle
#[derive(Debug, Clone)]
pub struct ConnectivityDeviceHandle {
    pub id: u32,
    pub driver_name: &'static str,
    pub info: ConnectivityDeviceInfo,
    pub connected: bool,
    pub enabled: bool,
}

impl ConnectivityManager {
    /// Create new connectivity manager
    pub fn new() -> Self {
        Self {
            drivers: ArrayVec::new(),
            devices: ArrayVec::new(),
            next_id: 1,
        }
    }
    
    /// Initialize all drivers - simplified for no_std
    pub fn init_all(&mut self) -> Result<(), ConnectivityError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Get device by ID
    pub fn get_device(&self, id: u32) -> Option<&ConnectivityDeviceHandle> {
        self.devices.iter().find(|device| device.id == id)
    }
}

impl Default for ConnectivityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize connectivity subsystem
pub fn init() -> Result<(), ConnectivityError> {
    println!("Initializing connectivity subsystem...");
    // Phase 3: Initialize connectivity drivers
    Ok(())
}
