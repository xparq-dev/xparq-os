// XPARQ OS - Phase 4: Device Drivers Integration
// Storage Manager
// Bridges HAL StorageDriver to Kernel APIs

use xparq_hal as hal;
use hal::storage::{StorageDriver, StorageDevice};
use spin::Mutex;
use arrayvec::ArrayVec;

pub const MAX_MOUNTED_VOLUMES: usize = 4;

pub struct StorageManager {
    pub volumes: ArrayVec<Volume, MAX_MOUNTED_VOLUMES>,
}

pub struct Volume {
    pub device_id: u32,
    pub start_lba: u64,
    pub sector_count: u64,
    pub fs_type: u8,
}

pub static STORAGE_MANAGER: Mutex<StorageManager> = Mutex::new(StorageManager::new());

impl StorageManager {
    pub const fn new() -> Self {
        Self {
            volumes: ArrayVec::new_const(),
        }
    }

    pub fn init(&mut self) {
        // Try to scan for partitions on device 2 (ATA Primary Slave)
        let device_id = 2;
        self.scan_partitions(device_id);
    }

    fn scan_partitions(&mut self, device_id: u32) {
        // For Phase 10, we know device 2 is a raw FAT32 image
        let _ = self.volumes.try_push(Volume {
            device_id,
            start_lba: 0,
            sector_count: 34 * 1024 * 1024 / 512,
            fs_type: 0x0B, // FAT32
        });
    }

    pub fn read_volume(&self, vol_idx: usize, lba_offset: u64, buffer: &mut [u8]) -> Result<(), ()> {
        if vol_idx >= self.volumes.len() {
            return Err(());
        }
        let vol = &self.volumes[vol_idx];
        if lba_offset >= vol.sector_count {
            return Err(());
        }
        
        let absolute_lba = vol.start_lba + lba_offset;
        
        if let Some(storage) = hal::x86_64::STORAGE.lock().as_mut() {
            if storage.read(vol.device_id, absolute_lba, buffer).is_ok() {
                return Ok(());
            }
        }
        Err(())
    }
}
