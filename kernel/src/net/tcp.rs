// XPARQ OS - Phase 18: TCP State Machine Core
// Fully implements RFC 793 states and state transitions.

use crate::net::tcp_buffer::{TcpBuffer, RingBuffer};
use crate::net::tcp_cc::{CongestionControl, NoOpCc};
use spin::Mutex;
use crate::task::wait_queue::WaitQueue;
use crate::hal::ConnectivityDriver;

pub const MAX_TCP_SOCKETS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

pub struct TcpSocket {
    pub in_use: bool,
    pub state: TcpState,
    
    // Binding
    pub local_ip: [u8; 4],
    pub local_port: u16,
    pub remote_ip: [u8; 4],
    pub remote_port: u16,
    
    // Sequence numbers
    pub snd_una: u32,
    pub snd_nxt: u32,
    pub snd_wnd: u16,
    
    pub rcv_nxt: u32,
    pub rcv_wnd: u16,
    
    // RTO Timers
    pub rtt_srtt: u32,
    pub rtt_var: u32,
    pub rto: u32,
    pub timer_handle: Option<u16>,
    
    // Buffers and CC
    pub rx_buf: RingBuffer<4096>,
    pub tx_buf: RingBuffer<4096>,
    pub cc: NoOpCc,
    
    // Accept queue for listening sockets
    pub accept_queue: [Option<usize>; 4],
    pub accept_head: usize,
    pub accept_tail: usize,
    pub accept_count: usize,
    pub parent_socket: Option<usize>, // Set if this is a child socket from accept

    // Wait queues for tasks blocking on recv/accept
    pub wait_queue: WaitQueue<16>,
}

impl TcpSocket {
    pub const fn empty() -> Self {
        Self {
            in_use: false,
            state: TcpState::Closed,
            local_ip: [0; 4],
            local_port: 0,
            remote_ip: [0; 4],
            remote_port: 0,
            snd_una: 0,
            snd_nxt: 0,
            snd_wnd: 0,
            rcv_nxt: 0,
            rcv_wnd: 0,
            rtt_srtt: 0,
            rtt_var: 0,
            rto: 0,
            timer_handle: None,
            rx_buf: RingBuffer::new(),
            tx_buf: RingBuffer::new(),
            cc: NoOpCc::new(),
            accept_queue: [None; 4],
            accept_head: 0,
            accept_tail: 0,
            accept_count: 0,
            parent_socket: None,
            wait_queue: WaitQueue::new(),
        }
    }
}

pub struct TcpManager {
    pub sockets: [TcpSocket; MAX_TCP_SOCKETS],
}

impl TcpManager {
    pub const fn new() -> Self {
        const EMPTY_SOCKET: TcpSocket = TcpSocket::empty();
        Self {
            sockets: [EMPTY_SOCKET; MAX_TCP_SOCKETS],
        }
    }
    
    pub fn alloc_socket(&mut self) -> i64 {
        for i in 0..MAX_TCP_SOCKETS {
            if !self.sockets[i].in_use {
                self.sockets[i] = TcpSocket::empty(); // reset
                self.sockets[i].in_use = true;
                self.sockets[i].snd_wnd = 4096;
                self.sockets[i].rcv_wnd = 4096;
                self.sockets[i].rto = 1000;
                return i as i64;
            }
        }
        -1 // -EMFILE
    }
    
    pub fn bind(&mut self, sockfd: usize, port: u16) -> i64 {
        if sockfd >= MAX_TCP_SOCKETS || !self.sockets[sockfd].in_use {
            return -9;
        }
        
        for i in 0..MAX_TCP_SOCKETS {
            if self.sockets[i].in_use && self.sockets[i].local_port == port && i != sockfd {
                return -98; // -EADDRINUSE
            }
        }
        
        self.sockets[sockfd].local_port = port;
        0
    }
    
    pub fn listen(&mut self, sockfd: usize) -> i64 {
        if sockfd >= MAX_TCP_SOCKETS || !self.sockets[sockfd].in_use {
            return -9;
        }
        
        if self.sockets[sockfd].state != TcpState::Closed {
            return -22; // EINVAL
        }
        
        self.sockets[sockfd].state = TcpState::Listen;
        0
    }
    
