
// XPARQ OS - MBR Support
// MBR partition table parsing

use core::mem;

/// MBR Partition Entry
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MbrPartitionEntry {
    pub bootable: u8,
    pub start_chs: [u8; 3],
    pub partition_type: u8,
    pub end_chs: [u8; 3],
    pub start_lba: u32,
    pub sector_count: u32,
}

/// MBR Partition Table
#[repr(C, packed)]
pub struct MbrPartitionTable {
    pub bootloader: [u8; 446],
    pub partitions: [MbrPartitionEntry; 4],
    pub signature: u16,
}

impl MbrPartitionTable {
    pub fn from_bytes(bytes: &[u8]) -> Option<&Self> {
        if bytes.len() < mem::size_of::<Self>() {
            return None;
        }
        let ptr = bytes.as_ptr() as *const Self;
        let table = unsafe { &*ptr };
        if table.signature != 0xAA55 {
            return None;
        }
        Some(table)
    }

    pub fn get_partition(&self, index: usize) -> Option<&MbrPartitionEntry> {
        if index >= 4 {
            return None;
        }
        let part = &self.partitions[index];
        if part.partition_type == 0 {
            return None;
        }
        Some(part)
    }
}

/// Supported MBR Partition Types
#[repr(u8)]
pub enum MbrPartitionType {
    Fat12 = 0x01,
    Fat16Lt32 = 0x04,
    Extended = 0x05,
    Fat16Ge32 = 0x06,
    Ntfs = 0x07,
    Fat32 = 0x0B,
    Fat32Lba = 0x0C,
}
