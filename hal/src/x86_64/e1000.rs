// XPARQ OS - Intel E1000 Gigabit Ethernet Driver
// Provides network connectivity via the Intel 8254x controller series.

use crate::connectivity::{
    ConnectivityCapabilities, ConnectivityDeviceInfo, ConnectivityDeviceType,
    ConnectivityDriver, ConnectivityError, ConnectivityInterface,
};
use crate::x86_64::pci::{PciDeviceInfo, PciDriver};
use core::ptr::{read_volatile, write_volatile};

// E1000 Register Offsets
const REG_CTRL: usize = 0x0000;
const REG_STATUS: usize = 0x0008;
const REG_EEPROM: usize = 0x0014;
const REG_CTRL_EXT: usize = 0x0018;
const REG_IMASK: usize = 0x00D0;
const REG_RCTRL: usize = 0x0100;
const REG_RXDESCLO: usize = 0x2800;
const REG_RXDESCHI: usize = 0x2804;
const REG_RXDESCLEN: usize = 0x2808;
const REG_RXDESCHEAD: usize = 0x2810;
const REG_RXDESCTAIL: usize = 0x2818;
const REG_TCTRL: usize = 0x0400;
const REG_TXDESCLO: usize = 0x3800;
const REG_TXDESCHI: usize = 0x3804;
const REG_TXDESCLEN: usize = 0x3808;
const REG_TXDESCHEAD: usize = 0x3810;
const REG_TXDESCTAIL: usize = 0x3818;
const REG_MTA: usize = 0x5200; // Multicast Table Array
const REG_RAL: usize = 0x5400; // Receive Address Low
const REG_RAH: usize = 0x5404; // Receive Address High

// Number of descriptors
const NUM_RX_DESCRIPTORS: usize = 32;
const NUM_TX_DESCRIPTORS: usize = 8;

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Default)]
struct RxDescriptor {
    addr: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Default)]
struct TxDescriptor {
    addr: u64,
    length: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u16,
}

static mut RX_RING: [RxDescriptor; NUM_RX_DESCRIPTORS] = [RxDescriptor {
    addr: 0,
    length: 0,
    checksum: 0,
    status: 0,
    errors: 0,
    special: 0,
}; NUM_RX_DESCRIPTORS];

static mut TX_RING: [TxDescriptor; NUM_TX_DESCRIPTORS] = [TxDescriptor {
    addr: 0,
    length: 0,
    cso: 0,
    cmd: 0,
    status: 0,
    css: 0,
    special: 0,
}; NUM_TX_DESCRIPTORS];

// Static buffers for RX/TX
const BUFFER_SIZE: usize = 2048;
static mut RX_BUFFERS: [[u8; BUFFER_SIZE]; NUM_RX_DESCRIPTORS] = [[0; BUFFER_SIZE]; NUM_RX_DESCRIPTORS];
static mut TX_BUFFERS: [[u8; BUFFER_SIZE]; NUM_TX_DESCRIPTORS] = [[0; BUFFER_SIZE]; NUM_TX_DESCRIPTORS];

pub struct E1000Driver {
    bar0: u64,
    mac_address: [u8; 6],
    initialized: bool,
    rx_cur: usize,
    tx_cur: usize,
    enabled: bool,
}

impl E1000Driver {
    pub const fn new() -> Self {
        Self {
            bar0: 0,
            mac_address: [0; 6],
            initialized: false,
            rx_cur: 0,
            tx_cur: 0,
            enabled: false,
        }
    }

    fn write_reg(&self, offset: usize, value: u32) {
        unsafe {
            write_volatile((self.bar0 + offset as u64) as *mut u32, value);
        }
    }

    fn read_reg(&self, offset: usize) -> u32 {
        unsafe { read_volatile((self.bar0 + offset as u64) as *const u32) }
    }

    fn detect_eeprom(&self) -> bool {
        self.write_reg(REG_EEPROM, 1);
        for _ in 0..1000 {
            let val = self.read_reg(REG_EEPROM);
            if (val & 0x10) != 0 {
                return true;
            }
        }
        false
    }

    fn read_eeprom(&self, addr: u8) -> u16 {
        let mut tmp: u32 = 0;
        let eeprom_exists = self.detect_eeprom();

        if eeprom_exists {
            self.write_reg(REG_EEPROM, 1 | ((addr as u32) << 8));
            loop {
                tmp = self.read_reg(REG_EEPROM);
                if (tmp & (1 << 4)) != 0 {
                    break;
                }
            }
        } else {
            self.write_reg(REG_EEPROM, 1 | ((addr as u32) << 2));
            loop {
                tmp = self.read_reg(REG_EEPROM);
                if (tmp & (1 << 1)) != 0 {
                    break;
                }
            }
        }
        ((tmp >> 16) & 0xFFFF) as u16
    }

