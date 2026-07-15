// XPARQ OS - x86-64 HAL
// Architecture-specific implementations for x86-64
use crate::display::DisplayDriver;
use crate::input::InputDriver;
use crate::storage::StorageDriver;
use crate::HalError;
use crate::power::PowerDriver;
use crate::audio::AudioDriver;
use spin::Mutex;
pub mod acpi;
pub mod ahci;
pub mod apic;
pub mod audio;
pub mod display;
pub mod idt;
pub mod keyboard;
pub mod mouse;
pub mod nvme;
pub mod pci;
pub mod pic;
pub mod power;
pub mod storage;
pub mod usb;
pub mod e1000;
pub mod syscall;
pub mod paging;
pub mod gdt;
use crate::x86_64::ahci::AHCI_PCI_DRIVER;
use crate::x86_64::apic::{sleep_ms, timer_handler};
use crate::x86_64::nvme::NVME_PCI_DRIVER;
use crate::x86_64::storage::ata_irq_handler;
use crate::x86_64::usb::XHCI_PCI_DRIVER;
use crate::x86_64::e1000::E1000_PCI_DRIVER;
use audio::X86AudioDriver;
use core::fmt::Write;
use display::X86Display;
use keyboard::Ps2Keyboard;
use mouse::Ps2Mouse;
use power::X86PowerDriver;
use storage::X86StorageDriver;

/// Static display driver instance
pub static DISPLAY: Mutex<Option<X86Display>> = Mutex::new(None);

/// Static PS/2 keyboard driver instance
pub static PS2_KEYBOARD: Mutex<Option<Ps2Keyboard>> = Mutex::new(None);

/// Static PS/2 mouse driver instance
pub static PS2_MOUSE: Mutex<Option<Ps2Mouse>> = Mutex::new(None);

/// Static storage driver instance
pub static STORAGE: Mutex<Option<X86StorageDriver>> = Mutex::new(None);

/// Static power driver instance
pub static POWER: Mutex<Option<X86PowerDriver>> = Mutex::new(None);

/// Static audio driver instance
pub static AUDIO: Mutex<Option<X86AudioDriver>> = Mutex::new(None);

/// Keyboard IRQ handler wrapper
fn keyboard_irq_handler() {
    trace_log(b"KBD_IRQ ");
    if let Some(keyboard) = PS2_KEYBOARD.lock().as_mut() {
        keyboard.irq_handler();
    }
}

/// Mouse IRQ handler wrapper
fn mouse_irq_handler() {
    if let Some(mouse) = PS2_MOUSE.lock().as_mut() {
        mouse.irq_handler();
    }
}

fn trace_log(msg: &[u8]) {
    unsafe {
        for &b in msg {
            core::arch::asm!("out dx, al", in("dx") 0x03F8u16, in("al") b);
        }
    }
}

