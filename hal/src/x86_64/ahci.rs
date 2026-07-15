// XPARQ OS - AHCI/SATA Driver
// AHCI (Advanced Host Controller Interface) driver for SATA drives

use crate::x86_64::pci::{PciDeviceInfo, PciDriver};
use core::ptr::{read_volatile, write_volatile};

// AHCI Host Controller Register Offsets
const AHCI_REG_CAP: u64 = 0x00; // Host Capabilities
const AHCI_REG_GHC: u64 = 0x04; // Global Host Control
const AHCI_REG_IS: u64 = 0x08; // Interrupt Status
const AHCI_REG_PI: u64 = 0x0C; // Ports Implemented
const AHCI_REG_VS: u64 = 0x10; // Version
const AHCI_REG_CCC_CTL: u64 = 0x14; // Command Completion Coalescing Control
const AHCI_REG_CCC_PORTS: u64 = 0x18; // Command Completion Coalescing Ports
const AHCI_REG_EM_LOC: u64 = 0x1C; // Enclosure Management Location
const AHCI_REG_EM_CTL: u64 = 0x20; // Enclosure Management Control
const AHCI_REG_CAP2: u64 = 0x24; // Host Capabilities Extended
const AHCI_REG_BOHC: u64 = 0x28; // BIOS/OS Handoff Control and Status

// AHCI Port Register Offsets
const AHCI_PORT_REG_CLB: u64 = 0x00; // Command List Base Address
const AHCI_PORT_REG_CLBU: u64 = 0x04; // Command List Base Address Upper 32 Bits
const AHCI_PORT_REG_FB: u64 = 0x08; // FIS Base Address
const AHCI_PORT_REG_FBU: u64 = 0x0C; // FIS Base Address Upper 32 Bits
const AHCI_PORT_REG_IS: u64 = 0x10; // Interrupt Status
const AHCI_PORT_REG_IE: u64 = 0x14; // Interrupt Enable
const AHCI_PORT_REG_CMD: u64 = 0x18; // Command and Status
const AHCI_PORT_REG_TFD: u64 = 0x20; // Task File Data
const AHCI_PORT_REG_SIG: u64 = 0x24; // Signature
const AHCI_PORT_REG_SSTS: u64 = 0x28; // SATA Status (SStatus)
const AHCI_PORT_REG_SCTL: u64 = 0x2C; // SATA Control (SControl)
const AHCI_PORT_REG_SERR: u64 = 0x30; // SATA Error (SError)
const AHCI_PORT_REG_SACT: u64 = 0x34; // SATA Active (SActive)
const AHCI_PORT_REG_CI: u64 = 0x38; // Command Issue
const AHCI_PORT_REG_SNTF: u64 = 0x3C; // SATA Notification (SNotification)
const AHCI_PORT_REG_FBS: u64 = 0x40; // FIS-based Switching Control
const AHCI_PORT_REG_DEVSLP: u64 = 0x44; // Device Sleep

// AHCI Command List Entry Structure
#[repr(C, align(128))]
#[derive(Clone, Copy, Debug)]
struct AhciCommandHeader {
    flags: u16,
    prdtl: u16,
    prdbc: u32,
    ctba: u64,
    _reserved: [u8; 16],
}

// AHCI Physical Region Descriptor Table Entry
#[repr(C, align(128))]
#[derive(Clone, Copy, Debug)]
struct AhciPrdtEntry {
    dba: u64,
    _reserved: u32,
    dbc: u32, // Data Byte Count (0-based)
}

// AHCI FIS Structures

// Host to Device FIS (Register - Host to Device)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct AhciH2DFis {
    fis_type: u8,
    flags: u8,
    command: u8,
    features_low: u8,
    lba0: u8,
    lba1: u8,
    lba2: u8,
    device: u8,
    lba3: u8,
    lba4: u8,
    lba5: u8,
    features_high: u8,
    count_low: u8,
    count_high: u8,
    icc: u8,
    control: u8,
    _reserved: [u8; 4],
}

