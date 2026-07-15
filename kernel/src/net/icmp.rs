// XPARQ OS - Phase 16: ICMP Subsystem

pub const ICMP_TYPE_ECHO_REPLY: u8 = 0;
pub const ICMP_TYPE_ECHO_REQUEST: u8 = 8;

#[derive(Debug)]
pub struct IcmpPacket<'a> {
    pub icmp_type: u8,
    pub code: u8,
    pub identifier: u16,
    pub sequence: u16,
    pub payload: &'a [u8],
}

impl<'a> IcmpPacket<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }

        let icmp_type = data[0];
        let code = data[1];
        let identifier = u16::from_be_bytes([data[4], data[5]]);
        let sequence = u16::from_be_bytes([data[6], data[7]]);
        let payload = &data[8..];

        Some(Self {
            icmp_type,
            code,
            identifier,
            sequence,
            payload,
        })
    }

    pub fn calculate_checksum(data: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        let mut i = 0;
        while i < data.len() {
            if i + 1 < data.len() {
                let word = u16::from_be_bytes([data[i], data[i + 1]]);
                sum += word as u32;
            } else {
                let word = (data[i] as u16) << 8;
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
        let total_length = 8 + payload_len;
        if out_buf.len() < total_length {
            return 0;
        }

        out_buf[0] = self.icmp_type;
        out_buf[1] = self.code;
        out_buf[2..4].copy_from_slice(&0u16.to_be_bytes()); // Checksum initially 0
        out_buf[4..6].copy_from_slice(&self.identifier.to_be_bytes());
        out_buf[6..8].copy_from_slice(&self.sequence.to_be_bytes());
        out_buf[8..total_length].copy_from_slice(self.payload);

        let checksum = Self::calculate_checksum(&out_buf[0..total_length]);
        out_buf[2..4].copy_from_slice(&checksum.to_be_bytes());

        total_length
    }
}
