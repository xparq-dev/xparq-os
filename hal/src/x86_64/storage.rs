// XPARQ OS - x86_64 Storage Driver
// Storage driver with RAM disk, ATA/IDE, and NVMe support

use crate::storage::{StorageDriver, StorageError, StorageDevice, StorageType, StorageInterface,
                   StorageInfo, StorageHealth, StorageCapabilities, StorageStatus, DeviceStatus,
                   StorageStatistics, PowerMode};
use arrayvec::ArrayVec;
use core::ptr::write_volatile;
use core::sync::atomic::{AtomicU64, Ordering};

// RAM Disk definitions
const RAM_DISK_SIZE: usize = 64 * 1024 * 1024;
const RAM_DISK_SECTOR_SIZE: usize = 512;
const RAM_DISK_SECTOR_COUNT: usize = RAM_DISK_SIZE / RAM_DISK_SECTOR_SIZE;

// Global RAM disk buffer (aligned to 512 bytes)
#[repr(align(512))]
struct RamDiskBuffer([u8; RAM_DISK_SIZE]);
static RAM_DISK: RamDiskBuffer = RamDiskBuffer([0; RAM_DISK_SIZE]);
static READS: AtomicU64 = AtomicU64::new(0);
static WRITES: AtomicU64 = AtomicU64::new(0);

// ATA/IDE definitions
const ATA_PRIMARY_BASE: u16 = 0x1F0;
const ATA_PRIMARY_CTRL: u16 = 0x3F6;

// I/O port functions
#[cfg(target_arch = "x86_64")]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

#[cfg(target_arch = "x86_64")]
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    core::arch::asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

#[cfg(target_arch = "x86_64")]
unsafe fn outw(port: u16, value: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
}

/// x86_64 storage driver
pub struct X86StorageDriver {
    initialized: bool,
}

impl X86StorageDriver {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    // ATA/IDE helper functions
    unsafe fn ata_wait_bsy(&self, base: u16) {
        loop {
            let status = inb(base + 7);
            if (status & 0x80) == 0 {
                break;
            }
        }
    }

    unsafe fn ata_wait_drq(&self, base: u16) {
        loop {
            let status = inb(base + 7);
            if (status & 0x08) != 0 {
                break;
            }
        }
    }

    unsafe fn ata_read_sectors(&mut self, base: u16, lba: u32, count: u8, buffer: &mut [u8]) -> Result<(), StorageError> {
        self.ata_wait_bsy(base);
        outb(base + 6, 0xE0 | ((lba >> 24) & 0x0F)); // Drive 0, LBA28
        outb(base + 2, count); // Sector count
        outb(base + 3, lba as u8); // LBA 0-7
        outb(base + 4, (lba >> 8) as u8); // LBA 8-15
        outb(base + 5, (lba >> 16) as u8); // LBA 16-23
        outb(base + 7, 0x20); // Read sectors command
        self.ata_wait_bsy(base);
        self.ata_wait_drq(base);

        for i in 0..count as usize {
            for j in 0..256 { // 512 bytes per sector, read as 16-bit words
                let word = inw(base + 0);
                buffer[i * 512 + j * 2] = (word & 0xFF) as u8;
                buffer[i * 512 + j * 2 + 1] = (word >> 8) as u8;
            }
        }
        Ok(())
    }

    unsafe fn ata_write_sectors(&mut self, base: u16, lba: u32, count: u8, data: &[u8]) -> Result<(), StorageError> {
        self.ata_wait_bsy(base);
        outb(base + 6, 0xE0 | ((lba >> 24) & 0x0F)); // Drive 0, LBA28
        outb(base + 2, count); // Sector count
        outb(base + 3, lba as u8); // LBA 0-7
        outb(base + 4, (lba >> 8) as u8); // LBA 8-15
        outb(base + 5, (lba >> 16) as u8); // LBA 16-23
        outb(base + 7, 0x30); // Write sectors command
        self.ata_wait_bsy(base);
        self.ata_wait_drq(base);

        for i in 0..count as usize {
            for j in 0..256 { // 512 bytes per sector, write as 16-bit words
                let word = (data[i * 512 + j * 2] as u16) | ((data[i * 512 + j * 2 + 1] as u16) << 8);
                outw(base + 0, word);
            }
        }
        Ok(())
    }
}

