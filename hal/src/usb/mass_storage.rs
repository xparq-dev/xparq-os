// XPARQ OS - USB Mass Storage Class (Bulk-Only Transport) Driver
// Skeleton for USB flash drives, etc.

use super::*;

/// USB Mass Storage Device
pub struct UsbMassStorageDevice {
    pub info: UsbDeviceInfo,
    pub block_size: u32,
    pub num_blocks: u64,
}

impl UsbMassStorageDevice {
    pub const fn new() -> Self {
        Self {
            info: UsbDeviceInfo {
                device_class: UsbDeviceClass::MassStorage,
                vendor_id: 0,
                product_id: 0,
                bus_number: 0,
                device_address: 0,
                speed: UsbSpeed::High,
            },
            block_size: 512,
            num_blocks: 0,
        }
    }

    pub fn read_blocks(&self, lba: u64, count: u32, buf: &mut [u8]) -> Result<(), UsbError> {
        // TODO: Implement SCSI READ(10) via Bulk-Only Transport
        Err(UsbError::Unsupported)
    }

    pub fn write_blocks(&mut self, lba: u64, count: u32, buf: &[u8]) -> Result<(), UsbError> {
        // TODO: Implement SCSI WRITE(10) via Bulk-Only Transport
        Err(UsbError::Unsupported)
    }
}
