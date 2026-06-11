// XPARQ OS - Phase 03: Hardware Abstraction Layer
// HAL Sensors module - Phase 3: Hardware Abstraction Layer
// Provides unified sensor interface across ARM and x86 architectures

use bitflags::bitflags;
use arrayvec::ArrayVec;

/// Sensor driver trait
pub trait SensorDriver {
    /// Get driver name
    fn name(&self) -> &'static str;
    
    /// Initialize sensor driver
    fn init(&mut self) -> Result<(), SensorError>;
    
    /// Get sensor information
    fn get_info(&self) -> SensorDeviceInfo;
    
    /// Enable/disable sensor
    fn set_enabled(&mut self, enabled: bool) -> Result<(), SensorError>;
    
    /// Check if sensor is enabled
    fn is_enabled(&self) -> bool;
    
    /// Read sensor data
    fn read(&mut self) -> Result<SensorData, SensorError>;
}

/// Sensor error type
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SensorError {
    #[default]
    /// Hardware failure
    HardwareFailure,
    /// Device not found
    DeviceNotFound,
    /// Unsupported operation
    Unsupported,
    /// Invalid parameter
    InvalidParameter,
}

/// Sensor types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorType {
    /// Accelerometer
    Accelerometer,
    /// Gyroscope
    Gyroscope,
    /// Magnetometer
    Magnetometer,
    /// Temperature sensor
    Temperature,
    /// Humidity sensor
    Humidity,
    /// Light sensor
    Light,
    /// Proximity sensor
    Proximity,
    /// Barometer (pressure)
    Barometer,
}

/// Sensor interface types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorInterface {
    I2C,
    SPI,
    UART,
    Integrated,
}

/// Sensor device information
#[derive(Debug, Clone, Copy)]
pub struct SensorDeviceInfo {
    pub sensor_type: SensorType,
    pub interface: SensorInterface,
    pub vendor_id: u16,
    pub product_id: u16,
    pub model: &'static str,
    pub capabilities: SensorCapabilities,
}

bitflags! {
    /// Sensor capabilities
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct SensorCapabilities: u32 {
        /// Supports high sample rate (>1 kHz)
        const HIGH_RATE = 1 << 0;
        /// Supports low-power mode
        const LOW_POWER = 1 << 1;
    }
}

/// Sensor data (generic container)
#[derive(Debug, Clone, Copy)]
pub enum SensorData {
    /// Accelerometer data (3 axes: x, y, z in g's)
    Accelerometer(f32, f32, f32),
    /// Gyroscope data (3 axes: x, y, z in rad/s)
    Gyroscope(f32, f32, f32),
    /// Magnetometer data (3 axes: x, y, z in T)
    Magnetometer(f32, f32, f32),
    /// Temperature data in degrees Celsius
    Temperature(f32),
    /// Humidity data in %RH
    Humidity(f32),
    /// Light data in lux
    Light(f32),
    /// Proximity distance in cm
    Proximity(f32),
    /// Barometer data in Pascals
    Barometer(f32),
}

/// Sensor manager
pub struct SensorManager {
    /// Registered sensor drivers - simplified for no_std
    drivers: ArrayVec<*const (), 32>,
    /// Active sensors
    sensors: ArrayVec<SensorDeviceHandle, 32>,
    /// Next sensor ID
    next_id: u32,
}

/// Sensor device handle
#[derive(Debug, Clone)]
pub struct SensorDeviceHandle {
    pub id: u32,
    pub driver_name: &'static str,
    pub info: SensorDeviceInfo,
    pub enabled: bool,
}

impl SensorManager {
    /// Create new sensor manager
    pub fn new() -> Self {
        Self {
            drivers: ArrayVec::new(),
            sensors: ArrayVec::new(),
            next_id: 1,
        }
    }
    
    /// Initialize all drivers - simplified for no_std
    pub fn init_all(&mut self) -> Result<(), SensorError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Get sensor by ID
    pub fn get_sensor(&self, id: u32) -> Option<&SensorDeviceHandle> {
        self.sensors.iter().find(|sensor| sensor.id == id)
    }
}

impl Default for SensorManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize sensors subsystem
pub fn init() -> Result<(), SensorError> {
    println!("Initializing sensors subsystem...");
    // Phase 3: Initialize sensor drivers
    Ok(())
}
