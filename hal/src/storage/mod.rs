// XPARQ OS - Phase 01: OS & Kernel Foundations
// HAL Storage module - Phase 3: Hardware Abstraction Layer
// Provides unified storage interface across ARM and x86 architectures
use arrayvec::ArrayVec;

/// Storage driver trait
pub trait StorageDriver {
    /// Get driver name
    fn name(&self) -> &'static str;
    
    /// Initialize storage driver
    fn init(&mut self) -> Result<(), StorageError>;
    
    /// Get storage device information
    fn get_devices(&self) -> ArrayVec<StorageDevice, 8>;
    
    /// Read from storage device
    fn read(&mut self, device_id: u32, lba: u64, buffer: &mut [u8]) -> Result<(), StorageError>;
    
    /// Write to storage device
    fn write(&mut self, device_id: u32, lba: u64, data: &[u8]) -> Result<(), StorageError>;
    
    /// Flush write cache
    fn flush(&mut self, device_id: u32) -> Result<(), StorageError>;
    
    /// Get device status
    fn get_device_status(&self, device_id: u32) -> Option<StorageStatus>;
    
    /// Get device statistics
    fn get_device_statistics(&self, device_id: u32) -> Option<StorageStatistics>;
    
    /// Erase blocks
    fn erase(&mut self, device_id: u32, lba: u64, count: u64) -> Result<(), StorageError>;
    
    /// Trim blocks
    fn trim(&mut self, device_id: u32, lba: u64, count: u64) -> Result<(), StorageError>;
    
    /// Set power mode
    fn set_power_mode(&mut self, device_id: u32, mode: PowerMode) -> Result<(), StorageError>;
    
    /// Get power mode
    fn get_power_mode(&self, device_id: u32) -> Option<PowerMode>;
}

/// Storage device
#[derive(Debug, Clone, Copy)]
pub struct StorageDevice {
    pub id: u32,
    pub name: &'static str,
    pub device_type: StorageType,
    pub interface: StorageInterface,
    pub info: StorageInfo,
    pub capabilities: StorageCapabilities,
}

/// Storage types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageType {
    HardDisk,
    SolidState,
    Flash,
    Optical,
    MagneticTape,
    RAMDisk,
    Virtual,
}

/// Storage interfaces
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageInterface {
    SATA,
    NVMe,
    USB,
    SD,
    eMMC,
    UFS,
    SCSI,
    IDE,
    Virtual,
}

/// Storage information
#[derive(Debug, Clone, Copy)]
pub struct StorageInfo {
    pub model: &'static str,
    pub serial: &'static str,
    pub firmware: &'static str,
    pub capacity: u64,         // Bytes
    pub block_size: u32,       // Bytes
    pub sector_size: u32,      // Bytes
    pub total_blocks: u64,
    pub usable_blocks: u64,
    pub temperature: Option<i16>, // Degrees Celsius
    pub health: StorageHealth,
}

/// Storage capabilities
#[derive(Debug, Clone, Copy)]
pub struct StorageCapabilities {
    pub read_cache: bool,
    pub write_cache: bool,
    pub command_queueing: bool,
    pub trim_support: bool,
    pub encryption: bool,
    pub power_management: bool,
    pub smart_support: bool,
    pub wear_leveling: bool,
}

/// Storage health
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageHealth {
    Unknown,
    Good,
    Fair,
    Poor,
    Critical,
    Failed,
}

/// Storage status
#[derive(Debug, Clone, Copy)]
pub struct StorageStatus {
    pub device_id: u32,
    pub status: DeviceStatus,
    pub temperature: Option<i16>,
    pub busy: bool,
    pub error_count: u32,
    pub last_error: Option<StorageError>,
}

/// Device status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Standby,
    Sleeping,
    Fault,
    Initializing,
}

/// Storage statistics
#[derive(Debug, Clone, Copy)]
pub struct StorageStatistics {
    pub device_id: u32,
    pub reads: u64,
    pub writes: u64,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub errors: u32,
    pub uptime: u64,         // Seconds
    pub power_on_hours: u32,  // Hours
    pub wear_level: Option<u8>, // Percentage (for SSD/Flash)
    pub endurance: Option<u64>,  // Remaining write cycles
}

/// Power modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PowerMode {
    Active,
    Idle,
    Standby,
    Sleep,
    DeepSleep,
    Off,
}

