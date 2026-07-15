// XPARQ OS - Phase 5: FAT32 VFS Binding
// Wraps the HAL's Fat32Fs in the VFS FileSystem trait

use crate::fs::vfs::{FileSystem, VNodeInfo, VNodeType};
use xparq_hal::fs::fat32::{Fat32Fs, Fat32Bpb};
use xparq_hal::fs::mbr::MbrPartitionEntry;
use xparq_hal::storage::StorageDriver;
use arrayvec::ArrayVec;
use crate::storage::STORAGE_MANAGER;

fn fmt_hex2(val: u8) -> [u8; 2] {
    let hex_chars = b"0123456789abcdef";
    [hex_chars[(val >> 4) as usize], hex_chars[(val & 0x0F) as usize]]
}

fn fmt_hex8(val: u32) -> [u8; 8] {
    let hex_chars = b"0123456789abcdef";
    [
        hex_chars[((val >> 28) & 0x0F) as usize],
        hex_chars[((val >> 24) & 0x0F) as usize],
        hex_chars[((val >> 20) & 0x0F) as usize],
        hex_chars[((val >> 16) & 0x0F) as usize],
        hex_chars[((val >> 12) & 0x0F) as usize],
        hex_chars[((val >> 8) & 0x0F) as usize],
        hex_chars[((val >> 4) & 0x0F) as usize],
        hex_chars[(val & 0x0F) as usize],
    ]
}

pub struct Fat32Vfs {
    pub fs: Fat32Fs,
    pub volume_id: usize, // Index in STORAGE_MANAGER.volumes
    pub device_id: u32,
}

impl FileSystem for Fat32Vfs {
    fn root(&self) -> VNodeInfo {
        let root_cluster = self.fs.root_cluster();
        VNodeInfo {
            name: [b' '; 11],
            size: 0,
            node_type: VNodeType::Directory,
            internal_id: root_cluster,
        }
    }

    fn get_children(&self, dir_id: u32, children_out: &mut [VNodeInfo]) -> Result<usize, ()> {
        unsafe { crate::uart_puts(b"    -> [fat32_vfs] get_children started\n"); }
        let mut storage_lock = STORAGE_MANAGER.lock();
        unsafe { crate::uart_puts(b"    -> [fat32_vfs] got STORAGE_MANAGER lock\n"); }
        let mut out_idx = 0;
        
        // Need to acquire raw storage to use Fat32Fs's list_directory
        // However, list_directory returns an ArrayVec of Fat32File.
        unsafe { crate::uart_puts(b"    -> [fat32_vfs] acquiring hal_storage lock (1)\n"); }
        let storage_opt = {
            let mut guard = xparq_hal::x86_64::STORAGE.lock();
            unsafe { crate::uart_puts(b"    -> [fat32_vfs] got hal_storage lock (1)\n"); }
            let ret = guard.take();
            unsafe { crate::uart_puts(b"    -> [fat32_vfs] about to drop guard (1)\n"); }
            ret
        }; // EXPLICITLY DROP THE GUARD
        unsafe { crate::uart_puts(b"    -> [fat32_vfs] guard explicitly dropped (1)\n"); }
        
        if let Some(mut hal_storage) = storage_opt {
            unsafe { crate::uart_puts(b"    -> [fat32_vfs] inside Some(hal_storage)\n"); }
            
            let mut dump_buf = [0u8; 512];
            let root_lba = self.fs.cluster_to_lba(self.fs.root_cluster());
            
            unsafe { 
                crate::uart_puts(b"    -> [fat32_vfs] calling hal_storage.read on LBA "); 
                let hex = fmt_hex8(root_lba);
                crate::uart_puts(&hex); 
                crate::uart_puts(b"\n"); 
            }
            
            if let Ok(_) = hal_storage.read(self.device_id, root_lba as u64, &mut dump_buf) {
                // Do nothing
            }

            unsafe { 
                crate::uart_puts(b"    -> [fat32_vfs] SPC: "); 
                let hex = fmt_hex2(self.fs.bpb.sectors_per_cluster);
                crate::uart_puts(&hex);
                crate::uart_puts(b"\n");
                
                crate::uart_puts(b"    -> [fat32_vfs] calling list_directory...\n"); 
            }
            if let Ok(files) = self.fs.list_directory(&mut hal_storage, self.device_id, dir_id) {
                unsafe { crate::uart_puts(b"    -> [fat32_vfs] list_directory OK, found "); }
                unsafe { crate::uart_puts(b" files\n"); }
                
                for file in files {
                    if out_idx >= children_out.len() {
                        break;
                    }
                    unsafe { crate::uart_puts(b"    -> [fat32_vfs] processing file: "); }
                    let node_name_str = core::str::from_utf8(&file.name).unwrap_or("").trim();
                    unsafe { crate::uart_puts(node_name_str.as_bytes()); crate::uart_puts(b"\n"); }
                    
                    let mut name = [0u8; 11];
                    name.copy_from_slice(&file.name);
                    
                    let is_dir = (file.attr & 0x10) != 0;
                    children_out[out_idx] = VNodeInfo {
                        name,
                        size: file.size,
                        node_type: if is_dir { VNodeType::Directory } else { VNodeType::File },
                        internal_id: file.start_cluster,
                    };
                    out_idx += 1;
                }
                unsafe { crate::uart_puts(b"    -> [fat32_vfs] loop done\n"); }
            }
            // Restore storage!
            unsafe { crate::uart_puts(b"    -> [fat32_vfs] acquiring hal_storage lock (2)\n"); }
            {
                let mut guard2 = xparq_hal::x86_64::STORAGE.lock();
                unsafe { crate::uart_puts(b"    -> [fat32_vfs] got hal_storage lock (2)\n"); }
                *guard2 = Some(hal_storage);
                unsafe { crate::uart_puts(b"    -> [fat32_vfs] assigned Some(hal_storage)\n"); }
            } // explicitly drop
            unsafe { crate::uart_puts(b"    -> [fat32_vfs] guard explicitly dropped (2)\n"); }
        }
        
        unsafe { crate::uart_puts(b"    -> [fat32_vfs] get_children finished\n"); }
        Ok(out_idx)
    }