pub fn init_arch_specific() -> Result<(), HalError> {
    // Initialize GDT and TSS
    gdt::init();
    trace_log(b"  -> GDT init done\n");

    // Initialize display
    let mut display = display::X86Display::new();
    display.init()?;
    *DISPLAY.lock() = Some(display);
    trace_log(b"  -> Display init done\n");

    // Initialize IDT
    idt::init();
    trace_log(b"  -> IDT init done\n");

    // Initialize ACPI
    let _ = acpi::init();
    trace_log(b"  -> ACPI init done\n");

    // Disable legacy 8259 PIC before initializing APIC
    pic::disable_pic();
    trace_log(b"  -> PIC disabled\n");

    // Initialize APIC
    let _ = apic::init();
    trace_log(b"  -> APIC init done\n");
    let _ = apic::init_interrupt_routing();
    trace_log(b"  -> APIC routing done\n");

    // Initialize LAPIC timer
    unsafe {
        if let Some(lapic) = (*(&raw const apic::LOCAL_APIC)).as_ref() {
            lapic.init_timer();
        }
    }
    trace_log(b"  -> LAPIC timer done\n");

    // Initialize PCIe bus manager
    pci::init()?;
    trace_log(b"  -> PCI init done\n");

    // Register and bind PCI drivers
    pci::register_driver(&AHCI_PCI_DRIVER);
    pci::register_driver(&NVME_PCI_DRIVER);
    pci::register_driver(&XHCI_PCI_DRIVER);
    pci::register_driver(&E1000_PCI_DRIVER);
    pci::bind_drivers();
    trace_log(b"  -> PCI drivers bound\n");

    // Initialize storage driver
    let mut storage = storage::X86StorageDriver::new();
    storage.init()?;
    *STORAGE.lock() = Some(storage);
    trace_log(b"  -> Storage init done\n");

    // Initialize power driver
    let mut power = power::X86PowerDriver::new();
    power.init()?;
    *POWER.lock() = Some(power);
    trace_log(b"  -> Power init done\n");

    // Initialize audio driver
    let mut audio = audio::X86AudioDriver::new();
    audio.init()?;
    *AUDIO.lock() = Some(audio);
    trace_log(b"  -> Audio init done\n");

    // Initialize PS/2 keyboard and mouse
    let mut keyboard = keyboard::Ps2Keyboard::new();
    keyboard.init()?;
    *PS2_KEYBOARD.lock() = Some(keyboard);
    trace_log(b"  -> Keyboard init done\n");

    let mut mouse = mouse::Ps2Mouse::new();
    let _ = mouse.init();
    *PS2_MOUSE.lock() = Some(mouse);
    trace_log(b"  -> Mouse init done\n");

    // Register IRQ handlers
    idt::register_irq_handler(32, timer_handler);
    idt::register_irq_handler(33, keyboard_irq_handler);
    idt::register_irq_handler(44, mouse_irq_handler);
    idt::register_irq_handler(46, ata_irq_handler);

    // Map IRQs (IRQ 1 to vector 33, IRQ 12 to vector 44, IRQ 14 to vector 46)
    let _ = apic::map_irq(1, 33);
    let _ = apic::map_irq(12, 44);
    let _ = apic::map_irq(14, 46);

    // Enable interrupts
    unsafe {
        core::arch::asm!("sti");
    }

    // Initialize connectivity subsystem
    crate::connectivity::init()?;
    trace_log(b"  -> Connectivity init done\n");

    // Register connected devices from our drivers
    {
        let e1000_driver = crate::x86_64::e1000::E1000_DRIVER.lock();
        use crate::connectivity::ConnectivityDriver;
        if e1000_driver.is_connected() {
            crate::connectivity::CONNECTIVITY_MANAGER.lock()
                .register_device(e1000_driver.name(), e1000_driver.get_info());
        }
    }
    trace_log(b"  -> Drivers registered\n");

    // --- Demo of New Capabilities ---
    let display_opt = DISPLAY.lock().take();
    if let Some(mut display) = display_opt {
        display.clear_screen(0);
        trace_log(b"  -> Display cleared\n");


        if let Some(storage) = STORAGE.lock().as_ref() {
            writeln!(&mut display, "Storage: Initialized").unwrap();
            let devices = storage.get_devices();
            for dev in devices {
                writeln!(
                    &mut display,
                    "  [{}] {} ({:?})",
                    dev.id, dev.name, dev.interface
                )
                .unwrap();
            }
        }
        trace_log(b"  -> Storage demo done\n");

        let conn_mgr = crate::connectivity::CONNECTIVITY_MANAGER.lock();
        let net_devices = conn_mgr.get_devices();
        if !net_devices.is_empty() {
            writeln!(&mut display, "Network: Initialized").unwrap();
            for dev in net_devices {
                let mac = dev.info.mac_address;
                writeln!(
                    &mut display,
                    "  [{}] {} MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    dev.id, dev.driver_name, mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                )
                .unwrap();
            }
        }

        writeln!(&mut display, "\nSystem ready.").unwrap();

        *DISPLAY.lock() = Some(display);
    }

    Ok(())
}