// AHCI Received FIS Structure
#[repr(C, align(256))]
#[derive(Debug)]
struct AhciReceivedFis {
    dsfis: [u8; 28], // DMA Setup FIS
    _pad1: [u8; 4],
    psfis: [u8; 20], // PIO Setup FIS
    _pad2: [u8; 12],
    rfis: [u8; 20], // D2H Register FIS
    _pad3: [u8; 4],
    sdbfis: [u8; 8], // Set Device Bits FIS
    ufis: [u8; 64],  // Unknown FIS
    _pad4: [u8; 96],
}

// Command Table (holds Command FIS and PRDT)
#[repr(C, align(128))]
struct AhciCommandTable {
    command_fis: [u8; 64], // Command FIS
    atapi_cmd: [u8; 16],   // ATAPI Command
    _reserved: [u8; 48],
    prdt: [AhciPrdtEntry; 8], // PRDT
}

// Static Command Tables
static mut CMD_TABLES: [[AhciCommandTable; NUM_CMD_SLOTS]; NUM_AHCI_PORTS] = [const {
    [const {
        AhciCommandTable {
            command_fis: [0; 64],
            atapi_cmd: [0; 16],
            _reserved: [0; 48],
            prdt: [const {
                AhciPrdtEntry {
                    dba: 0,
                    _reserved: 0,
                    dbc: 0,
                }
            }; 8],
        }
    }; NUM_CMD_SLOTS]
}; NUM_AHCI_PORTS];

// Static buffers for AHCI (aligned properly)
const NUM_AHCI_PORTS: usize = 32;
const NUM_CMD_SLOTS: usize = 32;

#[repr(C, align(1024))]
struct CommandList([AhciCommandHeader; NUM_CMD_SLOTS]);
#[repr(C, align(256))]
struct ReceivedFis(AhciReceivedFis);

// Static allocations
static mut CMD_LISTS: [CommandList; NUM_AHCI_PORTS] = [const {
    CommandList(
        [AhciCommandHeader {
            flags: 0,
            prdtl: 0,
            prdbc: 0,
            ctba: 0,
            _reserved: [0; 16],
        }; NUM_CMD_SLOTS],
    )
}; NUM_AHCI_PORTS];
static mut FIS_BUFFERS: [ReceivedFis; NUM_AHCI_PORTS] = [const {
    ReceivedFis(AhciReceivedFis {
        dsfis: [0; 28],
        _pad1: [0; 4],
        psfis: [0; 20],
        _pad2: [0; 12],
        rfis: [0; 20],
        _pad3: [0; 4],
        sdbfis: [0; 8],
        ufis: [0; 64],
        _pad4: [0; 96],
    })
}; NUM_AHCI_PORTS];

/// AHCI Driver
pub struct AhciDriver {
    bar5: u64,
    num_ports: usize,
    active_ports: [bool; 32],
    initialized: bool,
}

impl AhciDriver {
    /// Create a new AHCI driver instance
    pub const fn new() -> Self {
        Self {
            bar5: 0,
            num_ports: 0,
            active_ports: [false; 32],
            initialized: false,
        }
    }

    /// Initialize AHCI driver with given device info
    pub fn init(&mut self, dev: &PciDeviceInfo) -> Result<(), ()> {
        // Find BAR5 (memory-mapped registers)
        for bar in &dev.bars {
            if let Some(bar) = bar {
                if bar.index == 5 {
                    self.bar5 = bar.address;
                }
            }
        }

        if self.bar5 == 0 {
            return Err(());
        }

        // Read HBA capabilities
        let cap = unsafe { read_volatile((self.bar5 + AHCI_REG_CAP) as *const u32) };
        self.num_ports = ((cap >> 8) & 0x1F) as usize + 1;

        // Enable HBA
        let ghc = unsafe { read_volatile((self.bar5 + AHCI_REG_GHC) as *const u32) };
        unsafe { write_volatile((self.bar5 + AHCI_REG_GHC) as *mut u32, ghc | 0x80000000) };

        // Enable interrupts
        unsafe { write_volatile((self.bar5 + AHCI_REG_GHC) as *mut u32, ghc | 0x00000002) };

        // Initialize each implemented port
        let pi = unsafe { read_volatile((self.bar5 + AHCI_REG_PI) as *const u32) };
        for port in 0..32 {
            if (pi & (1 << port)) != 0 {
                if self.init_port(port).is_ok() {
                    let port_base = self.bar5 + 0x100 + (port as u64 * 0x80);
                    let ssts =
                        unsafe { read_volatile((port_base + AHCI_PORT_REG_SSTS) as *const u32) };
                    if (ssts & 0x0F) == 3 {
                        // Device present and communication established
                        self.active_ports[port] = true;
                    }
                }
            }
        }

        self.initialized = true;
        Ok(())
    }

