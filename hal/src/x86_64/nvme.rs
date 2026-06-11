
// XPARQ OS - NVMe Driver Skeleton
// Simple NVMe driver for Phase 2

use crate::x86_64::pci::{PciDeviceInfo, PciDriver};
use core::ptr::{read_volatile, write_volatile};

/// NVMe Driver
pub struct NvmeDriver {
    bar0: u64,
    initialized: bool,
}

impl NvmeDriver {
    pub const fn new() -> Self {
        Self { bar0: 0, initialized: false }
    }

    pub fn init(&mut self, dev: &PciDeviceInfo) -> Result<(), ()> {
        for bar in &dev.bars {
            if let Some(bar) = bar {
                if bar.index == 0 {
                    self.bar0 = bar.address;
                }
            }
        }
        if self.bar0 == 0 {
            return Err(());
        }

        // TODO: Real initialization
        self.initialized = true;
        Ok(())
    }
}

impl Default for NvmeDriver {
    fn default() -> Self {
        Self::new()
    }
}

// Static NVMe driver
use spin::Mutex;
pub static NVME_DRIVER: Mutex<NvmeDriver> = Mutex::new(NvmeDriver::new());

// PciDriver implementation for NVMe
struct NvmePciDriver;
pub static NVME_PCI_DRIVER: NvmePciDriver = NvmePciDriver;

impl PciDriver for NvmePciDriver {
    fn probe(&self, dev: &PciDeviceInfo) -> Result<(), ()> {
        // Check if NVMe controller (Class 0x01, Subclass 0x08, Interface 0x02)
        if dev.class_code.base == 0x01 && dev.class_code.sub == 0x08 {
            let mut driver = NVME_DRIVER.lock();
            driver.init(dev)?;
            Ok(())
        } else {
            Err(())
        }
    }

    fn get_supported_devices(&self) -> &'static [(u16, u16)] {
        // Wildcard for any NVMe controller
        &[(0x0000, 0x0000)]
    }
}
