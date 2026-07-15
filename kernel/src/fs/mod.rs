// XPARQ OS - Phase 5: Virtual File System Manager
// Manages the global file system namespace

pub mod vfs;
pub mod fat32_vfs;
pub mod fd;

use crate::sync::IrqSafeMutex;

pub enum FileSystemVariant {
    None,
    Fat32(fat32_vfs::Fat32Vfs),
}

pub struct VfsManager {
    pub root_fs: FileSystemVariant,
}

pub static VFS_MANAGER: IrqSafeMutex<VfsManager> = IrqSafeMutex::new(VfsManager::new());

impl VfsManager {
    pub const fn new() -> Self {
        Self {
            root_fs: FileSystemVariant::None,
        }
    }

    pub fn mount_root(&mut self, fs: FileSystemVariant) {
        self.root_fs = fs;
    }

    pub fn open(&self, path: &str) -> Option<vfs::VNodeInfo> {
        let path = path.trim_start_matches('/');

        let fs = match &self.root_fs {
            FileSystemVariant::None => return None,
            FileSystemVariant::Fat32(f) => f,
        };

        use vfs::FileSystem;
        let root = fs.root();
        
        if path.is_empty() {
            return Some(root);
        }

        // Extremely simplistic path resolution (only supports 1 level right now for Phase 9)
        fs.find(root.internal_id, path)
    }

    pub fn read_file(&self, node: &vfs::VNodeInfo, buf: &mut [u8]) -> Result<usize, ()> {

        let fs = match &self.root_fs {
            FileSystemVariant::None => return Err(()),
            FileSystemVariant::Fat32(f) => f,
        };

        use vfs::FileSystem;
        fs.read_file(node.internal_id, 0, buf)
    }
}