    fn find(&self, dir_id: u32, name: &str) -> Option<VNodeInfo> {
        unsafe { crate::uart_puts(b"    -> [fat32_vfs] find started\n"); }
        let mut children = [VNodeInfo {
            name: [0; 11],
            size: 0,
            node_type: VNodeType::File,
            internal_id: 0,
        }; 32];
        
        if let Ok(count) = self.get_children(dir_id, &mut children) {
            unsafe { crate::uart_puts(b"    -> [fat32_vfs] find got children\n"); }
            for i in 0..count {
                // Simplistic name matching for 8.3 FAT32 names
                let node = &children[i];
                let node_name_str = core::str::from_utf8(&node.name).unwrap_or("").trim();
                let search_name = name.trim();
                
                // Real comparison would need to handle "FILE    TXT" vs "FILE.TXT"
                // For now, if it matches exactly after trim (assuming no extensions for dirs or exactly matched for files)
                // We will do a basic string compare.
                
                let mut formatted_name = [0u8; 12];
                let mut f_idx = 0;
                for j in 0..8 {
                    if node.name[j] != b' ' {
                        formatted_name[f_idx] = node.name[j];
                        f_idx += 1;
                    }
                }
                if node.name[8] != b' ' {
                    formatted_name[f_idx] = b'.';
                    f_idx += 1;
                    for j in 8..11 {
                        if node.name[j] != b' ' {
                            formatted_name[f_idx] = node.name[j];
                            f_idx += 1;
                        }
                    }
                }
                
                let formatted_str = core::str::from_utf8(&formatted_name[0..f_idx]).unwrap_or("");
                unsafe { crate::uart_puts(b"      found: "); crate::uart_puts(formatted_str.as_bytes()); crate::uart_puts(b"\n"); }
                
                if formatted_str.eq_ignore_ascii_case(search_name) {
                    return Some(node.clone());
                }
            }
        }
        None
    }

    fn read_file(&self, file_id: u32, offset: u32, buf: &mut [u8]) -> Result<usize, ()> {
        // FAT32 read_file doesn't support offset natively in our HAL yet,
        // it just reads from start. For this phase, we just call read_file.
        // A full implementation would seek to offset.
        
        let storage_opt = {
            let mut guard = xparq_hal::x86_64::STORAGE.lock();
            guard.take()
        };
        
        if let Some(mut hal_storage) = storage_opt {
            use xparq_hal::fs::fat32::Fat32File;
            
            // Create a fake Fat32File since we only need start_cluster and size
            let fake_file = Fat32File {
                name: [0; 11],
                attr: 0,
                start_cluster: file_id,
                size: 0xFFFFFFFF, // Doesn't matter for read_file
            };
            
            let res = self.fs.read_file(&mut hal_storage, self.device_id, &fake_file, buf);
            {
                let mut guard = xparq_hal::x86_64::STORAGE.lock();
                *guard = Some(hal_storage);
            }
            
            if let Ok(bytes_read) = res {
                Ok(bytes_read)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}
