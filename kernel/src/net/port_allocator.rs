// XPARQ OS - Phase 18: Ephemeral Port Allocator
// Implements RFC 6056 for secure ephemeral port allocation (49152 - 65535)

use spin::Mutex;

const EPHEMERAL_PORT_MIN: u16 = 49152;
const EPHEMERAL_PORT_MAX: u16 = 65535;
const EPHEMERAL_PORT_COUNT: usize = (EPHEMERAL_PORT_MAX - EPHEMERAL_PORT_MIN + 1) as usize;

pub struct PortAllocator {
    next_port: u16,
    // A simple bitset to track port usage (16384 bits = 2048 bytes)
    used_ports: [u8; EPHEMERAL_PORT_COUNT / 8],
}

impl PortAllocator {
    pub const fn new() -> Self {
        Self {
            next_port: EPHEMERAL_PORT_MIN,
            used_ports: [0; EPHEMERAL_PORT_COUNT / 8],
        }
    }

    /// Allocates an ephemeral port.
    /// In a real system, this should be randomized per RFC 6056.
    pub fn allocate(&mut self) -> Option<u16> {
        let start_port = self.next_port;

        loop {
            let port = self.next_port;
            self.next_port = if self.next_port == EPHEMERAL_PORT_MAX {
                EPHEMERAL_PORT_MIN
            } else {
                self.next_port + 1
            };

            let idx = (port - EPHEMERAL_PORT_MIN) as usize;
            let byte_idx = idx / 8;
            let bit_idx = idx % 8;

            if (self.used_ports[byte_idx] & (1 << bit_idx)) == 0 {
                // Port is free
                self.used_ports[byte_idx] |= 1 << bit_idx;
                return Some(port);
            }

            if self.next_port == start_port {
                // We checked all ports
                return None;
            }
        }
    }

    /// Frees an ephemeral port
    pub fn free(&mut self, port: u16) {
        if port >= EPHEMERAL_PORT_MIN && port <= EPHEMERAL_PORT_MAX {
            let idx = (port - EPHEMERAL_PORT_MIN) as usize;
            let byte_idx = idx / 8;
            let bit_idx = idx % 8;
            self.used_ports[byte_idx] &= !(1 << bit_idx);
        }
    }
}

pub static PORT_ALLOCATOR: Mutex<PortAllocator> = Mutex::new(PortAllocator::new());
