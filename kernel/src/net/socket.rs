// XPARQ OS - Phase 17: Kernel UDP Socket API
// Provides a static socket table for kernel tasks and user space processes.
// Zero-allocation, no_std design using fixed-size arrays and WaitQueues.

use spin::Mutex;
use crate::task::wait_queue::WaitQueue;

/// Maximum number of simultaneously open sockets.
pub const MAX_SOCKETS: usize = 16;
/// Maximum number of pending datagrams per socket.
pub const MAX_PENDING_DATAGRAMS: usize = 8;
/// Maximum payload size per datagram (including UDP header).
pub const MAX_DATAGRAM_PAYLOAD: usize = 1472; // 1500 MTU - 20 IP - 8 UDP

/// A received datagram stored in the socket's inbox.
#[derive(Clone, Copy)]
pub struct PendingDatagram {
    pub src_ip: [u8; 4],
    pub src_port: u16,
    pub data: [u8; MAX_DATAGRAM_PAYLOAD],
    pub len: usize,
}

impl PendingDatagram {
    pub const fn empty() -> Self {
        Self {
            src_ip: [0; 4],
            src_port: 0,
            data: [0; MAX_DATAGRAM_PAYLOAD],
            len: 0,
        }
    }
}

/// A single UDP socket entry in the socket table.
pub struct UdpSocket {
    /// Whether this slot is in use.
    pub in_use: bool,
    /// The local port this socket is bound to. 0 = unbound.
    pub bound_port: u16,
    /// Incoming datagram inbox (circular ring).
    pub inbox: [PendingDatagram; MAX_PENDING_DATAGRAMS],
    pub inbox_head: usize,
    pub inbox_tail: usize,
    pub inbox_count: usize,
    /// Wait queue for blocking `recvfrom` calls.
    pub wait_queue: WaitQueue<8>,
}

impl UdpSocket {
    pub const fn new() -> Self {
        const EMPTY: PendingDatagram = PendingDatagram::empty();
        Self {
            in_use: false,
            bound_port: 0,
            inbox: [EMPTY; MAX_PENDING_DATAGRAMS],
            inbox_head: 0,
            inbox_tail: 0,
            inbox_count: 0,
            wait_queue: WaitQueue::new(),
        }
    }

    /// Push an incoming datagram into the socket's inbox ring buffer.
    /// Returns false if the inbox is full (datagram is dropped).
    pub fn push_datagram(&mut self, src_ip: [u8; 4], src_port: u16, data: &[u8]) -> bool {
        if self.inbox_count >= MAX_PENDING_DATAGRAMS {
            return false; // Drop packet: inbox full
        }
        let len = data.len().min(MAX_DATAGRAM_PAYLOAD);
        let entry = &mut self.inbox[self.inbox_tail];
        entry.src_ip = src_ip;
        entry.src_port = src_port;
        entry.data[..len].copy_from_slice(&data[..len]);
        entry.len = len;

        self.inbox_tail = (self.inbox_tail + 1) % MAX_PENDING_DATAGRAMS;
        self.inbox_count += 1;
        true
    }

    /// Pop a datagram from the inbox. Returns None if empty.
    pub fn pop_datagram(&mut self) -> Option<PendingDatagram> {
        if self.inbox_count == 0 {
            return None;
        }
        let entry = self.inbox[self.inbox_head];
        self.inbox_head = (self.inbox_head + 1) % MAX_PENDING_DATAGRAMS;
        self.inbox_count -= 1;
        Some(entry)
    }
}

/// Global socket table.
pub struct SocketManager {
    pub sockets: [UdpSocket; MAX_SOCKETS],
}

pub static SOCKET_MANAGER: Mutex<SocketManager> = Mutex::new(SocketManager::new());

impl SocketManager {
    pub const fn new() -> Self {
        const EMPTY: UdpSocket = UdpSocket::new();
        Self {
            sockets: [EMPTY; MAX_SOCKETS],
        }
    }

    /// Allocate a new socket. Returns the socket file descriptor (index + 100 to distinguish from
    /// regular FDs), or -1 if no slots are free.
    pub fn alloc_socket(&mut self) -> i64 {
        for i in 0..MAX_SOCKETS {
            if !self.sockets[i].in_use {
                self.sockets[i].in_use = true;
                self.sockets[i].bound_port = 0;
                self.sockets[i].inbox_head = 0;
                self.sockets[i].inbox_tail = 0;
                self.sockets[i].inbox_count = 0;
                return i as i64;
            }
        }
        -1
    }

    /// Bind a socket to a local port. Returns 0 on success, negative errno on failure.
    pub fn bind_socket(&mut self, sockfd: usize, port: u16) -> i64 {
        let idx = sockfd;
        if idx >= MAX_SOCKETS || !self.sockets[idx].in_use {
            return -9; // -EBADF
        }
        // Check for port conflict
        for i in 0..MAX_SOCKETS {
            if self.sockets[i].in_use && self.sockets[i].bound_port == port && i != idx {
                return -98; // -EADDRINUSE
            }
        }
        self.sockets[idx].bound_port = port;
        0
    }

    /// Close a socket, freeing its slot.
    pub fn close_socket(&mut self, sockfd: usize) -> i64 {
        let idx = sockfd;
        if idx >= MAX_SOCKETS || !self.sockets[idx].in_use {
            return -9;
        }
        self.sockets[idx].in_use = false;
        self.sockets[idx].bound_port = 0;
        0
    }

    /// Deliver an incoming UDP packet to the appropriate bound socket.
    /// Returns true if a socket consumed the packet.
    pub fn deliver(&mut self, dst_port: u16, src_ip: [u8; 4], src_port: u16, payload: &[u8]) -> bool {
        for i in 0..MAX_SOCKETS {
            if self.sockets[i].in_use && self.sockets[i].bound_port == dst_port {
                let delivered = self.sockets[i].push_datagram(src_ip, src_port, payload);
                if delivered {
                    // Wake any task blocked in recvfrom on this socket
                    self.sockets[i].wait_queue.wake_one();
                }
                return delivered;
            }
        }
        false
    }

    /// Non-blocking recv: returns Some(datagram) if data is available, None otherwise.
    pub fn try_recv(&mut self, sockfd: usize) -> Option<PendingDatagram> {
        let idx = sockfd;
        if idx >= MAX_SOCKETS || !self.sockets[idx].in_use {
            return None;
        }
        self.sockets[idx].pop_datagram()
    }
}
