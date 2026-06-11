
// XPARQ OS - FAT32 Support
// Basic FAT32 filesystem support

use super::mbr::MbrPartitionEntry;

/// FAT32 Boot Sector (BPB)
#[repr(C, packed)]
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
        while end &gt; 0 &amp;&amp; self.filename[end - 1] == b' ' {
            end -= 1;
        }
        core::str::from_utf8(&amp;self.filename[start..end]).unwrap_or("")
    }

    pub fn is_free(&self) -&gt; bool {
        self.filename[0] == 0xE5 || self.filename[0] == 0x00
    }
}

/// FAT32 Filesystem
pub struct Fat32Fs {
    bpb: Fat32Bpb,
    partition: MbrPartitionEntry,
    fat_start: u32,
    data_start: u32,
}

impl Fat32Fs {
    pub fn new(bpb: Fat32Bpb, partition: MbrPartitionEntry) -&gt; Self {
        let fat_start = partition.start_lba + bpb.reserved_sectors as u32;
        let data_start = fat_start + (bpb.num_fats as u32) * bpb.sectors_per_fat_32;

        Self {
            bpb,
            partition,
            fat_start,
            data_start,
        }
    }

    pub fn cluster_to_lba(&amp;self, cluster: u32) -&gt; u32 {
        let data_clusters = cluster - 2;
        self.data_start + (data_clusters * (self.bpb.sectors_per_cluster as u32))
    }

    pub fn first_sector_of_cluster(&amp;self, cluster: u32) -&gt; u32 {
        if cluster &lt; 2 {
            return 0;
        }
        self.cluster_to_lba(cluster)
    }
}
