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
pub mod usb;
use display::X86Display;
use keyboard::Ps2Keyboard;
use mouse::Ps2Mouse;
use storage::X86StorageDriver;
use power::X86PowerDriver;
use audio::X86AudioDriver;

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

pub fn init_arch_specific() -> Result<(), HalError> {
    // Initialize display
    let mut display = display::X86Display::new();
    display.init()?;
    *DISPLAY.lock() = Some(display);

    // Initialize IDT
    idt::init();

    // Initialize ACPI
    let _ = acpi::init();

    // Initialize APIC
    let _ = apic::init();

    // Initialize PCIe bus manager
    pci::init()?;

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

    Ok(())
}
