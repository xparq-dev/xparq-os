// XPARQ OS - Phase 16/17: Basic Network Stack (TCP/IP + UDP Sockets)

pub mod e1000;
pub mod ethernet;
pub mod arp;
pub mod ipv4;
pub mod icmp;
pub mod udp;
pub mod socket;
pub mod tcp_buffer;
pub mod tcp_cc;
pub mod tcp_packet;
pub mod tcp;
pub mod port_allocator;

use spin::Mutex;
use crate::hal;
use hal::connectivity::ConnectivityDriver;

// Our OS default IP: 10.0.2.15 (typical QEMU user net)
pub const OUR_IP: [u8; 4] = [10, 0, 2, 15];

pub struct NetworkManager {
    pub is_initialized: bool,
    pub mac_address: [u8; 6],
}

pub static NETWORK_MANAGER: Mutex<NetworkManager> = Mutex::new(NetworkManager::new());

impl NetworkManager {
    pub const fn new() -> Self {
        Self {
            is_initialized: false,
            mac_address: [0; 6],
        }
    }

    pub fn init(&mut self) {
        if self.is_initialized {
            return;
        }

        // Initialize E1000 Wrapper
        e1000::init();
        
        let e1000 = hal::x86_64::e1000::E1000_DRIVER.lock();
        self.mac_address = e1000.get_info().mac_address;
        
        self.is_initialized = true;
    }

    pub fn poll(&self) {
        if !self.is_initialized {
            return;
        }

        let mut e1000 = hal::x86_64::e1000::E1000_DRIVER.lock();
        let mut rx_buf = [0u8; 2048];
        let mut tx_buf = [0u8; 2048];

        if let Ok(len) = e1000.receive(&mut rx_buf) {
            if len == 0 {
                return;
            }

            let frame_data = &rx_buf[0..len];
            if let Some(frame) = ethernet::EthernetFrame::parse(frame_data) {
                // If the frame is not for us and not broadcast, ignore
                if frame.dest_mac != self.mac_address && frame.dest_mac != [0xFF; 6] {
                    return;
                }

                if frame.ethertype == ethernet::ETHERTYPE_ARP {
                    if let Some(arp) = arp::ArpPacket::parse(frame.payload) {
                        if arp.opcode == arp::ARP_OPCODE_REQUEST && arp.target_ip == OUR_IP {
                            // Generate ARP Reply
                            let reply = arp::ArpPacket {
                                opcode: arp::ARP_OPCODE_REPLY,
                                sender_mac: self.mac_address,
                                sender_ip: OUR_IP,
                                target_mac: arp.sender_mac,
                                target_ip: arp.sender_ip,
                            };

                            let mut arp_buf = [0u8; 28];
                            let arp_len = reply.to_bytes(&mut arp_buf);

                            let eth_len = ethernet::EthernetFrame::to_bytes(
                                arp.sender_mac,
                                self.mac_address,
                                ethernet::ETHERTYPE_ARP,
                                &arp_buf[0..arp_len],
                                &mut tx_buf
                            );
                            
                            let _ = e1000.send(&tx_buf[0..eth_len]);
                        }
                    }
                } else if frame.ethertype == ethernet::ETHERTYPE_IPV4 {
                    if let Some(ipv4_pkt) = ipv4::Ipv4Packet::parse(frame.payload) {
                        if ipv4_pkt.dest_ip == OUR_IP {
                            match ipv4_pkt.protocol {
                                ipv4::IP_PROTOCOL_ICMP => {
                                    if let Some(icmp) = icmp::IcmpPacket::parse(ipv4_pkt.payload) {
                                        if icmp.icmp_type == icmp::ICMP_TYPE_ECHO_REQUEST {
                                            // Generate ICMP Echo Reply
                                            let reply = icmp::IcmpPacket {
                                                icmp_type: icmp::ICMP_TYPE_ECHO_REPLY,
                                                code: 0,
                                                identifier: icmp.identifier,
                                                sequence: icmp.sequence,
                                                payload: icmp.payload,
                                            };

                                            let mut icmp_buf = [0u8; 1500];
                                            let icmp_len = reply.to_bytes(&mut icmp_buf);

                                            let ipv4_reply = ipv4::Ipv4Packet {
                                                protocol: ipv4::IP_PROTOCOL_ICMP,
                                                src_ip: OUR_IP,
                                                dest_ip: ipv4_pkt.src_ip,
                                                payload: &icmp_buf[0..icmp_len],
                                            };

                                            let mut ip_buf = [0u8; 1500];
                                            let ip_len = ipv4_reply.to_bytes(&mut ip_buf);

                                            let eth_len = ethernet::EthernetFrame::to_bytes(
                                                frame.src_mac,
                                                self.mac_address,
                                                ethernet::ETHERTYPE_IPV4,
                                                &ip_buf[0..ip_len],
                                                &mut tx_buf
                                            );

                                            let _ = e1000.send(&tx_buf[0..eth_len]);
                                        }
                                    }
                                }
                                ipv4::IP_PROTOCOL_UDP => {
                                    // Route incoming UDP packets to bound sockets
                                    if let Some(udp_pkt) = udp::UdpPacket::parse(ipv4_pkt.payload) {
                                        // Drop e1000 lock before taking socket lock to avoid deadlock
                                        drop(e1000);
                                        socket::SOCKET_MANAGER.lock().deliver(
                                            udp_pkt.dst_port,
                                            ipv4_pkt.src_ip,
                                            udp_pkt.src_port,
                                            udp_pkt.payload,
                                        );
                                        return; // Already dropped e1000
                                    }
                                }
                                ipv4::IP_PROTOCOL_TCP => {
                                    drop(e1000);
                                    tcp::TCP_SOCKET_MANAGER.lock().receive_packet(
                                        ipv4_pkt.src_ip,
                                        OUR_IP,
                                        ipv4_pkt.payload,
                                    );
                                    return;
                                }
                                _ => {} // Unknown protocol, ignore
                            }
                        }
                    }
                }
            }
        }
    }
}

