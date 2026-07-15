// XPARQ OS - Phase 16/17: IPv4 + UDP constants

pub const IP_PROTOCOL_ICMP: u8 = 1;
pub const IP_PROTOCOL_UDP: u8 = 17;
pub const IP_PROTOCOL_TCP: u8 = 6;

#[derive(Debug)]
pub struct Ipv4Packet<'a> {
    pub protocol: u8,
    pub src_ip: [u8; 4],
    pub dest_ip: [u8; 4],
    pub payload: &'a [u8],
}

impl<'a> Ipv4Packet<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        if data.len() < 20 {
            return None;
        }

        let version_ihl = data[0];
        if (version_ihl >> 4) != 4 {
            return None; // Not IPv4
        }

        let ihl = (version_ihl & 0x0F) * 4;
        if data.len() < ihl as usize {
            return None;
        }

        let total_length = u16::from_be_bytes([data[2], data[3]]) as usize;
        if data.len() < total_length {
            return None;
        }

        let protocol = data[9];
        
        let mut src_ip = [0; 4];
        src_ip.copy_from_slice(&data[12..16]);
        
        let mut dest_ip = [0; 4];
        dest_ip.copy_from_slice(&data[16..20]);

        let payload = &data[ihl as usize..total_length];

        Some(Self {
            protocol,
            src_ip,
            dest_ip,
            payload,
        })
    }

    pub fn calculate_checksum(header: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        let mut i = 0;
        while i < header.len() {
            if i + 1 < header.len() {
                let word = u16::from_be_bytes([header[i], header[i + 1]]);
                sum += word as u32;
            }
            i += 2;
        }
        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        !(sum as u16)
    }

    pub fn to_bytes(&self, out_buf: &mut [u8]) -> usize {
        let payload_len = self.payload.len();
        let total_length = 20 + payload_len;
        if out_buf.len() < total_length {
            return 0;
        }

        out_buf[0] = 0x45; // Version 4, IHL 5
        out_buf[1] = 0;    // DSCP / ECN
        out_buf[2..4].copy_from_slice(&(total_length as u16).to_be_bytes());
        out_buf[4..6].copy_from_slice(&0u16.to_be_bytes()); // ID
        out_buf[6..8].copy_from_slice(&0u16.to_be_bytes()); // Flags/Frag
        out_buf[8] = 64; // TTL
        out_buf[9] = self.protocol;
        out_buf[10..12].copy_from_slice(&0u16.to_be_bytes()); // Initial checksum is 0
        out_buf[12..16].copy_from_slice(&self.src_ip);
        out_buf[16..20].copy_from_slice(&self.dest_ip);

        let checksum = Self::calculate_checksum(&out_buf[0..20]);
        out_buf[10..12].copy_from_slice(&checksum.to_be_bytes());

        out_buf[20..total_length].copy_from_slice(self.payload);

        total_length
    }
}
