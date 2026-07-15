// XPARQ OS - Phase 18: Pluggable TCP Congestion Control
// Defines the CC interface and a basic flow-control-only implementation.

pub trait CongestionControl {
    /// Called when an ACK is received that acknowledges new data.
    fn on_ack_received(&mut self, bytes_acked: usize, rtt: u32);
    
    /// Called when a packet is dropped or RTO expires.
    fn on_packet_loss(&mut self);
    
    /// Called to determine how many bytes we are allowed to send right now.
    /// This will be min(cwnd, rwnd).
    fn allowed_to_send(&self, rwnd: usize) -> usize;
    
    /// Called when we send data.
    fn on_data_sent(&mut self, bytes: usize);
}

/// A No-Op Congestion Control that only respects the receiver's advertised window (rwnd).
/// No slow-start, no congestion avoidance. Useful for initial RFC 793 verification.
#[derive(Clone, Copy)]
pub struct NoOpCc;

impl NoOpCc {
    pub const fn new() -> Self {
        Self
    }
}

impl CongestionControl for NoOpCc {
    fn on_ack_received(&mut self, _bytes_acked: usize, _rtt: u32) {
        // No-op
    }

    fn on_packet_loss(&mut self) {
        // No-op
    }

    fn allowed_to_send(&self, rwnd: usize) -> usize {
        rwnd
    }

    fn on_data_sent(&mut self, _bytes: usize) {
        // No-op
    }
}
