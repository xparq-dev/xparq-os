// XPARQ OS - Phase 5: Virtual File System Core
// Provides abstraction over multiple file systems (FAT32, Ext2, etc.)
// Adapted for no_alloc environment

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VNodeType {
    File,
    Directory,
}

#[derive(Debug, Clone, Copy)]
pub struct VNodeInfo {
    pub name: [u8; 11],
    pub size: u32,
    pub node_type: VNodeType,
    pub internal_id: u32, // e.g., cluster number
}

pub trait FileSystem {
    fn root(&self) -> VNodeInfo;
    fn get_children(&self, dir_id: u32, children_out: &mut [VNodeInfo]) -> Result<usize, ()>;
    fn find(&self, dir_id: u32, name: &str) -> Option<VNodeInfo>;
    fn read_file(&self, file_id: u32, offset: u32, buf: &mut [u8]) -> Result<usize, ()>;
}
