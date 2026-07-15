// XPARQ OS - xHCI USB 3.0 Host Controller
// High-performance USB driver foundation

use crate::x86_64::pci::{PciDeviceInfo, PciDriver, PCI_BUS_MANAGER};
use core::ptr::{read_volatile, write_volatile};

/// xHCI Transfer Request Block (TRB) - 16 bytes
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Default)]
pub struct XhciTrb {
    pub parameter: u64,
    pub status: u32,
    pub control: u32,
}

/// xHCI Event Ring Segment Table Entry
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, Default)]
pub struct XhciEventRingSegment {
    pub base_addr: u64,
    pub size: u32,
    pub reserved: u32,
}

// Ring sizes
const COMMAND_RING_SIZE: usize = 64;
const EVENT_RING_SIZE: usize = 64;
const MAX_PORTS: usize = 16;

// Static buffers for rings
static mut COMMAND_RING: [XhciTrb; COMMAND_RING_SIZE] = [XhciTrb {
    parameter: 0,
    status: 0,
    control: 0,
}; COMMAND_RING_SIZE];
static mut EVENT_RING: [XhciTrb; EVENT_RING_SIZE] = [XhciTrb {
    parameter: 0,
    status: 0,
    control: 0,
}; EVENT_RING_SIZE];
static mut EVENT_RING_SEGMENT: XhciEventRingSegment = XhciEventRingSegment {
    base_addr: 0,
    size: EVENT_RING_SIZE as u32,
    reserved: 0,
};
static mut DCBAA: [u64; 256] = [0; 256];

/// xHCI Controller Driver
pub struct XhciController {
    pub bar0: u64,
    pub op_base: u64,
    pub rt_base: u64,
    pub db_base: u64,
    pub cmd_ring_index: usize,
    pub cmd_cycle: u8,
    pub event_ring_index: usize,
    pub event_cycle: u8,
    pub initialized: bool,
    pub num_ports: u8,
    pub max_slots: u8,
}

impl XhciController {
    pub const fn new() -> Self {
        Self {
            bar0: 0,
            op_base: 0,
            rt_base: 0,
            db_base: 0,
            cmd_ring_index: 0,
            cmd_cycle: 1,
            event_ring_index: 0,
            event_cycle: 1,
            initialized: false,
            num_ports: 0,
            max_slots: 0,
        }
    }

    pub fn init(&mut self, dev: &PciDeviceInfo) -> Result<(), ()> {
        self.bar0 = dev.bars[0].ok_or(())?.address;

        unsafe {
            let cap_len = read_volatile(self.bar0 as *const u8) as u64;
            self.op_base = self.bar0 + cap_len;

            let hcsparams1 = read_volatile((self.bar0 + 0x04) as *const u32);
            self.num_ports = (hcsparams1 >> 24) as u8;
            self.max_slots = (hcsparams1 & 0xFF) as u8;

            let hccparams1 = read_volatile((self.bar0 + 0x10) as *const u32);
            self.db_base = self.bar0 + read_volatile((self.bar0 + 0x14) as *const u32) as u64;
            self.rt_base = self.bar0 + read_volatile((self.bar0 + 0x18) as *const u32) as u64;

            // 1. Reset Controller
            write_volatile((self.op_base + 0x00) as *mut u32, 0x02); // USBCMD.HCRST = 1
            loop {
                if (read_volatile((self.op_base + 0x00) as *const u32) & 0x02) == 0 {
                    break;
                }
            }
            loop {
                if (read_volatile((self.op_base + 0x04) as *const u32) & 0x800) == 0 {
                    break;
                } // Wait for CNR=0
            }

            // 2. Set Max Device Slots
            let config = read_volatile((self.op_base + 0x38) as *const u32);
            write_volatile(
                (self.op_base + 0x38) as *mut u32,
                (config & !0xFF) | self.max_slots as u32,
            );

            // 3. Set DCBAAP
            write_volatile((self.op_base + 0x30) as *mut u64, &raw const DCBAA as *const _ as u64);

            // 4. Initialize Command Ring
            write_volatile(
                (self.op_base + 0x18) as *mut u64,
                (&raw const COMMAND_RING as *const _ as u64) | 0x1,
            );

            // 5. Initialize Event Ring
            EVENT_RING_SEGMENT.base_addr = &raw const EVENT_RING as *const _ as u64;
            EVENT_RING_SEGMENT.size = EVENT_RING_SIZE as u32;

            // Runtime register for interrupter 0
            let ir_base = self.rt_base + 0x20;
            write_volatile((ir_base + 0x08) as *mut u32, 1); // ERSTZ = 1
            write_volatile(
                (ir_base + 0x10) as *mut u64,
                &raw const EVENT_RING_SEGMENT as *const _ as u64,
            );
            write_volatile((ir_base + 0x18) as *mut u64, &raw const EVENT_RING as *const _ as u64);

            // 6. Setup MSI-X if available (from our new pci foundation)
            let _ = PCI_BUS_MANAGER.lock().map_msix_vector(dev, 0, 45, 0); // Vector 45 for USB

            // 7. Start Controller
            let usbcmd = read_volatile((self.op_base + 0x00) as *const u32);
            write_volatile((self.op_base + 0x00) as *mut u32, usbcmd | 0x01 | 0x04); // RS=1, INTE=1

            self.initialized = true;
        }

        Ok(())
    }