    fn read_mac_address(&mut self) {
        if self.detect_eeprom() {
            let temp = self.read_eeprom(0);
            self.mac_address[0] = (temp & 0xFF) as u8;
            self.mac_address[1] = (temp >> 8) as u8;
            let temp = self.read_eeprom(1);
            self.mac_address[2] = (temp & 0xFF) as u8;
            self.mac_address[3] = (temp >> 8) as u8;
            let temp = self.read_eeprom(2);
            self.mac_address[4] = (temp & 0xFF) as u8;
            self.mac_address[5] = (temp >> 8) as u8;
        } else {
            // Read from registers (RAL/RAH)
            let mac_low = self.read_reg(REG_RAL);
            let mac_high = self.read_reg(REG_RAH);
            self.mac_address[0] = (mac_low & 0xFF) as u8;
            self.mac_address[1] = ((mac_low >> 8) & 0xFF) as u8;
            self.mac_address[2] = ((mac_low >> 16) & 0xFF) as u8;
            self.mac_address[3] = ((mac_low >> 24) & 0xFF) as u8;
            self.mac_address[4] = (mac_high & 0xFF) as u8;
            self.mac_address[5] = ((mac_high >> 8) & 0xFF) as u8;
        }
    }

    fn rx_init(&mut self) {
        unsafe {
            for i in 0..NUM_RX_DESCRIPTORS {
                RX_RING[i].addr = &raw const RX_BUFFERS[i] as *const _ as u64;
                RX_RING[i].status = 0;
            }

            self.write_reg(REG_RXDESCLO, &raw const RX_RING as *const _ as u32);
            self.write_reg(REG_RXDESCHI, 0);

            self.write_reg(REG_RXDESCLEN, (NUM_RX_DESCRIPTORS * 16) as u32);
            self.write_reg(REG_RXDESCHEAD, 0);
            self.write_reg(REG_RXDESCTAIL, (NUM_RX_DESCRIPTORS - 1) as u32);

            self.rx_cur = 0;
            
            // RCTRL: EN (bit 1) | SBP (bit 2) | UPE (bit 3) | MPE (bit 4) | LBM_NONE (0<<6) | RTOM (bit 15) | BSIZE_2048 (0<<16) | SECRC (bit 26)
            self.write_reg(REG_RCTRL, 0x0400801E); 
        }
    }

    fn tx_init(&mut self) {
        unsafe {
            for i in 0..NUM_TX_DESCRIPTORS {
                TX_RING[i].addr = 0;
                TX_RING[i].cmd = 0;
                TX_RING[i].status = 1; // DD (Descriptor Done) bit
            }

            self.write_reg(REG_TXDESCLO, &raw const TX_RING as *const _ as u32);
            self.write_reg(REG_TXDESCHI, 0);

            self.write_reg(REG_TXDESCLEN, (NUM_TX_DESCRIPTORS * 16) as u32);
            self.write_reg(REG_TXDESCHEAD, 0);
            self.write_reg(REG_TXDESCTAIL, 0);

            self.tx_cur = 0;

            // TCTRL: EN (bit 1) | PSP (bit 3) | CT (15<<4) | COLD (0x3f<<12) | RTLC (bit 24)
            self.write_reg(REG_TCTRL, 0x0103F0FA); 
        }
    }

    pub fn init(&mut self, dev: &PciDeviceInfo) -> Result<(), ()> {
        // Find BAR0 (Memory Mapped Registers)
        for bar in &dev.bars {
            if let Some(bar) = bar {
                if bar.index == 0 && !bar.is_io {
                    self.bar0 = bar.address;
                    break;
                }
            }
        }

        if self.bar0 == 0 {
            return Err(());
        }

        self.read_mac_address();

        // Clear multicast table
        for i in 0..128 {
            self.write_reg(REG_MTA + (i * 4), 0);
        }

        // Disable interrupts
        self.write_reg(REG_IMASK, 0x00000000);

        self.rx_init();
        self.tx_init();

        // Set Link Up
        let ctrl = self.read_reg(REG_CTRL);
        self.write_reg(REG_CTRL, ctrl | 0x40);

        self.initialized = true;
        self.enabled = true;

        Ok(())
    }
}

impl Default for E1000Driver {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectivityDriver for E1000Driver {
    fn name(&self) -> &'static str {
        "Intel E1000 Gigabit Ethernet"
    }

    fn init(&mut self) -> Result<(), ConnectivityError> {
        Ok(())
    }