    /// Initialize a specific AHCI port
    fn init_port(&mut self, port: usize) -> Result<(), ()> {
        let port_base = self.bar5 + 0x100 + (port as u64 * 0x80);

        // Stop port
        unsafe {
            let cmd = read_volatile((port_base + AHCI_PORT_REG_CMD) as *const u32);
            write_volatile(
                (port_base + AHCI_PORT_REG_CMD) as *mut u32,
                cmd & !0x00000010,
            ); // Clear ST
            write_volatile(
                (port_base + AHCI_PORT_REG_CMD) as *mut u32,
                cmd & !0x00000001,
            ); // Clear CR
            loop {
                let cmd = read_volatile((port_base + AHCI_PORT_REG_CMD) as *const u32);
                if (cmd & 0x00008000) == 0 {
                    break;
                }
            }
        }

        // Set up command list and FIS buffer
        unsafe {
            let cmd_list_addr = &CMD_LISTS[port] as *const _ as u64;
            let fis_addr = &FIS_BUFFERS[port] as *const _ as u64;

            write_volatile(
                (port_base + AHCI_PORT_REG_CLB) as *mut u32,
                (cmd_list_addr & 0xFFFFFFFF) as u32,
            );
            write_volatile(
                (port_base + AHCI_PORT_REG_CLBU) as *mut u32,
                (cmd_list_addr >> 32) as u32,
            );
            write_volatile(
                (port_base + AHCI_PORT_REG_FB) as *mut u32,
                (fis_addr & 0xFFFFFFFF) as u32,
            );
            write_volatile(
                (port_base + AHCI_PORT_REG_FBU) as *mut u32,
                (fis_addr >> 32) as u32,
            );
        }

        // Enable FIS reception and start port
        unsafe {
            let cmd = read_volatile((port_base + AHCI_PORT_REG_CMD) as *const u32);
            write_volatile(
                (port_base + AHCI_PORT_REG_CMD) as *mut u32,
                cmd | 0x00000010,
            ); // Set FRE
            write_volatile(
                (port_base + AHCI_PORT_REG_CMD) as *mut u32,
                cmd | 0x00000001,
            ); // Set ST
        }

        // Check if device is present
        let ssts = unsafe { read_volatile((port_base + AHCI_PORT_REG_SSTS) as *const u32) };
        if (ssts & 0x0F) == 0 {
            return Ok(()); // No device, continue
        }

        Ok(())
    }

