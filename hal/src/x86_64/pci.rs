
// XPARQ OS - x86_64 PCIe Bus Enumeration (ECAM)
// Implementation using ECAM at 0xE0000000

use core::ptr::{read_volatile, write_volatile};
use arrayvec::ArrayVec;

/// PCIe Configuration Space Address (ECAM - Enhanced Configuration Access Mechanism)
const ECAM_BASE: u64 = 0xE0000000;
const MAX_BUS: u8 = 255;
const MAX_DEVICE: u8 = 31;
const MAX_FUNCTION: u8 = 7;

/// PCIe Device Function
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciDeviceFunction {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

/// PCIe Vendor ID / Device ID
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PciDeviceId {
    pub vendor_id: u16,
    pub device_id: u16,
}

/// PCIe Class Code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciClassCode {
    pub base: u8,
    pub sub: u8,
    pub interface: u8,
}

/// PCIe Header Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PciHeaderType {
    General,
    PciToPciBridge,
    CardbusBridge,
}

/// PCIe BAR (Base Address Register)
#[derive(Debug, Clone, Copy)]
pub struct PciBar {
    pub index: u8,
    pub address: u64,
    pub size: u64,
    pub is_io: bool,
    pub is_64bit: bool,
}

/// PCIe Device Information
#[derive(Debug, Clone, Copy)]
pub struct PciDeviceInfo {
    pub func: PciDeviceFunction,
    pub device_id: PciDeviceId,
    pub class_code: PciClassCode,
    pub header_type: PciHeaderType,
    pub bars: [Option<PciBar>; 6],
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
}

/// PCIe Bus Manager
pub struct PciBusManager {
    devices: ArrayVec<PciDeviceInfo, 32>, // Up to 32 devices for now
}

impl PciBusManager {
    /// Create a new PCIe bus manager
    pub const fn new() -> Self {
        Self {
            devices: ArrayVec::new_const(),
        }
    }

    /// Initialize PCIe bus enumeration
    pub fn init(&mut self) -> Result<(), super::HalError> {
        // Enumerate all buses
        for bus in 0..=MAX_BUS {
            if let Err(e) = self.enumerate_bus(bus) {
                // Continue enumeration even if one bus fails
                let _ = e;
            }
        }
        Ok(())
    }

    /// Enumerate a PCI bus
    fn enumerate_bus(&mut self, bus: u8) -> Result<(), super::HalError> {
        for device in 0..=MAX_DEVICE {
            self.enumerate_device(bus, device)?;
        }
        Ok(())
    }

    /// Enumerate a PCI device
    fn enumerate_device(&mut self, bus: u8, device: u8) -> Result<(), super::HalError> {
        // Check function 0 first
        let func0 = PciDeviceFunction { bus, device, function: 0 };
        if let Some(info) = self.read_device_info(func0)? {
            self.devices.try_push(info).ok();

            // Check if multi-function device (header type bit 7)
            let header_type = self.read_config_byte(func0, 0x0E);
            if (header_type & 0x80) != 0 {
                // Enumerate other functions
                for function in 1..=MAX_FUNCTION {
                    let func = PciDeviceFunction { bus, device, function };
                    if let Some(info) = self.read_device_info(func)? {
                        self.devices.try_push(info).ok();
                    }
                }
            }
        }
        Ok(())
    }

    /// Read device info from PCI config space
    fn read_device_info(&self, func: PciDeviceFunction) -> Result<Option<PciDeviceInfo>, super::HalError> {
        // Check if vendor ID is 0xFFFF (invalid device)
        let vendor_id = self.read_config_word(func, 0x00);
        if vendor_id == 0xFFFF {
            return Ok(None);
        }

        let device_id = self.read_config_word(func, 0x02);
        let class_base = self.read_config_byte(func, 0x0B);
        let class_sub = self.read_config_byte(func, 0x0A);
        let class_interface = self.read_config_byte(func, 0x09);
        let header_type_byte = self.read_config_byte(func, 0x0E) & 0x7F;
        let interrupt_line = self.read_config_byte(func, 0x3C);
        let interrupt_pin = self.read_config_byte(func, 0x3D);

        let header_type = match header_type_byte {
            0x00 => PciHeaderType::General,
            0x01 => PciHeaderType::PciToPciBridge,
            0x02 => PciHeaderType::CardbusBridge,
            _ => PciHeaderType::General,
        };

        let mut bars = [None; 6];
        let max_bar = if matches!(header_type, PciHeaderType::General) { 6 } else { 2 };
        for i in 0..max_bar {
            bars[i as usize] = self.read_bar(func, i)?;
        }

        Ok(Some(PciDeviceInfo {
            func,
            device_id: PciDeviceId { vendor_id, device_id },
            class_code: PciClassCode {
                base: class_base,
                sub: class_sub,
                interface: class_interface,
            },
            header_type,
            bars,
            interrupt_line,
            interrupt_pin,
        }))
    }

