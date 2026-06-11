// XPARQ OS - x86-64 USB Host Controller
// USB skeleton driver with xHCI support

use crate::x86_64::pci::{PciDeviceInfo, PciDriver};
use core::ptr::{read_volatile, write_volatile};

/// xHCI Capability Registers (CAPLENGTH to HCCPARAMS2)
#[repr(C, align(4))]
pub struct XhciCapRegs {
    pub caplength: u8,
    pub reserved: u8,
    pub hciversion: u16,
    pub hcsparams1: u32,
    pub hcsparams2: u32,
    pub hcsparams3: u32,
    pub hccparams1: u32,
    pub dboffset: u32,
    pub rtsoffset: u32,
    pub hccparams2: u32,
}

/// xHCI Operational Registers
#[repr(C, align(4))]
pub struct XhciOpRegs {
    pub usbcmd: u32,
    pub usbsts: u32,
    pub pagesize: u32,
    pub reserved1: [u32; 2],
    pub dnctrl: u32,
    pub crcr: u64,
    pub reserved2: [u32; 4],
    pub dcbaap: u64,
    pub config: u32,
    // Port registers are variable (up to MAX_PORTS)
}

/// xHCI Runtime Registers
#[repr(C, align(4))]
pub struct XhciRtRegs {
    pub mfindex: u32,
    pub reserved: [u32; 7],
    // Interrupter registers are variable (up to MAX_INTRS)
}

/// xHCI Doorbell Array
#[repr(C, align(4))]
pub struct XhciDoorbell {
    pub doorbell: u32,
}

/// xHCI Port Status and Control Register
#[repr(C, align(4))]
pub struct XhciPortRegs {
    pub portsc: u32,
    pub portpmsc: u32,
    pub portli: u32,
    pub reserved: u32,
}

/// xHCI Host Controller Driver
pub struct XhciController {
    pub bar0: u64,
    pub cap_regs: *const XhciCapRegs,
    pub op_regs: *mut XhciOpRegs,
    pub rt_regs: *const XhciRtRegs,
    pub doorbells: *const XhciDoorbell,
    pub num_ports: u8,
    pub initialized: bool,
}

impl XhciController {
    pub const fn new() -> Self {
        Self {
            bar0: 0,
            cap_regs: core::ptr::null(),
            op_regs: core::ptr::null_mut(),
            rt_regs: core::ptr::null(),
            doorbells: core::ptr::null(),
            num_ports: 0,
            initialized: false,
        }
    }

    pub fn init(&mut self, dev: &PciDeviceInfo) -> Result<(), ()> {
        // Find BAR0 (MMIO)
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

        // Calculate offsets from capability registers
        let cap_len = unsafe { read_volatile((self.bar0 as *const u8).offset(0)) };
        self.cap_regs = self.bar0 as *const XhciCapRegs;
        self.op_regs = (self.bar0 + cap_len as u64) as *mut XhciOpRegs;

        // Get number of ports (HCSPARAMS1 bits 24-31)
        let hcsp = unsafe { read_volatile(&(*self.cap_regs).hcsparams1) };
        self.num_ports = (hcsp >> 24) as u8;

        self.initialized = true;
        Ok(())
    }
}

/// Legacy x86 USB Host Controller (still available)
pub struct X86UsbHost {
    initialized: bool,
}

impl X86UsbHost {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    pub fn init(&mut self) -> Result<(), ()> {
        // TODO: Implement USB host controller initialization
        self.initialized = true;
        Ok(())
    }
}

impl Default for X86UsbHost {
    fn default() -> Self {
        Self::new()
    }
}

// PCI Driver for xHCI Controllers
struct XhciPciDriver;
pub static XHCI_PCI_DRIVER: XhciPciDriver = XhciPciDriver;

impl PciDriver for XhciPciDriver {
    fn probe(&self, dev: &PciDeviceInfo) -> Result<(), ()> {
        // Check for xHCI (Class 0x0c, Subclass 0x03, Interface 0x30)
        if dev.class_code.base == 0x0c && dev.class_code.sub == 0x03 && dev.class_code.interface == 0x30 {
            let mut controller = XHCI_CONTROLLER.lock();
            controller.init(dev)?;
            Ok(())
        } else {
            Err(())
        }
    }

    fn get_supported_devices(&self) -> &'static [(u16, u16)] {
        // Wildcard for any xHCI controller
        &[(0x0000, 0x0000)]
    }
}

// Static xHCI controller instance
use spin::Mutex;
pub static XHCI_CONTROLLER: Mutex<XhciController> = Mutex::new(XhciController::new());
