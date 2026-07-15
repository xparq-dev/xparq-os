// XPARQ OS - Phase 18: Unified File Descriptor (FD) Table
// Provides a unified abstraction for files, devices, and network sockets
// using an Enum-based variant system to avoid heap allocation (no_alloc).

use crate::fs::vfs::VNodeInfo;

#[derive(Debug, Clone, Copy)]
pub enum FdVariant {
    None,
    /// Regular VFS File/Directory (holds the internal node info)
    File(VNodeInfo),
    /// UDP Socket (holds the socket index in SOCKET_MANAGER)
    UdpSocket(usize),
    /// TCP Socket (holds the socket index in TCP_SOCKET_MANAGER)
    TcpSocket(usize),
}

#[derive(Debug, Clone, Copy)]
pub struct FileDescriptor {
    pub variant: FdVariant,
    pub offset: u32, // For file read/write position
}

impl FileDescriptor {
    pub const fn empty() -> Self {
        Self {
            variant: FdVariant::None,
            offset: 0,
        }
    }
}

/// Helper methods to dispatch operations based on the FD variant
impl FileDescriptor {
    /// Read from the file descriptor
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, i64> {
        match self.variant {
            FdVariant::None => Err(-9), // EBADF
            FdVariant::File(ref node) => {
                let vfs_mgr = crate::fs::VFS_MANAGER.lock();
                // We use offset 0 here for simplicity, but we should use self.offset
                match vfs_mgr.read_file(node, buf) {
                    Ok(bytes) => {
                        self.offset += bytes as u32;
                        Ok(bytes)
                    }
                    Err(_) => Err(-5), // EIO
                }
            }
            FdVariant::UdpSocket(idx) => {
                // UDP sockets ignore offset
                let mut sock_mgr = crate::net::socket::SOCKET_MANAGER.lock();
                // Try recv. If block, syscall layer handles it. 
                // For a unified read(), we just return what's available.
                // Normally this would be implemented via sys_recvfrom.
                if let Some(dg) = sock_mgr.try_recv(idx) {
                    let len = dg.len.min(buf.len());
                    buf[..len].copy_from_slice(&dg.data[..len]);
                    Ok(len)
                } else {
                    Err(-11) // EAGAIN
                }
            }
            FdVariant::TcpSocket(_idx) => {
                // To be implemented in Phase 18
                Err(-38) // ENOSYS
            }
        }
    }

    /// Write to the file descriptor
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, i64> {
        match self.variant {
            FdVariant::None => Err(-9),
            FdVariant::File(_) => {
                // FAT32 is currently read-only in XPARQ OS
                Err(-30) // EROFS
            }
            FdVariant::UdpSocket(_idx) => {
                // Needs destination IP/Port, so write() on unconnected UDP isn't standard unless connected
                // (sendto should be used instead).
                Err(-38) // ENOSYS
            }
            FdVariant::TcpSocket(_idx) => {
                // To be implemented in Phase 18
                Err(-38) // ENOSYS
            }
        }
    }
}
