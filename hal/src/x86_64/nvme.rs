// XPARQ OS - NVMe Driver
// High-performance NVMe storage driver

use crate::x86_64::pci::{PciDeviceInfo, PciDriver, PCI_BUS_MANAGER};
use core::ptr::{read_volatile, write_volatile};

// NVMe Register Offsets
const NVME_REG_CAP: u64 = 0x00; // Controller Capabilities
const NVME_REG_VS: u64 = 0x08; // Version
const NVME_REG_INTMS: u64 = 0x0C; // Interrupt Mask Set
const NVME_REG_INTMC: u64 = 0x10; // Interrupt Mask Clear
const NVME_REG_CC: u64 = 0x14; // Controller Configuration
const NVME_REG_CSTS: u64 = 0x1C; // Controller Status
const NVME_REG_AQA: u64 = 0x24; // Admin Queue Attributes
const NVME_REG_ASQ: u64 = 0x28; // Admin Submission Queue Base Address
const NVME_REG_ACQ: u64 = 0x30; // Admin Completion Queue Base Address

// NVMe Submission Queue Entry (64 bytes)
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, Default)]
pub struct NvmeSqEntry {
    pub cdw0: u32, // Opcode, Fuse, PSDT, CID
    pub nsid: u32, // Namespace Identifier
    pub _reserved: u64,
    pub mptr: u64,      // Metadata Pointer
    pub dptr: [u64; 2], // Data Pointer (PRP1, PRP2)
    pub cdw10: u32,
    pub cdw11: u32,
    pub cdw12: u32,
    pub cdw13: u32,
    pub cdw14: u32,
    pub cdw15: u32,
}

// NVMe Completion Queue Entry (16 bytes)
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default)]
pub struct NvmeCqEntry {
    pub dw0: u32,    // Command Specific
    pub dw1: u32,    // Reserved
    pub dw2: u16,    // SQ Head Pointer
    pub dw2_1: u16,  // SQ ID
    pub cid: u16,    // Command Identifier
    pub status: u16, // Status, Phase Tag
}

// Admin Queue Sizes
const ADMIN_QUEUE_SIZE: u16 = 8;

// Static Admin Queues
static mut ADMIN_SQ: [NvmeSqEntry; ADMIN_QUEUE_SIZE as usize] = [NvmeSqEntry {
    cdw0: 0,
    nsid: 0,
    _reserved: 0,
    mptr: 0,
    dptr: [0; 2],
    cdw10: 0,
    cdw11: 0,
    cdw12: 0,
    cdw13: 0,
    cdw14: 0,
    cdw15: 0,
}; ADMIN_QUEUE_SIZE as usize];

static mut ADMIN_CQ: [NvmeCqEntry; ADMIN_QUEUE_SIZE as usize] = [NvmeCqEntry {
    dw0: 0,
    dw1: 0,
    dw2: 0,
    dw2_1: 0,
    cid: 0,
    status: 0,
}; ADMIN_QUEUE_SIZE as usize];

/// NVMe Driver
pub struct NvmeDriver {
    bar0: u64,
    db_stride: u32,
    admin_sq_tail: u16,
    admin_cq_head: u16,
    admin_phase: u16,
    pub initialized: bool,
    pub nsid: u32,
    pub lba_count: u64,
    pub lba_size: u32,
}

impl NvmeDriver {
    pub const fn new() -> Self {
        Self {
            bar0: 0,
            db_stride: 0,
            admin_sq_tail: 0,
            admin_cq_head: 0,
            admin_phase: 1,
            initialized: false,
            nsid: 0,
            lba_count: 0,
            lba_size: 0,
        }
    }

    pub fn init(&mut self, dev: &PciDeviceInfo) -> Result<(), ()> {
        for bar in &dev.bars {
            if let Some(bar) = bar {
                if bar.index == 0 {
                    self.bar0 = bar.address;
                }
            }
        }
        if self.bar0 == 0 {
            return Err(());
        }

        unsafe {
            // 1. Read Capabilities
            let cap = read_volatile(self.bar0 as *const u64);
            self.db_stride = 1 << (2 + ((cap >> 32) & 0xF));

            // 2. Reset Controller
            let cc = read_volatile((self.bar0 + NVME_REG_CC) as *const u32);
            write_volatile((self.bar0 + NVME_REG_CC) as *mut u32, cc & !0x1); // EN = 0

            // Wait for CSTS.RDY = 0
            loop {
                let csts = read_volatile((self.bar0 + NVME_REG_CSTS) as *const u32);
                if (csts & 0x1) == 0 {
                    break;
                }
            }

            // 3. Set Admin Queue Attributes
            let aqa = ((ADMIN_QUEUE_SIZE - 1) as u32) | (((ADMIN_QUEUE_SIZE - 1) as u32) << 16);
            write_volatile((self.bar0 + NVME_REG_AQA) as *mut u32, aqa);

            // 4. Set Admin Queue Addresses
            write_volatile(
                (self.bar0 + NVME_REG_ASQ) as *mut u64,
                &raw const ADMIN_SQ as *const _ as u64,
            );
            write_volatile(
                (self.bar0 + NVME_REG_ACQ) as *mut u64,
                &raw const ADMIN_CQ as *const _ as u64,
            );

            // 5. Enable Controller
            let cc = (0 << 16) | (0 << 14) | (6 << 7) | (4 << 4) | 0x1; // CSS=NVM, MPS=4KB, AMS=RR, EN=1
            write_volatile((self.bar0 + NVME_REG_CC) as *mut u32, cc);

            // Wait for CSTS.RDY = 1
            loop {
                let csts = read_volatile((self.bar0 + NVME_REG_CSTS) as *const u32);
                if (csts & 0x1) != 0 {
                    break;
                }
            }
        }

        self.initialized = true;

        // Identify Namespace 1
        self.identify(1)?;

        Ok(())
    }

