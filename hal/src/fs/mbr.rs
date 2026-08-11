
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_fat32_partition() {
        let mut sector = [0u8; 512];
        sector[450] = MbrPartitionType::Fat32Lba as u8;
        sector[454..458].copy_from_slice(&2048u32.to_le_bytes());
        sector[458..462].copy_from_slice(&69632u32.to_le_bytes());
        sector[510] = 0x55;
        sector[511] = 0xAA;
        let table = MbrPartitionTable::from_bytes(&sector).expect("valid MBR");
        let partition = table.get_partition(0).expect("partition 0");
        let start_lba = partition.start_lba;
        let sector_count = partition.sector_count;
        assert_eq!(partition.partition_type, 0x0C);
        assert_eq!(start_lba, 2048);
        assert_eq!(sector_count, 69632);
    }

    #[test]
    fn rejects_bad_signature_and_empty_partition() {
        let mut sector = [0u8; 512];
        assert!(MbrPartitionTable::from_bytes(&sector).is_none());
        sector[510] = 0x55;
        sector[511] = 0xAA;
        let table = MbrPartitionTable::from_bytes(&sector).expect("valid empty MBR");
        assert!(table.get_partition(0).is_none());
    }
}
