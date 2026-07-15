// XPARQ OS - Phase 16: Ethernet Subsystem
use crate::hal;

pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const ETHERTYPE_ARP: u16 = 0x0806;

#[derive(Debug)]
pub struct EthernetFrame<'a> {
    pub dest_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ethertype: u16,
    pub payload: &'a [u8],
}

impl<'a> EthernetFrame<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        if data.len() < 14 {
            return None; // Ethernet header is 14 bytes
        }

        let mut dest_mac = [0; 6];
        dest_mac.copy_from_slice(&data[0..6]);

        let mut src_mac = [0; 6];
        src_mac.copy_from_slice(&data[6..12]);

        let ethertype = u16::from_be_bytes([data[12], data[13]]);
        let payload = &data[14..];

        Some(Self {
            dest_mac,
            src_mac,
            ethertype,
            payload,
        })
    }

    pub fn to_bytes(dest: [u8; 6], src: [u8; 6], ethertype: u16, payload: &[u8], out_buf: &mut [u8]) -> usize {
        let total_len = 14 + payload.len();
        if out_buf.len() < total_len {
            return 0;
        }

        out_buf[0..6].copy_from_slice(&dest);
        out_buf[6..12].copy_from_slice(&src);
        out_buf[12..14].copy_from_slice(&ethertype.to_be_bytes());
        out_buf[14..total_len].copy_from_slice(payload);

        total_len
    }
}