impl Default for X86StorageDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageDriver for X86StorageDriver {
    fn name(&self) -> &'static str {
        "x86_64 Storage Driver (RAM Disk + ATA/IDE + NVMe)"
    }

    fn init(&mut self) -> Result<(), StorageError> {
        self.initialized = true;
        Ok(())
    }

    fn get_devices(&self) -> ArrayVec<StorageDevice, 8> {
        let mut devices = ArrayVec::new();
        // RAM Disk
        devices.push(StorageDevice {
            id: 0,
            name: "XPARQ RAM Disk",
            device_type: StorageType::SolidState,
            interface: StorageInterface::Virtual,
            info: StorageInfo {
                model: "XPARQ RAM Disk 64MB",
                serial: "XPARQ-RAM-0001",
                firmware: "v1.0.0",
                capacity: RAM_DISK_SIZE as u64,
                block_size: RAM_DISK_SECTOR_SIZE as u32,
                sector_size: RAM_DISK_SECTOR_SIZE as u32,
                total_blocks: RAM_DISK_SECTOR_COUNT as u64,
                usable_blocks: RAM_DISK_SECTOR_COUNT as u64,
                temperature: None,
                health: StorageHealth::Good,
            },
            capabilities: StorageCapabilities {
                read_cache: true,
                write_cache: true,
                command_queueing: false,
                trim_support: false,
                encryption: false,
                power_management: false,
                smart_support: false,
                wear_leveling: false,
            },
        });
        // ATA Primary Master (QEMU disk.img)
        devices.push(StorageDevice {
            id: 1,
            name: "QEMU ATA Primary Master",
            device_type: StorageType::HardDisk,
            interface: StorageInterface::ATA,
            info: StorageInfo {
                model: "QEMU ATA HDD",
                serial: "QEMU-ATA-12345",
                firmware: "v1.0.0",
                capacity: 32 * 1024, // 32KB (matches disk.img size)
                block_size: 512,
                sector_size: 512,
                total_blocks: 64, // 32KB / 512 bytes
                usable_blocks: 64,
                temperature: None,
                health: StorageHealth::Good,
            },
            capabilities: StorageCapabilities {
                read_cache: true,
                write_cache: true,
                command_queueing: false,
                trim_support: false,
                encryption: false,
                power_management: false,
                smart_support: false,
                wear_leveling: false,
            },
        });
        // NVMe Device (skeleton)
        devices.push(StorageDevice {
            id: 2,
            name: "XPARQ NVMe SSD",
            device_type: StorageType::SolidState,
            interface: StorageInterface::NVMe,
            info: StorageInfo {
                model: "XPARQ NVMe SSD 256GB",
                serial: "XPARQ-NVME-0001",
                firmware: "v1.0.0",
                capacity: 256 * 1024 * 1024 * 1024, // 256 GB
                block_size: 512,
                sector_size: 512,
                total_blocks: (256 * 1024 * 1024 * 1024) / 512,
                usable_blocks: (256 * 1024 * 1024 * 1024) / 512,
                temperature: None,
                health: StorageHealth::Good,
            },
            capabilities: StorageCapabilities {
                read_cache: true,
                write_cache: true,
                command_queueing: true,
                trim_support: true,
                encryption: false,
                power_management: false,
                smart_support: false,
                wear_leveling: false,
            },
        });
        devices
    }

    fn read(&mut self, device_id: u32, lba: u64, buffer: &mut [u8]) -> Result<(), StorageError> {
        match device_id {
            0 => {
                // RAM disk
                let start_sector = lba as usize;
                let count = buffer.len() / RAM_DISK_SECTOR_SIZE;
                if start_sector + count > RAM_DISK_SECTOR_COUNT {
                    return Err(StorageError::InvalidParameter);
                }
                let start_addr = start_sector * RAM_DISK_SECTOR_SIZE;
                for (i, byte) in buffer.iter_mut().enumerate() {
                    unsafe {
                        *byte = core::ptr::read_volatile(&RAM_DISK.0[start_addr + i]);
                    }
                }
                READS.fetch_add(count as u64, Ordering::Relaxed);
                Ok(())
            }
            1 => {
                // ATA/IDE drive
                let count = buffer.len() / 512;
                if count > 255 {
                    return Err(StorageError::InvalidParameter);
                }
                unsafe {
                    self.ata_read_sectors(ATA_PRIMARY_BASE, lba as u32, count as u8, buffer)
                }
            }
            2 => {
                // NVMe drive (skeleton)
                // TODO: Implement actual NVMe read
                Ok(())
            }
            _ => Err(StorageError::DeviceNotFound),
        }
    }

    fn write(&mut self, device_id: u32, lba: u64, data: &[u8]) -> Result<(), StorageError> {
        match device_id {
            0 => {
                // RAM disk
                let start_sector = lba as usize;
                let count = data.len() / RAM_DISK_SECTOR_SIZE;
                if start_sector + count > RAM_DISK_SECTOR_COUNT {
                    return Err(StorageError::InvalidParameter);
                }
                let start_addr = start_sector * RAM_DISK_SECTOR_SIZE;
                for (i, &byte) in data.iter().enumerate() {
                    unsafe {
                        write_volatile(&mut RAM_DISK.0[start_addr + i] as *mut u8, byte);
                    }
                }
                WRITES.fetch_add(count as u64, Ordering::Relaxed);
                Ok(())
            }
            1 => {
                // ATA/IDE drive
                let count = data.len() / 512;
                if count > 255 {
                    return Err(StorageError::InvalidParameter);
                }
                unsafe {
                    self.ata_write_sectors(ATA_PRIMARY_BASE, lba as u32, count as u8, data)
                }
            }
            2 => {
                // NVMe drive (skeleton)
                // TODO: Implement actual NVMe write
                Ok(())
            }
            _ => Err(StorageError::DeviceNotFound),
        }
    }

    fn flush(&mut self, _device_id: u32) -> Result<(), StorageError> {
        Ok(())
    }

    fn get_device_status(&self, device_id: u32) -> Option<StorageStatus> {
        match device_id {
            0 => Some(StorageStatus {
                device_id: 0,
                status: DeviceStatus::Online,
                temperature: None,
                busy: false,
                error_count: 0,
                last_error: None,
            }),
            1 => Some(StorageStatus {
                device_id: 1,
                status: DeviceStatus::Online,
                temperature: None,
                busy: false,
                error_count: 0,
                last_error: None,
            }),
            2 => Some(StorageStatus {
                device_id: 2,
                status: DeviceStatus::Online,
                temperature: None,
                busy: false,
                error_count: 0,
                last_error: None,
            }),
            _ => None,
        }
    }

    fn get_device_statistics(&self, device_id: u32) -> Option<StorageStatistics> {
        match device_id {
            0 => Some(StorageStatistics {
                device_id: 0,
                reads: READS.load(Ordering::Relaxed),
                writes: WRITES.load(Ordering::Relaxed),
                read_bytes: READS.load(Ordering::Relaxed) * RAM_DISK_SECTOR_SIZE as u64,
                write_bytes: WRITES.load(Ordering::Relaxed) * RAM_DISK_SECTOR_SIZE as u64,
                errors: 0,
                uptime: 0,
                power_on_hours: 0,
                wear_level: None,
                endurance: None,
            }),
            1 => Some(StorageStatistics {
                device_id: 1,
                reads: 0,
                writes: 0,
                read_bytes: 0,
                write_bytes: 0,
                errors: 0,
                uptime: 0,
                power_on_hours: 0,
                wear_level: None,
                endurance: None,
            }),
            2 => Some(StorageStatistics {
                device_id: 2,
                reads: 0,
                writes: 0,
                read_bytes: 0,
                write_bytes: 0,
                errors: 0,
                uptime: 0,
                power_on_hours: 0,
                wear_level: None,
                endurance: None,
            }),
            _ => None,
        }
    }

    fn erase(&mut self, _device_id: u32, _lba: u64, _count: u64) -> Result<(), StorageError> {
        Ok(())
    }

    fn trim(&mut self, _device_id: u32, _lba: u64, _count: u64) -> Result<(), StorageError> {
        Ok(())
    }

    fn set_power_mode(&mut self, _device_id: u32, _mode: PowerMode) -> Result<(), StorageError> {
        Ok(())
    }

    fn get_power_mode(&self, _device_id: u32) -> Option<PowerMode> {
        Some(PowerMode::Active)
    }
}