    fn get_info(&self) -> ConnectivityDeviceInfo {
        ConnectivityDeviceInfo {
            device_type: ConnectivityDeviceType::Ethernet,
            interface: ConnectivityInterface::PCIe,
            vendor_id: 0x8086,
            product_id: 0x100E,
            model: "Intel 82540EM Gigabit Ethernet Controller",
            serial: "N/A",
            mac_address: self.mac_address,
            capabilities: ConnectivityCapabilities::GIGABIT_ETHERNET,
        }
    }

    fn is_connected(&self) -> bool {
        self.initialized && self.enabled
    }

    fn connect(&mut self) -> Result<(), ConnectivityError> {
        self.enabled = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), ConnectivityError> {
        self.enabled = false;
        Ok(())
    }

    fn send(&mut self, data: &[u8]) -> Result<usize, ConnectivityError> {
        if !self.enabled || data.len() > BUFFER_SIZE {
            return Err(ConnectivityError::InvalidParameter);
        }

        unsafe {
            // Wait for descriptor to be done
            while (TX_RING[self.tx_cur].status & 1) == 0 {
                // Should use a proper wait or timeout here
            }

            // Copy data to TX buffer
            let tx_buffer_ptr = &mut TX_BUFFERS[self.tx_cur] as *mut u8;
            core::ptr::copy_nonoverlapping(data.as_ptr(), tx_buffer_ptr, data.len());

            TX_RING[self.tx_cur].addr = tx_buffer_ptr as u64;
            TX_RING[self.tx_cur].length = data.len() as u16;
            TX_RING[self.tx_cur].cmd = 0x0B; // EOP, IFCS, RS
            TX_RING[self.tx_cur].status = 0;

            let old_cur = self.tx_cur;
            self.tx_cur = (self.tx_cur + 1) % NUM_TX_DESCRIPTORS;

            self.write_reg(REG_TXDESCTAIL, self.tx_cur as u32);

            // Wait for transmission
            while (TX_RING[old_cur].status & 1) == 0 {}
        }

        Ok(data.len())
    }

    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, ConnectivityError> {
        if !self.enabled {
            return Err(ConnectivityError::Disconnected);
        }

        unsafe {
            let status = RX_RING[self.rx_cur].status;
            if (status & 1) == 0 {
                // No packet
                return Ok(0);
            }

            let len = RX_RING[self.rx_cur].length as usize;
            if len > buffer.len() {
                // Buffer too small, drop it for now
                RX_RING[self.rx_cur].status = 0;
                self.write_reg(REG_RXDESCTAIL, self.rx_cur as u32);
                self.rx_cur = (self.rx_cur + 1) % NUM_RX_DESCRIPTORS;
                return Err(ConnectivityError::InvalidParameter);
            }

            let rx_buffer_ptr = &RX_BUFFERS[self.rx_cur] as *const u8;
            core::ptr::copy_nonoverlapping(rx_buffer_ptr, buffer.as_mut_ptr(), len);

            RX_RING[self.rx_cur].status = 0;
            self.write_reg(REG_RXDESCTAIL, self.rx_cur as u32);
            self.rx_cur = (self.rx_cur + 1) % NUM_RX_DESCRIPTORS;

            Ok(len)
        }
    }

    fn set_enabled(&mut self, enabled: bool) -> Result<(), ConnectivityError> {
        self.enabled = enabled;
        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// Static E1000 Driver
use spin::Mutex;
pub static E1000_DRIVER: Mutex<E1000Driver> = Mutex::new(E1000Driver::new());

// PciDriver
pub struct E1000PciDriver;
pub static E1000_PCI_DRIVER: E1000PciDriver = E1000PciDriver;

impl PciDriver for E1000PciDriver {
    fn probe(&self, dev: &PciDeviceInfo) -> Result<(), ()> {
        // Known Intel E1000 Vendor/Device IDs
        if dev.device_id.vendor_id == 0x8086 && 
           (dev.device_id.device_id == 0x100E || // 82540EM
            dev.device_id.device_id == 0x100F || // 82545EM
            dev.device_id.device_id == 0x10D3)   // 82574L
        {
            let mut driver = E1000_DRIVER.lock();
            driver.init(dev)?;
            Ok(())
        } else {
            Err(())
        }
    }

    fn get_supported_devices(&self) -> &'static [(u16, u16)] {
        &[
            (0x8086, 0x100E), // Intel 82540EM Gigabit Ethernet Controller
            (0x8086, 0x100F), // Intel 82545EM Gigabit Ethernet Controller
            (0x8086, 0x10D3), // Intel 82574L Gigabit Network Connection
        ]
    }
}
