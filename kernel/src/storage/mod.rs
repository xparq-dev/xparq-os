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
        // The canonical x86-64 image is attached as the primary ATA master.
        self.volumes.clear();
        self.scan_partitions(1);
    }

    fn scan_partitions(&mut self, device_id: u32) {
        let mut sector = [0u8; 512];
        let mut storage_guard = hal::x86_64::STORAGE.lock();
        let Some(storage) = storage_guard.as_mut() else { return; };
        if storage.read(device_id, 0, &mut sector).is_err() { return; }
        if sector[510] != 0x55 || sector[511] != 0xAA { return; }

        for index in 0..4 {
            let offset = 446 + index * 16;
            let fs_type = sector[offset + 4];
            if fs_type != 0x0B && fs_type != 0x0C { continue; }
            let start_lba = u32::from_le_bytes([
                sector[offset + 8], sector[offset + 9], sector[offset + 10], sector[offset + 11],
            ]) as u64;
            let sector_count = u32::from_le_bytes([
                sector[offset + 12], sector[offset + 13], sector[offset + 14], sector[offset + 15],
            ]) as u64;
            if start_lba == 0 || sector_count == 0 { continue; }
            let _ = self.volumes.try_push(Volume { device_id, start_lba, sector_count, fs_type });
        }
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