    pub fn submit_command(&mut self, mut trb: XhciTrb) {
        unsafe {
            trb.control &= !0x1;
            if self.cmd_cycle == 1 {
                trb.control |= 1;
            }

            COMMAND_RING[self.cmd_ring_index] = trb;
            self.cmd_ring_index += 1;

            if self.cmd_ring_index == COMMAND_RING_SIZE - 1 {
                // Link TRB
                let mut link = XhciTrb::default();
                link.parameter = &raw const COMMAND_RING as *const _ as u64;
                link.control = (6 << 10) | (1 << 1); // Link TRB, TC=1
                if self.cmd_cycle == 1 {
                    link.control |= 1;
                }
                COMMAND_RING[self.cmd_ring_index] = link;

                self.cmd_ring_index = 0;
                self.cmd_cycle ^= 1;
            }

            // Ring Doorbell
            write_volatile(self.db_base as *mut u32, 0); // Host Controller Command
        }
    }

    pub fn probe_ports(&self) -> u8 {
        let mut count = 0;
        for i in 0..self.num_ports {
            let port_addr = self.op_base + 0x400 + (i as u64 * 0x10);
            unsafe {
                let status = read_volatile(port_addr as *const u32);
                if (status & 0x01) != 0 {
                    // Current Connect Status
                    count += 1;
                }
            }
        }
        count
    }

    pub fn poll_event_ring(&mut self) -> Option<XhciTrb> {
        unsafe {
            let trb = EVENT_RING[self.event_ring_index];
            if (trb.control & 1) as u8 == self.event_cycle {
                self.event_ring_index += 1;
                if self.event_ring_index == EVENT_RING_SIZE {
                    self.event_ring_index = 0;
                    self.event_cycle ^= 1;
                }

                // Update ERDP (Event Ring Dequeue Pointer) and clear EHB (Event Handler Busy)
                let ir_base = self.rt_base + 0x20;
                let erdp = &EVENT_RING[self.event_ring_index] as *const _ as u64;
                write_volatile((ir_base + 0x18) as *mut u64, erdp | 0x08);

                Some(trb)
            } else {
                None
            }
        }
    }

    pub fn reset_port(&self, port_idx: u8) {
        let port_addr = self.op_base + 0x400 + (port_idx as u64 * 0x10);
        unsafe {
            let mut portsc = read_volatile(port_addr as *const u32);
            // Preserve write-1-to-clear bits as 0 so we don't accidentally clear them
            portsc &= !((1 << 17) | (1 << 18) | (1 << 19) | (1 << 20) | (1 << 21) | (1 << 22) | (1 << 23));
            // PR (Port Reset) is bit 4
            portsc |= 1 << 4;
            write_volatile(port_addr as *mut u32, portsc);

            // Wait for PRC (Port Reset Change) bit 21
            loop {
                let status = read_volatile(port_addr as *const u32);
                if (status & (1 << 21)) != 0 {
                    // Clear PRC by writing 1 to it
                    let mut clear_portsc = status;
                    clear_portsc &= !((1 << 17) | (1 << 18) | (1 << 19) | (1 << 20) | (1 << 21) | (1 << 22) | (1 << 23));
                    clear_portsc |= 1 << 21;
                    write_volatile(port_addr as *mut u32, clear_portsc);
                    break;
                }
            }
        }
    }

    pub fn enumerate_devices(&mut self) {
        for i in 0..self.num_ports {
            let port_addr = self.op_base + 0x400 + (i as u64 * 0x10);
            unsafe {
                let status = read_volatile(port_addr as *const u32);
                // CCS (Current Connect Status) is bit 0
                if (status & 0x01) != 0 {
                    self.reset_port(i);

                    // Send Enable Slot Command (TRB Type 9)
                    let mut trb = XhciTrb::default();
                    trb.control = 9 << 10;
                    self.submit_command(trb);

                    // Wait for completion
                    loop {
                        if let Some(event) = self.poll_event_ring() {
                            let trb_type = (event.control >> 10) & 0x3F;
                            if trb_type == 33 { // Command Completion Event
                                let slot_id = (event.control >> 24) & 0xFF;
                                // We have the Slot ID, we would Address Device here
                                // (Saved for future phase)
                                let _ = slot_id;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

// PCI Driver Wrapper
pub struct XhciPciDriver;
pub static XHCI_PCI_DRIVER: XhciPciDriver = XhciPciDriver;

impl PciDriver for XhciPciDriver {
    fn probe(&self, dev: &PciDeviceInfo) -> Result<(), ()> {
        if dev.class_code.base == 0x0c
            && dev.class_code.sub == 0x03
            && dev.class_code.interface == 0x30
        {
            let mut xhci = XHCI_CONTROLLER.lock();
            xhci.init(dev)?;
            Ok(())
        } else {
            Err(())
        }
    }
    fn get_supported_devices(&self) -> &'static [(u16, u16)] {
        &[(0x0000, 0x0000)]
    }
}

use spin::Mutex;
pub static XHCI_CONTROLLER: Mutex<XhciController> = Mutex::new(XhciController::new());