    /// Read a BAR
    fn read_bar(&self, func: PciDeviceFunction, bar_index: u8) -> Result<Option<PciBar>, super::HalError> {
        let offset = 0x10 + (bar_index as u16) * 4;
        let bar_low = self.read_config_dword(func, offset);
        if bar_low == 0 {
            return Ok(None);
        }

        let is_io = (bar_low & 0x1) != 0;
        let mut bar = PciBar {
            index: bar_index,
            address: 0,
            size: 0,
            is_io,
            is_64bit: false,
        };

        if is_io {
            bar.address = (bar_low & !0x3) as u64;
            // Calculate size (simplified)
            unsafe {
                write_volatile(self.ecam_address(func, offset) as *mut u32, 0xFFFFFFFF);
                let size_mask = read_volatile(self.ecam_address(func, offset) as *mut u32);
                write_volatile(self.ecam_address(func, offset) as *mut u32, bar_low);
                bar.size = (!(size_mask & !0x3) + 1) as u64;
            }
        } else {
            let is_mem64 = ((bar_low >> 1) & 0x3) == 0x2;
            bar.is_64bit = is_mem64;

            if is_mem64 && bar_index < 5 {
                let bar_high = self.read_config_dword(func, offset + 4);
                bar.address = ((bar_high as u64) << 32) | ((bar_low & !0xF) as u64);
                // Calculate size
                unsafe {
                    write_volatile(self.ecam_address(func, offset) as *mut u32, 0xFFFFFFFF);
                    write_volatile(self.ecam_address(func, offset + 4) as *mut u32, 0xFFFFFFFF);
                    let size_mask_low = read_volatile(self.ecam_address(func, offset) as *mut u32);
                    let size_mask_high = read_volatile(self.ecam_address(func, offset + 4) as *mut u32);
                    write_volatile(self.ecam_address(func, offset) as *mut u32, bar_low);
                    write_volatile(self.ecam_address(func, offset + 4) as *mut u32, bar_high);
                    let size_mask = ((size_mask_high as u64) << 32) | ((size_mask_low & !0xF) as u64);
                    bar.size = (!size_mask + 1) as u64;
                }
            } else {
                bar.address = (bar_low & !0xF) as u64;
                // Calculate size
                unsafe {
                    write_volatile(self.ecam_address(func, offset) as *mut u32, 0xFFFFFFFF);
                    let size_mask = read_volatile(self.ecam_address(func, offset) as *mut u32);
                    write_volatile(self.ecam_address(func, offset) as *mut u32, bar_low);
                    bar.size = (!(size_mask & !0xF) + 1) as u64;
                }
            }
        }

        Ok(Some(bar))
    }

    /// Read a byte from PCI config space
    fn read_config_byte(&self, func: PciDeviceFunction, offset: u16) -> u8 {
        let addr = self.ecam_address(func, offset) as *const u8;
        unsafe { read_volatile(addr) }
    }

    /// Read a word from PCI config space
    fn read_config_word(&self, func: PciDeviceFunction, offset: u16) -> u16 {
        let addr = self.ecam_address(func, offset) as *const u16;
        unsafe { read_volatile(addr) }
    }

    /// Read a dword from PCI config space
    fn read_config_dword(&self, func: PciDeviceFunction, offset: u16) -> u32 {
        let addr = self.ecam_address(func, offset) as *const u32;
        unsafe { read_volatile(addr) }
    }

    /// Calculate ECAM address for a device function and offset
    fn ecam_address(&self, func: PciDeviceFunction, offset: u16) -> u64 {
        let bus = func.bus as u64;
        let device = func.device as u64;
        let function = func.function as u64;
        let offset = offset as u64;

        ECAM_BASE + (bus << 20) + (device << 15) + (function << 12) + offset
    }

    /// Get list of enumerated PCI devices
    pub fn devices(&self) -> &[PciDeviceInfo] {
        &self.devices
    }
}

/// Static PCIe Bus Manager instance
pub static mut PCI_BUS_MANAGER: PciBusManager = PciBusManager::new();

/// Initialize PCIe bus manager
pub fn init() -> Result<(), super::HalError> {
    unsafe {
        PCI_BUS_MANAGER.init()
    }
}

/// Get list of enumerated PCI devices
pub fn get_devices() -> &'static [PciDeviceInfo] {
    unsafe {
        PCI_BUS_MANAGER.devices()
    }
}