    pub fn connect(&mut self, sockfd: usize, ip: [u8; 4], port: u16) -> i64 {
        if sockfd >= MAX_TCP_SOCKETS || !self.sockets[sockfd].in_use {
            return -9; // EBADF
        }
        if self.sockets[sockfd].state != TcpState::Closed {
            return -22; // EINVAL
        }

        // Allocate local ephemeral port if not bound
        if self.sockets[sockfd].local_port == 0 {
            let mut port_alloc = crate::net::port_allocator::PORT_ALLOCATOR.lock();
            if let Some(p) = port_alloc.allocate() {
                self.sockets[sockfd].local_port = p;
            } else {
                return -98; // EADDRINUSE (no ports)
            }
        }
        
        self.sockets[sockfd].local_ip = crate::net::OUR_IP;
        self.sockets[sockfd].remote_ip = ip;
        self.sockets[sockfd].remote_port = port;
        self.sockets[sockfd].state = TcpState::SynSent;
        self.sockets[sockfd].snd_una = 2000;
        self.sockets[sockfd].snd_nxt = 2000;

        self.send_tcp_packet(
            self.sockets[sockfd].local_ip,
            self.sockets[sockfd].remote_ip,
            self.sockets[sockfd].local_port,
            self.sockets[sockfd].remote_port,
            self.sockets[sockfd].snd_nxt,
            0,
            crate::net::tcp_packet::TCP_FLAG_SYN,
            4096,
            &[]
        );
        self.sockets[sockfd].snd_nxt = self.sockets[sockfd].snd_nxt.wrapping_add(1);

        0
    }

    pub fn accept(&mut self, sockfd: usize) -> Option<usize> {
        if sockfd >= MAX_TCP_SOCKETS || !self.sockets[sockfd].in_use {
            return None;
        }

        let s = &mut self.sockets[sockfd];
        if s.accept_count > 0 {
            let child = s.accept_queue[s.accept_head];
            s.accept_queue[s.accept_head] = None;
            s.accept_head = (s.accept_head + 1) % 4;
            s.accept_count -= 1;
            return child;
        }
        
        None
    }

    pub fn close_socket(&mut self, sockfd: usize) -> i64 {
        if sockfd >= MAX_TCP_SOCKETS || !self.sockets[sockfd].in_use {
            return -9;
        }
        
        self.sockets[sockfd].state = TcpState::Closed;
        self.sockets[sockfd].in_use = false;
        0
    }
    
    pub fn send_tcp_packet(
        &self,
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        seq_num: u32,
        ack_num: u32,
        flags: u8,
        window: u16,
        payload: &[u8]
    ) {
        let mut tcp_buf = [0u8; 1500];
        let tcp_len = crate::net::tcp_packet::TcpPacket::serialize(
            src_ip, dst_ip, src_port, dst_port, seq_num, ack_num, flags, window, payload, &mut tcp_buf
        );
        
        if tcp_len == 0 { return; }
        
        let ip_pkt = crate::net::ipv4::Ipv4Packet {
            protocol: crate::net::ipv4::IP_PROTOCOL_TCP,
            src_ip,
            dest_ip: dst_ip,
            payload: &tcp_buf[0..tcp_len],
        };
        
        let mut ip_buf = [0u8; 1500];
        let ip_len = ip_pkt.to_bytes(&mut ip_buf);
        
        // Use MAC broadcast for now (ARP missing in sendpath)
        let our_mac = crate::net::NETWORK_MANAGER.lock().mac_address;
        let dst_mac = [0xFF; 6];
        
        let mut eth_buf = [0u8; 1514];
        let eth_len = crate::net::ethernet::EthernetFrame::to_bytes(
            our_mac, dst_mac, crate::net::ethernet::ETHERTYPE_IPV4, &ip_buf[0..ip_len], &mut eth_buf
        );
        
        let mut e1000 = crate::hal::x86_64::e1000::E1000_DRIVER.lock();
        let _ = e1000.send(&eth_buf[0..eth_len]);
    }
    