    /// Helper to issue an ATA command to a specific port and slot
    fn issue_ata_cmd(
        &mut self,
        port: usize,
        slot: usize,
        lba: u64,
        count: u16,
        cmd: u8,
        buffer: *mut u8,
        write: bool,
    ) -> Result<(), ()> {
        let port_base = self.bar5 + 0x100 + (port as u64 * 0x80);

        // Wait for port to be idle
        let mut timeout = 1000000;
        unsafe {
            while (read_volatile((port_base + AHCI_PORT_REG_TFD) as *const u32) & (0x80 | 0x08))
                != 0
                && timeout > 0
            {
                timeout -= 1;
            }
            if timeout == 0 {
                return Err(());
            }
        }

        // Set up Command Table
        unsafe {
            let cmd_table = &mut CMD_TABLES[port][slot];
            let cmd_list = &mut CMD_LISTS[port].0[slot];

            // Clear command table
            core::ptr::write_bytes(
                cmd_table as *mut _ as *mut u8,
                0,
                core::mem::size_of::<AhciCommandTable>(),
            );

            // Fill Command FIS
            let cmd_fis = &mut cmd_table.command_fis as *mut _ as *mut AhciH2DFis;
            (*cmd_fis).fis_type = 0x27; // Host to Device
            (*cmd_fis).flags = 0x80; // Command (bit 7)
            (*cmd_fis).command = cmd;
            (*cmd_fis).device = 0x40; // LBA mode
            (*cmd_fis).lba0 = (lba & 0xFF) as u8;
            (*cmd_fis).lba1 = ((lba >> 8) & 0xFF) as u8;
            (*cmd_fis).lba2 = ((lba >> 16) & 0xFF) as u8;
            (*cmd_fis).lba3 = ((lba >> 24) & 0xFF) as u8;
            (*cmd_fis).lba4 = ((lba >> 32) & 0xFF) as u8;
            (*cmd_fis).lba5 = ((lba >> 40) & 0xFF) as u8;
            (*cmd_fis).count_low = (count & 0xFF) as u8;
            (*cmd_fis).count_high = ((count >> 8) & 0xFF) as u8;

            // Set up PRDT
            cmd_table.prdt[0].dba = buffer as u64;
            cmd_table.prdt[0].dbc = (count as u32 * 512) - 1; // 0-based count

            // Set up Command Header
            let mut flags = 0x0005; // Command FIS length (5 dwords)
            if write {
                flags |= 0x0040; // Write bit (bit 6)
            }
            cmd_list.flags = flags;
            cmd_list.prdtl = 1;
            cmd_list.ctba = cmd_table as *const _ as u64;

            // Issue command
            write_volatile((port_base + AHCI_PORT_REG_CI) as *mut u32, 1 << slot);

            // Wait for completion
            loop {
                let ci = read_volatile((port_base + AHCI_PORT_REG_CI) as *const u32);
                if (ci & (1 << slot)) == 0 {
                    break;
                }
                let tfd = read_volatile((port_base + AHCI_PORT_REG_TFD) as *const u32);
                if (tfd & 0x01) != 0 {
                    // Error bit
                    return Err(());
                }
            }

            // Check final status
            let tfd = read_volatile((port_base + AHCI_PORT_REG_TFD) as *const u32);
            if (tfd & 0x01) != 0 {
                return Err(());
            }
        }

        Ok(())
    }

    /// Read blocks from a port
    pub fn read(&mut self, port: usize, lba: u64, count: u16, buffer: *mut u8) -> Result<(), ()> {
        if !self.active_ports[port] {
            return Err(());
        }
        self.issue_ata_cmd(port, 0, lba, count, 0x25, buffer, false) // READ DMA EXT (LBA48)
    }

    /// Write blocks to a port
    pub fn write(
        &mut self,
        port: usize,
        lba: u64,
        count: u16,
        buffer: *const u8,
    ) -> Result<(), ()> {
        if !self.active_ports[port] {
            return Err(());
        }
        self.issue_ata_cmd(port, 0, lba, count, 0x35, buffer as *mut u8, true) // WRITE DMA EXT (LBA48)
    }

    /// Get list of active ports
    pub fn get_active_ports(&self) -> [bool; 32] {
        self.active_ports
    }
}

impl Default for AhciDriver {
    fn default() -> Self {
        Self::new()
    }
}

// Static AHCI driver instance
use spin::Mutex;
pub static AHCI_DRIVER: Mutex<AhciDriver> = Mutex::new(AhciDriver::new());

// Implement PciDriver for a wrapper type
pub struct AhciPciDriver;
pub static AHCI_PCI_DRIVER: AhciPciDriver = AhciPciDriver;

impl PciDriver for AhciPciDriver {
    fn probe(&self, dev: &PciDeviceInfo) -> Result<(), ()> {
        // Check if it's an AHCI controller (class 0x01, subclass 0x06, interface 0x01)
        if dev.class_code.base == 0x01
            && dev.class_code.sub == 0x06
            && dev.class_code.interface == 0x01
        {
            // Initialize driver
            let mut driver = AHCI_DRIVER.lock();
            driver.init(dev)?;
            Ok(())
        } else {
            Err(())
        }
    }

    fn get_supported_devices(&self) -> &'static [(u16, u16)] {
        // Support any AHCI controller (wildcard vendor/device)
        &[(0x0000, 0x0000)]
    }
}
