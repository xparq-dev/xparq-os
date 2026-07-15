// XPARQ OS - Phase 19: TCP Packet Parsing & Serialization
// Implements TCP Header parsing, checksum calculation, and generation.

pub const TCP_FLAG_FIN: u8 = 0x01;
pub const TCP_FLAG_SYN: u8 = 0x02;
pub const TCP_FLAG_RST: u8 = 0x04;
pub const TCP_FLAG_PSH: u8 = 0x08;
pub const TCP_FLAG_ACK: u8 = 0x10;
pub const TCP_FLAG_URG: u8 = 0x20;

#[derive(Debug)]
pub struct TcpPacket<'a> {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub data_offset: u8,
    pub flags: u8,
    pub window: u16,
    pub checksum: u16,
    pub urg_ptr: u16,
    pub payload: &'a [u8],
}

impl<'a> TcpPacket<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        if data.len() < 20 {
            return None;
        }

        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let seq_num = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ack_num = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        
        let data_offset_raw = data[12];
        let data_offset = (data_offset_raw >> 4) * 4; // header length in bytes
        
        if data.len() < data_offset as usize {
            return None;
        }

        let flags = data[13];
        let window = u16::from_be_bytes([data[14], data[15]]);
        let checksum = u16::from_be_bytes([data[16], data[17]]);
        let urg_ptr = u16::from_be_bytes([data[18], data[19]]);
        
        let payload = &data[(data_offset as usize)..];

        Some(Self {
            src_port,
            dst_port,
            seq_num,
            ack_num,
            data_offset,
            flags,
            window,
            checksum,
            urg_ptr,
            payload,
        })
    }

    pub fn serialize(
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        seq_num: u32,
        ack_num: u32,
        flags: u8,
        window: u16,
        payload: &[u8],
        buf: &mut [u8],
    ) -> usize {
        let total_len = 20 + payload.len();
        if buf.len() < total_len {
            return 0;
        }

        buf[0..2].copy_from_slice(&src_port.to_be_bytes());
        buf[2..4].copy_from_slice(&dst_port.to_be_bytes());
        buf[4..8].copy_from_slice(&seq_num.to_be_bytes());
        buf[8..12].copy_from_slice(&ack_num.to_be_bytes());
        
        buf[12] = (20 / 4) << 4; // Data offset (5 words)
        buf[13] = flags;
        
        buf[14..16].copy_from_slice(&window.to_be_bytes());
        buf[16..18].copy_from_slice(&[0, 0]); // Checksum placeholder
        buf[18..20].copy_from_slice(&[0, 0]); // Urgent pointer

        if !payload.is_empty() {
            buf[20..total_len].copy_from_slice(payload);
        }

        let cs = Self::compute_checksum(src_ip, dst_ip, &buf[..total_len]);
        buf[16..18].copy_from_slice(&cs.to_be_bytes());

        total_len
    }

    pub fn compute_checksum(src_ip: [u8; 4], dst_ip: [u8; 4], packet: &[u8]) -> u16 {
        let mut sum = 0u32;

        // Pseudo Header
        for i in (0..4).step_by(2) {
            sum += u16::from_be_bytes([src_ip[i], src_ip[i+1]]) as u32;
            sum += u16::from_be_bytes([dst_ip[i], dst_ip[i+1]]) as u32;
        }
        sum += 6u32; // IP_PROTOCOL_TCP
        sum += packet.len() as u32;

        // TCP Header + Payload
        let mut i = 0;
        while i < packet.len() - 1 {
            sum += u16::from_be_bytes([packet[i], packet[i+1]]) as u32;
            i += 2;
        }
        if i < packet.len() {
            sum += u16::from_be_bytes([packet[i], 0]) as u32;
        }

        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !(sum as u16)
    }
}
