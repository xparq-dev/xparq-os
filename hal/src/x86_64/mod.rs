// XPARQ OS - x86_64 HAL
// Architecture-specific implementations for x86_64

use crate::HalError;
use crate::display::DisplayDriver;
use crate::input::InputDriver;
use spin::Mutex;

pub mod display;
pub mod keyboard;
pub mod mouse;
pub mod power;
pub mod storage;

use display::VgaTextDisplay;
use keyboard::Ps2Keyboard;
use mouse::Ps2Mouse;

/// Static VGA text display driver instance
pub static VGA_DISPLAY: Mutex<Option<VgaTextDisplay>> = Mutex::new(None);

/// Static PS/2 keyboard driver instance
pub static PS2_KEYBOARD: Mutex<Option<Ps2Keyboard>> = Mutex::new(None);

/// Static PS/2 mouse driver instance
pub static PS2_MOUSE: Mutex<Option<Ps2Mouse>> = Mutex::new(None);

pub fn init_arch_specific() -> Result<(), HalError> {
    println!("Initializing x86-64-specific HAL...");
    
    // Initialize VGA display
    let mut vga = VgaTextDisplay::new();
    vga.init()?;
    *VGA_DISPLAY.lock() = Some(vga);
    
    // Initialize PS/2 keyboard
    let mut keyboard = Ps2Keyboard::new();
    keyboard.init()?;
    *PS2_KEYBOARD.lock() = Some(keyboard);
    
    // Initialize PS/2 mouse
    let mut mouse = Ps2Mouse::new();
    mouse.init()?;
    *PS2_MOUSE.lock() = Some(mouse);
    
    println!("x86-64-specific HAL initialized");
    Ok(())
}
