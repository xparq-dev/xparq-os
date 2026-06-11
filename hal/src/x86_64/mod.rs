// XPARQ OS - x86-64 HAL
// Architecture-specific implementations for x86-64
use crate::HalError;
use crate::display::DisplayDriver;
use crate::input::InputDriver;
use spin::Mutex;
pub mod display;
pub mod keyboard;
pub mod mouse;
pub mod power;
pub mod storage;
pub mod pci;
pub mod audio;
pub mod acpi;
pub mod apic;
pub mod idt;
pub mod pic;
pub mod usb;
pub mod ahci;
pub mod nvme;
use display::X86Display;
use keyboard::Ps2Keyboard;
use mouse::Ps2Mouse;
use storage::X86StorageDriver;
use power::X86PowerDriver;
use audio::X86AudioDriver;
use crate::x86_64::apic::timer_handler;
use crate::x86_64::ahci::AHCI_PCI_DRIVER;
use crate::x86_64::nvme::NVME_PCI_DRIVER;
use crate::x86_64::usb::XHCI_PCI_DRIVER;
use crate::x86_64::storage::ata_irq_handler;

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
extern "C" fn keyboard_irq_handler() {
    if let Some(mut keyboard) = PS2_KEYBOARD.lock().as_mut() {
        keyboard.irq_handler();
    }
}

/// Mouse IRQ handler wrapper
extern "C" fn mouse_irq_handler() {
    if let Some(mut mouse) = PS2_MOUSE.lock().as_mut() {
        mouse.irq_handler();
    }
}

pub fn init_arch_specific() -> Result<(), HalError> {
    // Initialize display
    let mut display = display::X86Display::new();
    display.init()?;
    *DISPLAY.lock() = Some(display);

    // Initialize IDT
    idt::init();

    // Initialize ACPI
    let _ = acpi::init();

    // Disable legacy 8259 PIC before initializing APIC
    pic::disable_pic();

    // Initialize APIC
    let _ = apic::init();
    let _ = apic::init_interrupt_routing();

    // Initialize LAPIC timer
    unsafe {
        if let Some(lapic) = &apic::LOCAL_APIC {
            lapic.init_timer();
        }
    }

    // Initialize PCIe bus manager
    pci::init()?;

    // Register and bind PCI drivers
    pci::register_driver(&AHCI_PCI_DRIVER);
    pci::register_driver(&NVME_PCI_DRIVER);
    pci::register_driver(&XHCI_PCI_DRIVER);
    pci::bind_drivers();

    // Initialize storage driver
    let mut storage = storage::X86StorageDriver::new();
    storage.init()?;
    *STORAGE.lock() = Some(storage);

    // Initialize power driver
    let mut power = power::X86PowerDriver::new();
    power.init()?;
    *POWER.lock() = Some(power);

    // Initialize audio driver
    let mut audio = audio::X86AudioDriver::new();
    audio.init()?;
    *AUDIO.lock() = Some(audio);

    // Initialize PS/2 keyboard and mouse
    let mut keyboard = keyboard::Ps2Keyboard::new();
    keyboard.init()?;
    *PS2_KEYBOARD.lock() = Some(keyboard);

    let mut mouse = mouse::Ps2Mouse::new();
    mouse.init()?;
    *PS2_MOUSE.lock() = Some(mouse);

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
    unsafe { core::arch::asm!("sti"); }

    Ok(())
}