    fn identify(&mut self, nsid: u32) -> Result<(), ()> {
        let mut buffer = [0u8; 4096];
        let mut cmd = NvmeSqEntry::default();
        cmd.cdw0 = 0x06; // Identify Opcode
        cmd.nsid = nsid;
        cmd.dptr[0] = buffer.as_mut_ptr() as u64;
        cmd.cdw10 = 0x00; // Identify Namespace

        self.submit_admin_cmd(cmd)?;

        // Extract LBA count and size from identify buffer
        self.nsid = nsid;
        self.lba_count = u64::from_le_bytes(buffer[0..8].try_into().unwrap());
        self.lba_size = 1 << buffer[128 + (buffer[100] as usize) * 4 + 3]; // Simplified

        Ok(())
    }

    fn submit_admin_cmd(&mut self, mut cmd: NvmeSqEntry) -> Result<NvmeCqEntry, ()> {
        unsafe {
            cmd.cdw0 |= (self.admin_sq_tail as u32) << 16; // CID
            ADMIN_SQ[self.admin_sq_tail as usize] = cmd;

            self.admin_sq_tail = (self.admin_sq_tail + 1) % ADMIN_QUEUE_SIZE;

            // Ring Doorbell
            let db_addr = self.bar0 + 0x1000;
            write_volatile(db_addr as *mut u32, self.admin_sq_tail as u32);

            // Wait for completion
            loop {
                let cq_entry = ADMIN_CQ[self.admin_cq_head as usize];
                let phase = (cq_entry.status >> 15) & 0x1;

                if phase == self.admin_phase {
                    self.admin_cq_head = (self.admin_cq_head + 1) % ADMIN_QUEUE_SIZE;
                    if self.admin_cq_head == 0 {
                        self.admin_phase ^= 1;
                    }

                    // Update CQ Doorbell
                    let db_addr = self.bar0 + 0x1000 + self.db_stride as u64;
                    write_volatile(db_addr as *mut u32, self.admin_cq_head as u32);

                    return Ok(cq_entry);
                }
            }
        }
    }

    pub fn read_blocks(&mut self, lba: u64, count: u16, buffer: *mut u8) -> Result<(), ()> {
        // For simplicity, we use the Admin queue for I/O in this Phase 2 driver
        // (Real drivers should create I/O queues)
        let mut cmd = NvmeSqEntry::default();
        cmd.cdw0 = 0x02; // Read Opcode
        cmd.nsid = self.nsid;
        cmd.dptr[0] = buffer as u64;
        cmd.cdw10 = (lba & 0xFFFFFFFF) as u32;
        cmd.cdw11 = (lba >> 32) as u32;
        cmd.cdw12 = (count - 1) as u32;

        self.submit_admin_cmd(cmd).map(|_| ())
    }

    pub fn write_blocks(&mut self, lba: u64, count: u16, buffer: *const u8) -> Result<(), ()> {
        // For simplicity, we use the Admin queue for I/O in this Phase 2/3 driver
        // (Real drivers should create I/O queues)
        let mut cmd = NvmeSqEntry::default();
        cmd.cdw0 = 0x01; // Write Opcode
        cmd.nsid = self.nsid;
        cmd.dptr[0] = buffer as u64;
        cmd.cdw10 = (lba & 0xFFFFFFFF) as u32;
        cmd.cdw11 = (lba >> 32) as u32;
        cmd.cdw12 = (count - 1) as u32;

        self.submit_admin_cmd(cmd).map(|_| ())
    }
}

impl Default for NvmeDriver {
    fn default() -> Self {
        Self::new()
    }
}

// Static NVMe driver
use spin::Mutex;
pub static NVME_DRIVER: Mutex<NvmeDriver> = Mutex::new(NvmeDriver::new());

// PciDriver implementation for NVMe
pub struct NvmePciDriver;
pub static NVME_PCI_DRIVER: NvmePciDriver = NvmePciDriver;

impl PciDriver for NvmePciDriver {
    fn probe(&self, dev: &PciDeviceInfo) -> Result<(), ()> {
        // Check if NVMe controller (Class 0x01, Subclass 0x08, Interface 0x02)
        if dev.class_code.base == 0x01 && dev.class_code.sub == 0x08 {
            let mut driver = NVME_DRIVER.lock();
            driver.init(dev)?;
            Ok(())
        } else {
            Err(())
        }
    }

    fn get_supported_devices(&self) -> &'static [(u16, u16)] {
        // Wildcard for any NVMe controller
        &[(0x0000, 0x0000)]
    }
}