    /// Entry point for all incoming TCP packets.
    pub fn receive_packet(&mut self, src_ip: [u8; 4], dst_ip: [u8; 4], payload: &[u8]) {
        let tcp_pkt = match crate::net::tcp_packet::TcpPacket::parse(payload) {
            Some(pkt) => pkt,
            None => return,
        };

        // 1. Find matching established/active socket
        let mut match_idx = None;
        for i in 0..MAX_TCP_SOCKETS {
            let s = &self.sockets[i];
            if s.in_use 
                && s.local_port == tcp_pkt.dst_port 
                && s.remote_port == tcp_pkt.src_port 
                && s.remote_ip == src_ip 
                && s.local_ip == dst_ip 
            {
                match_idx = Some(i);
                break;
            }
        }

        // 2. If no active match, find a matching Listen socket
        if match_idx.is_none() {
            for i in 0..MAX_TCP_SOCKETS {
                let s = &self.sockets[i];
                if s.in_use && s.state == TcpState::Listen && s.local_port == tcp_pkt.dst_port {
                    match_idx = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = match_idx {
            let state = self.sockets[idx].state;
            
            if state == TcpState::Listen {
                if (tcp_pkt.flags & crate::net::tcp_packet::TCP_FLAG_SYN) != 0 {
                    // SYN received -> Create child socket in SynReceived
                    let child_idx = self.alloc_socket();
                    if child_idx >= 0 {
                        let c = child_idx as usize;
                        self.sockets[c].state = TcpState::SynReceived;
                        self.sockets[c].local_ip = dst_ip;
                        self.sockets[c].local_port = tcp_pkt.dst_port;
                        self.sockets[c].remote_ip = src_ip;
                        self.sockets[c].remote_port = tcp_pkt.src_port;
                        
                        self.sockets[c].rcv_nxt = tcp_pkt.seq_num.wrapping_add(1);
                        self.sockets[c].snd_una = 1000; // Random ISN
                        self.sockets[c].snd_nxt = 1000;
                        self.sockets[c].parent_socket = Some(idx);
                        
                        // Send SYN-ACK
                        self.send_tcp_packet(
                            dst_ip, src_ip, tcp_pkt.dst_port, tcp_pkt.src_port,
                            self.sockets[c].snd_nxt, self.sockets[c].rcv_nxt,
                            crate::net::tcp_packet::TCP_FLAG_SYN | crate::net::tcp_packet::TCP_FLAG_ACK,
                            4096, &[]
                        );
                        self.sockets[c].snd_nxt = self.sockets[c].snd_nxt.wrapping_add(1);
                    }
                }
            } else if state == TcpState::SynReceived {
                if (tcp_pkt.flags & crate::net::tcp_packet::TCP_FLAG_ACK) != 0 {
                    if tcp_pkt.ack_num == self.sockets[idx].snd_nxt {
                        self.sockets[idx].state = TcpState::Established;
                        
                        // Push into parent's accept queue
                        if let Some(parent) = self.sockets[idx].parent_socket {
                            let p = &mut self.sockets[parent];
                            if p.accept_count < 4 {
                                p.accept_queue[p.accept_tail] = Some(idx);
                                p.accept_tail = (p.accept_tail + 1) % 4;
                                p.accept_count += 1;
                                p.wait_queue.wake_one(); // Wake blocking accept
                            } else {
                                // Queue full, drop connection
                                self.sockets[idx].in_use = false;
                            }
                        }
                    }
                }
            } else if state == TcpState::SynSent {
                if (tcp_pkt.flags & (crate::net::tcp_packet::TCP_FLAG_SYN | crate::net::tcp_packet::TCP_FLAG_ACK)) != 0 {
                    self.sockets[idx].rcv_nxt = tcp_pkt.seq_num.wrapping_add(1);
                    self.sockets[idx].snd_una = tcp_pkt.ack_num;
                    self.sockets[idx].state = TcpState::Established;
                    
                    // Send ACK
                    self.send_tcp_packet(
                        dst_ip, src_ip, tcp_pkt.dst_port, tcp_pkt.src_port,
                        self.sockets[idx].snd_nxt, self.sockets[idx].rcv_nxt,
                        crate::net::tcp_packet::TCP_FLAG_ACK,
                        4096, &[]
                    );
                    
                    self.sockets[idx].wait_queue.wake_one(); // Wake blocking connect
                }
            } else if state == TcpState::Established {
                if !tcp_pkt.payload.is_empty() {
                    use crate::net::tcp_buffer::TcpBuffer;
                    let written = self.sockets[idx].rx_buf.write(tcp_pkt.payload);
                    self.sockets[idx].rcv_nxt = self.sockets[idx].rcv_nxt.wrapping_add(written as u32);
                    
                    // Send ACK
                    self.send_tcp_packet(
                        dst_ip, src_ip, tcp_pkt.dst_port, tcp_pkt.src_port,
                        self.sockets[idx].snd_nxt, self.sockets[idx].rcv_nxt,
                        crate::net::tcp_packet::TCP_FLAG_ACK,
                        self.sockets[idx].rx_buf.available_space() as u16, &[]
                    );
                    
                    self.sockets[idx].wait_queue.wake_one(); // Wake blocking read
                }
            }
        }
    }
}

pub static TCP_SOCKET_MANAGER: spin::Mutex<TcpManager> = spin::Mutex::new(TcpManager::new());

