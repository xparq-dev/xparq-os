// XPARQ OS - Phase 16: ARP Subsystem
use crate::hal;

pub const ARP_OPCODE_REQUEST: u16 = 1;
pub const ARP_OPCODE_REPLY: u16 = 2;

#[derive(Debug)]
pub struct ArpPacket {
    pub opcode: u16,
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
}

impl ArpPacket {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 28 {
            return None;
        }

        let hw_type = u16::from_be_bytes([data[0], data[1]]);
        let proto_type = u16::from_be_bytes([data[2], data[3]]);
        let hw_size = data[4];
        let proto_size = data[5];

        if hw_type != 1 || proto_type != 0x0800 || hw_size != 6 || proto_size != 4 {
            return None; // We only support Ethernet + IPv4
        }

        let opcode = u16::from_be_bytes([data[6], data[7]]);
        
        let mut sender_mac = [0; 6];
        sender_mac.copy_from_slice(&data[8..14]);
        let mut sender_ip = [0; 4];
        sender_ip.copy_from_slice(&data[14..18]);

        let mut target_mac = [0; 6];
        target_mac.copy_from_slice(&data[18..24]);
        let mut target_ip = [0; 4];
        target_ip.copy_from_slice(&data[24..28]);

        Some(Self {
            opcode,
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        })
    }

    pub fn to_bytes(&self, out_buf: &mut [u8]) -> usize {
        if out_buf.len() < 28 {
            return 0;
        }

        out_buf[0..2].copy_from_slice(&1u16.to_be_bytes()); // Ethernet
        out_buf[2..4].copy_from_slice(&0x0800u16.to_be_bytes()); // IPv4
        out_buf[4] = 6; // MAC size
        out_buf[5] = 4; // IP size
        out_buf[6..8].copy_from_slice(&self.opcode.to_be_bytes());
        
        out_buf[8..14].copy_from_slice(&self.sender_mac);
        out_buf[14..18].copy_from_slice(&self.sender_ip);
        
        out_buf[18..24].copy_from_slice(&self.target_mac);
        out_buf[24..28].copy_from_slice(&self.target_ip);

        28
    }
}