/// Storage errors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageError {
    /// Device not found
    DeviceNotFound,
    /// Invalid LBA
    InvalidLBA,
    /// Read error
    ReadError,
    /// Write error
    WriteError,
    /// Media error
    MediaError,
    /// Hardware failure
    HardwareFailure,
    /// Timeout
    Timeout,
    /// Permission denied
    PermissionDenied,
    /// Buffer overflow
    BufferOverflow,
    /// Unsupported operation
    Unsupported,
    /// No space left
    NoSpaceLeft,
    /// Write protected
    WriteProtected,
}

/// Storage manager
pub struct StorageManager {
    /// Registered storage drivers - simplified for no_std
    drivers: ArrayVec<*const (), 8>,
    /// Active storage devices
    devices: ArrayVec<StorageDevice, 16>,
    /// Device status cache
    device_status: ArrayVec<StorageStatus, 16>,
    /// Statistics cache
    device_statistics: ArrayVec<StorageStatistics, 16>,
    /// Next device ID
    next_id: u32,
}

impl StorageManager {
    /// Create new storage manager
    pub fn new() -> Self {
        Self {
            drivers: ArrayVec::new(),
            devices: ArrayVec::new(),
            device_status: ArrayVec::new(),
            device_statistics: ArrayVec::new(),
            next_id: 1,
        }
    }
    
    /// Register storage driver - simplified for no_std
    pub fn register_driver(&mut self, _driver: *const ()) -> Result<(), StorageError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Initialize all drivers - simplified for no_std
    pub fn init_all(&mut self) -> Result<(), StorageError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Enumerate storage devices - simplified for no_std
    pub fn enumerate_devices(&mut self) -> Result<(), StorageError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Get storage devices
    pub fn get_devices(&self) -> &ArrayVec<StorageDevice, 16> {
        &self.devices
    }
    
    /// Get device by ID
    pub fn get_device(&self, id: u32) -> Option<&StorageDevice> {
        self.devices.iter().find(|device| device.id == id)
    }
    
    /// Read from device - simplified for no_std
    pub fn read(&mut self, _device_id: u32, _lba: u64, _buffer: &mut [u8]) -> Result<(), StorageError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Write to device - simplified for no_std
    pub fn write(&mut self, _device_id: u32, _lba: u64, _data: &[u8]) -> Result<(), StorageError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Flush device cache - simplified for no_std
    pub fn flush(&mut self, _device_id: u32) -> Result<(), StorageError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Erase blocks - simplified for no_std
    pub fn erase(&mut self, _device_id: u32, _lba: u64, _count: u64) -> Result<(), StorageError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Trim blocks - simplified for no_std
    pub fn trim(&mut self, _device_id: u32, _lba: u64, _count: u64) -> Result<(), StorageError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Set power mode - simplified for no_std
    pub fn set_power_mode(&mut self, _device_id: u32, _mode: PowerMode) -> Result<(), StorageError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Get device status
    pub fn get_device_status(&self, device_id: u32) -> Option<&StorageStatus> {
        self.device_status.iter().find(|status| status.device_id == device_id)
    }
    
    /// Get device statistics
    pub fn get_device_statistics(&self, device_id: u32) -> Option<&StorageStatistics> {
        self.device_statistics.iter().find(|stats| stats.device_id == device_id)
    }
    
    /// Update all device statuses - simplified for no_std
    pub fn update_device_statuses(&mut self) -> Result<(), StorageError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Update all device statistics - simplified for no_std
    pub fn update_device_statistics(&mut self) -> Result<(), StorageError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Get devices by type
    pub fn get_devices_by_type(&self, storage_type: StorageType) -> ArrayVec<&StorageDevice, 16> {
        self.devices.iter()
            .filter(|device| device.device_type == storage_type)
            .collect()
    }
    
    /// Get devices by interface
    pub fn get_devices_by_interface(&self, interface: StorageInterface) -> ArrayVec<&StorageDevice, 16> {
        self.devices.iter()
            .filter(|device| device.interface == interface)
            .collect()
    }
}

/// Global storage manager
static mut STORAGE_MANAGER: Option<StorageManager> = None;
static mut STORAGE_MANAGER_INITIALIZED: bool = false;

