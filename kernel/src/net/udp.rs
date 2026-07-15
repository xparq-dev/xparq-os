// XPARQ OS - Phase 17: UDP Transport Layer
// Implements UDP packet parsing, construction, and checksum verification.
// All operations are zero-allocation and no_std compatible.

/// A parsed UDP datagram (zero-copy view into a buffer).
#[derive(Debug, Clone, Copy)]
pub struct UdpPacket<'a> {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
    pub payload: &'a [u8],
}

impl<'a> UdpPacket<'a> {
    /// Parse a UDP header from a raw byte slice.
    /// Returns `None` if the data is too short or the length field is inconsistent.
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let length = u16::from_be_bytes([data[4], data[5]]);
        let checksum = u16::from_be_bytes([data[6], data[7]]);

        if (length as usize) < 8 || (length as usize) > data.len() {
            return None;
        }

        let payload = &data[8..length as usize];

        Some(Self {
            src_port,
            dst_port,
            length,
            checksum,
            payload,
        })
    }

    /// Serialize a UDP datagram into `out_buf`.
    /// `src_ip` and `dst_ip` are needed for pseudo-header checksum.
    /// Returns the number of bytes written, or 0 on failure.
    pub fn serialize(
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
        out_buf: &mut [u8],
    ) -> usize {
        let total_len = 8 + payload.len();
        if out_buf.len() < total_len || total_len > 0xFFFF {
            return 0;
        }

        out_buf[0..2].copy_from_slice(&src_port.to_be_bytes());
        out_buf[2..4].copy_from_slice(&dst_port.to_be_bytes());
        out_buf[4..6].copy_from_slice(&(total_len as u16).to_be_bytes());
        out_buf[6..8].copy_from_slice(&0u16.to_be_bytes()); // checksum placeholder
        out_buf[8..total_len].copy_from_slice(payload);

        let checksum = udp_checksum(src_ip, dst_ip, &out_buf[0..total_len]);
        out_buf[6..8].copy_from_slice(&checksum.to_be_bytes());

        total_len
    }
}

/// Calculate the UDP checksum over the pseudo-header + UDP segment.
/// The pseudo-header consists of: src_ip, dst_ip, zero, protocol (17), udp_length.
pub fn udp_checksum(src_ip: [u8; 4], dst_ip: [u8; 4], udp_segment: &[u8]) -> u16 {
    let udp_len = udp_segment.len() as u16;
    let mut sum: u32 = 0;

    // Pseudo-header: src IP
    sum += u16::from_be_bytes([src_ip[0], src_ip[1]]) as u32;
    sum += u16::from_be_bytes([src_ip[2], src_ip[3]]) as u32;
    // Pseudo-header: dst IP
    sum += u16::from_be_bytes([dst_ip[0], dst_ip[1]]) as u32;
    sum += u16::from_be_bytes([dst_ip[2], dst_ip[3]]) as u32;
    // Pseudo-header: zero + protocol (0x0011 = 17 for UDP)
    sum += 0x0011u32;
    // Pseudo-header: UDP length
    sum += udp_len as u32;

    // UDP segment
    let mut i = 0;
    while i + 1 < udp_segment.len() {
        sum += u16::from_be_bytes([udp_segment[i], udp_segment[i + 1]]) as u32;
        i += 2;
    }
    if i < udp_segment.len() {
        sum += (udp_segment[i] as u32) << 8;
    }

    // Fold 32-bit sum into 16-bit
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    let result = !(sum as u16);
    // A checksum of 0 is sent as 0xFFFF (per RFC 768)
    if result == 0 { 0xFFFF } else { result }
}
