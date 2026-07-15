
// XPARQ OS - FAT32 Support
// Basic FAT32 filesystem support

use super::mbr::MbrPartitionEntry;

/// FAT32 Boot Sector (BPB)
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct Fat32Bpb {
    pub jmp_boot: [u8; 3],
    pub oem_name: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub root_entries: u16,
    pub total_sectors_16: u16,
    pub media: u8,
    pub sectors_per_fat_16: u16,
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors_32: u32,
    pub sectors_per_fat_32: u32,
    pub flags: u16,
    pub fs_version: u16,
    pub root_cluster: u32,
    pub fs_info_sector: u16,
    pub backup_boot_sector: u16,
    pub reserved: [u8; 12],
    pub drive_number: u8,
    pub reserved2: u8,
    pub boot_signature: u8,
    pub volume_id: u32,
    pub volume_label: [u8; 11],
    pub fs_type: [u8; 8],
    pub boot_code: [u8; 420],
    pub boot_signature2: u16,
}

/// FAT32 Directory Entry
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Fat32DirEntry {
    pub filename: [u8; 8],
    pub ext: [u8; 3],
    pub attr: u8,
    pub nt_reserved: u8,
    pub creation_time_tenth: u8,
    pub creation_time: u16,
    pub creation_date: u16,
    pub last_access_date: u16,
    pub high_cluster: u16,
    pub mod_time: u16,
    pub mod_date: u16,
    pub low_cluster: u16,
    pub file_size: u32,
}

impl Fat32DirEntry {
    pub fn filename_str(&self) -> &str {
        let start = 0;
        let mut end = 8;
        while end > 0 && self.filename[end - 1] == b' ' {
            end -= 1;
        }
        core::str::from_utf8(&self.filename[start..end]).unwrap_or("")
    }

    pub fn is_free(&self) -> bool {
        self.filename[0] == 0xE5 || self.filename[0] == 0x00
    }
}

/// FAT32 Filesystem
pub struct Fat32Fs {
    pub bpb: Fat32Bpb,
    partition: MbrPartitionEntry,
    fat_start: u32,
    data_start: u32,
}

impl Fat32Fs {
    pub fn new(bpb: Fat32Bpb, partition: MbrPartitionEntry) -> Self {
        let fat_start = partition.start_lba + bpb.reserved_sectors as u32;
        let data_start = fat_start + (bpb.num_fats as u32) * bpb.sectors_per_fat_32;

        Self {
            bpb,
            partition,
            fat_start,
            data_start,
        }
    }

    pub fn cluster_to_lba(&self, cluster: u32) -> u32 {
        let data_clusters = cluster - 2;
        self.data_start + (data_clusters * (self.bpb.sectors_per_cluster as u32))
    }

    pub fn first_sector_of_cluster(&self, cluster: u32) -> u32 {
        if cluster < 2 {
            return 0;
        }
        self.cluster_to_lba(cluster)
    }

    pub fn root_cluster(&self) -> u32 {
        self.bpb.root_cluster
    }

    pub fn next_cluster(&self, driver: &mut dyn crate::storage::StorageDriver, device_id: u32, cluster: u32) -> Result<u32, crate::storage::StorageError> {
        let fat_offset = cluster * 4;
        let fat_sector = self.fat_start + (fat_offset / 512);
        let ent_offset = (fat_offset % 512) as usize;
        let mut buf = [0u8; 512];
        driver.read(device_id, fat_sector as u64, &mut buf)?;
        let mut next_cluster = u32::from_le_bytes(buf[ent_offset..ent_offset+4].try_into().unwrap());
        next_cluster &= 0x0FFFFFFF;
        Ok(next_cluster)
    }

    pub fn read_cluster(&self, driver: &mut dyn crate::storage::StorageDriver, device_id: u32, cluster: u32, buffer: &mut [u8]) -> Result<(), crate::storage::StorageError> {
        let lba = self.first_sector_of_cluster(cluster) as u64;
        driver.read(device_id, lba, buffer)
    }

    pub fn list_directory(&self, driver: &mut dyn crate::storage::StorageDriver, device_id: u32, dir_cluster: u32) -> Result<arrayvec::ArrayVec<Fat32File, 32>, crate::storage::StorageError> {
        let mut files = arrayvec::ArrayVec::new();
        let mut cluster = dir_cluster;
        let cluster_size = (self.bpb.sectors_per_cluster as usize) * 512;
        let mut buffer = [0u8; 512]; 
        
        while cluster < 0x0FFFFFF8 && cluster >= 2 {
            let lba = self.first_sector_of_cluster(cluster) as u64;
            
            for sector_idx in 0..(self.bpb.sectors_per_cluster as u32) {
                driver.read(device_id, lba + (sector_idx as u64), &mut buffer)?;
                
                crate::println!("    -> [fat32] buffer[0..8]: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}", buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7]);

                for i in 0..16 {
                    let offset = i * 32;
                    let entry_ptr = buffer[offset..].as_ptr() as *const Fat32DirEntry;
                let entry = unsafe { *entry_ptr };
                if entry.filename[0] == 0x00 { // End of directory
                    return Ok(files);
                }
                if entry.filename[0] == 0xE5 { // Deleted
                    continue;
                }
                if entry.attr & 0x0F == 0x0F { // LFN
                    continue; 
                }
                let mut name = [0u8; 11];
                name[..8].copy_from_slice(&entry.filename);
                name[8..].copy_from_slice(&entry.ext);
                    let entry = unsafe { *entry_ptr };
                    if entry.filename[0] == 0x00 { // End of directory
                        return Ok(files);
                    }
                    if entry.filename[0] == 0xE5 { // Deleted
                        continue;
                    }
                    if entry.attr & 0x0F == 0x0F { // LFN
                        continue; 
                    }
                    let mut name = [0u8; 11];
                    name[..8].copy_from_slice(&entry.filename);
                    name[8..].copy_from_slice(&entry.ext);
                    let start_cluster = ((entry.high_cluster as u32) << 16) | (entry.low_cluster as u32);
                    let _ = files.try_push(Fat32File {
                        name,
                        attr: entry.attr,
                        start_cluster,
                        size: entry.file_size,
                    });
                }
            }
            cluster = self.next_cluster(driver, device_id, cluster)?;
        }
        Ok(files)
    }

    pub fn read_file(&self, driver: &mut dyn crate::storage::StorageDriver, device_id: u32, file: &Fat32File, buf: &mut [u8]) -> Result<usize, crate::storage::StorageError> {
        let mut cluster = file.start_cluster;
        let mut bytes_read = 0;
        let mut temp_buf = [0u8; 512];
        
        while cluster >= 2 && cluster < 0x0FFFFFF8 && bytes_read < buf.len() && bytes_read < file.size as usize {
            let lba = self.first_sector_of_cluster(cluster) as u64;
            
            for sector_idx in 0..(self.bpb.sectors_per_cluster as u32) {
                if bytes_read >= buf.len() || bytes_read >= file.size as usize {
                    break;
                }
                
                driver.read(device_id, lba + (sector_idx as u64), &mut temp_buf)?;
                let remaining = core::cmp::min(file.size as usize - bytes_read, buf.len() - bytes_read);
                let to_copy = core::cmp::min(remaining, 512);
                buf[bytes_read..bytes_read + to_copy].copy_from_slice(&temp_buf[..to_copy]);
                bytes_read += to_copy;
            }
            cluster = self.next_cluster(driver, device_id, cluster)?;
        }
        Ok(bytes_read)
    }
}

pub struct Fat32File {
    pub name: [u8; 11],
    pub attr: u8,
    pub start_cluster: u32,
    pub size: u32,
}