/// Initialize storage subsystem
pub fn init() -> Result<(), super::HalError> {
    println!("Initializing storage subsystem...");
    
    unsafe {
        if STORAGE_MANAGER_INITIALIZED {
            return Ok(());
        }
        
        STORAGE_MANAGER = Some(StorageManager::new());
        STORAGE_MANAGER_INITIALIZED = true;
        
        // Initialize architecture-specific storage drivers
        #[cfg(target_arch = "aarch64")]
        {
            // Phase 1: Dummy ARM64 storage drivers
            // Phase 2: Real ARM64 eMMC, UFS, SD card drivers
            println!("Initializing ARM64 storage drivers");
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            // Phase 1: Dummy x86-64 storage drivers
            // Phase 2: Real x86-64 SATA, NVMe, USB storage drivers
            println!("Initializing x86-64 storage drivers");
        }
        
        if let Some(manager) = &mut STORAGE_MANAGER {
            manager.init_all()?;
            manager.enumerate_devices()?;
        }
    }
    
    println!("Storage subsystem initialized");
    Ok(())
}

/// Get global storage manager
pub fn get_storage_manager() -> Option<&'static StorageManager> {
    unsafe { STORAGE_MANAGER.as_ref() }
}

/// Get mutable global storage manager
pub fn get_storage_manager_mut() -> Option<&'static mut StorageManager> {
    unsafe { STORAGE_MANAGER.as_mut() }
}

/// Storage utilities
pub mod utils {
    use super::*;
    
    /// Convert bytes to human readable format
    pub fn format_bytes(bytes: u64) -> (f64, &'static str) {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        (size, UNITS[unit_index])
    }
    
    /// Calculate storage utilization
    pub fn calculate_utilization(used_blocks: u64, total_blocks: u64) -> u8 {
        if total_blocks == 0 {
            return 0;
        }
        
        ((used_blocks * 100) / total_blocks) as u8
    }
    
    /// Estimate remaining life (for SSD/Flash)
    pub fn estimate_remaining_life(wear_level: u8) -> u8 {
        100u8.saturating_sub(wear_level)
    }
    
    /// Check if device needs maintenance
    pub fn needs_maintenance(device: &StorageDevice, stats: &StorageStatistics) -> bool {
        // Check error rate
        if stats.errors > 100 {
            return true;
        }
        
        // Check wear level for SSD/Flash
        if matches!(device.device_type, StorageType::SolidState | StorageType::Flash) {
            if let Some(wear_level) = stats.wear_level {
                if wear_level > 80 {
                    return true;
                }
            }
        }
        
        // Check temperature
        if let Some(temp) = device.info.temperature {
            if temp > 70 {
                return true;
            }
        }
        
        false
    }
    
    /// Get optimal block size for device
    pub fn get_optimal_block_size(device: &StorageDevice) -> u32 {
        match device.device_type {
            StorageType::SolidState => device.info.block_size.max(4096),
            StorageType::Flash => device.info.block_size.max(4096),
            StorageType::HardDisk => device.info.block_size.max(512),
            _ => device.info.block_size,
        }
    }
    
    /// Validate LBA range
    pub fn validate_lba_range(device: &StorageDevice, lba: u64, count: u64) -> bool {
        if lba >= device.info.total_blocks {
            return false;
        }
        
        if count == 0 {
            return false;
        }
        
        if lba + count > device.info.total_blocks {
            return false;
        }
        
        true
    }
    
    /// Calculate optimal transfer size
    pub fn calculate_optimal_transfer_size(device: &StorageDevice) -> usize {
        // Phase 1: Use 64KB as default
        // Phase 2: Calculate based on device capabilities
        
        match device.interface {
            StorageInterface::NVMe => 64 * 1024,     // 64KB
            StorageInterface::SATA => 32 * 1024,     // 32KB
            StorageInterface::USB => 16 * 1024,      // 16KB
            StorageInterface::SD => 4 * 1024,        // 4KB
            StorageInterface::eMMC => 8 * 1024,      // 8KB
            StorageInterface::UFS => 16 * 1024,      // 16KB
            _ => 4 * 1024,                           // 4KB default
        }
    }
}

impl Default for StorageCapabilities {
    fn default() -> Self {
        Self {
            read_cache: true,
            write_cache: true,
            command_queueing: false,
            trim_support: false,
            encryption: false,
            power_management: true,
            smart_support: false,
            wear_leveling: false,
        }
    }
}

impl Default for StorageStatistics {
    fn default() -> Self {
        Self {
            device_id: 0,
            reads: 0,
            writes: 0,
            read_bytes: 0,
            write_bytes: 0,
            errors: 0,
            uptime: 0,
            power_on_hours: 0,
            wear_level: None,
            endurance: None,
        }
    }
}

impl Default for StorageStatus {
    fn default() -> Self {
        Self {
            device_id: 0,
            status: DeviceStatus::Offline,
            temperature: None,
            busy: false,
            error_count: 0,
            last_error: None,
        }
    }
}
