// XPARQ OS - x86_64 PS/2 Keyboard Driver
// Simple PS/2 keyboard driver for QEMU x86_64

use crate::input::{InputDriver, InputError, InputDeviceInfo, InputDeviceType, InputCapabilities, InputEvent, InputEventKind, InputEventData, Modifiers};
use core::ptr::write_volatile;
use core::ptr::read_volatile;
use arrayvec::ArrayVec;
use spin::Mutex;

/// Port addresses for PS/2 controller
const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;
const PS2_COMMAND_PORT: u16 = 0x64;

/// PS/2 keyboard driver
pub struct Ps2Keyboard {
    initialized: bool,
    modifiers: Modifiers,
    event_queue: spin::Mutex<arrayvec::ArrayVec<InputEvent, 64>>,
    callback: Option<fn(&InputEvent)>,
}

impl Ps2Keyboard {
    /// Create a new PS/2 keyboard driver
    pub const fn new() -> Self {
        Self {
            initialized: false,
            modifiers: Modifiers::empty(),
            event_queue: spin::Mutex::new(arrayvec::ArrayVec::new_const()),
            callback: None,
        }
    }

    /// Read a byte from PS/2 data port
    fn read_data(&self) -> u8 {
        unsafe {
            for _ in 0..100000 {
                if (crate::hal_inb(PS2_STATUS_PORT) & 0x01) != 0 {
                    return crate::hal_inb(PS2_DATA_PORT);
                }
                core::arch::asm!("pause");
            }
            0
        }
    }

    /// Write a byte to PS/2 data port
    fn write_data(&self, data: u8) {
        unsafe {
            for _ in 0..100000 {
                if (crate::hal_inb(PS2_STATUS_PORT) & 0x02) == 0 {
                    break;
                }
                core::arch::asm!("pause");
            }
            crate::hal_outb(PS2_DATA_PORT, data);
        }
    }

    /// Write a command to PS/2 command port
    fn write_command(&self, cmd: u8) {
        unsafe {
            for _ in 0..100000 {
                if (crate::hal_inb(PS2_STATUS_PORT) & 0x02) == 0 {
                    break;
                }
                core::arch::asm!("pause");
            }
            crate::hal_outb(PS2_COMMAND_PORT, cmd);
        }
    }

    /// Interrupt handler for keyboard
    pub fn irq_handler(&mut self) {
        unsafe {
            if (crate::hal_inb(PS2_STATUS_PORT) & 0x01) == 0 {
                return;
            }
        }

        let scancode = self.read_data();
        let keycode = scancode & 0x7F;
        let pressed = (scancode & 0x80) == 0;

        match keycode {
            0x2A => {
                if pressed { self.modifiers |= Modifiers::SHIFT; }
                else { self.modifiers &= !Modifiers::SHIFT; }
            }
            0x36 => {
                if pressed { self.modifiers |= Modifiers::SHIFT; }
                else { self.modifiers &= !Modifiers::SHIFT; }
            }
            0x1D => {
                if pressed { self.modifiers |= Modifiers::CTRL; }
                else { self.modifiers &= !Modifiers::CTRL; }
            }
            0x38 => {
                if pressed { self.modifiers |= Modifiers::ALT; }
                else { self.modifiers &= !Modifiers::ALT; }
            }
            0x3A => {
                if pressed { self.modifiers ^= Modifiers::CAPS_LOCK; }
            }
            _ => {}
        }

        let event = InputEvent {
            timestamp: 0,
            device_type: InputDeviceType::Keyboard,
            event_kind: if pressed { InputEventKind::KeyDown } else { InputEventKind::KeyUp },
            data: InputEventData::Key {
                keycode: keycode as u32,
                scancode: scancode as u32,
                modifiers: self.modifiers,
            },
        };

        let mut queue = self.event_queue.lock();
        let _ = queue.try_push(event.clone());
        drop(queue);
        
        if let Some(cb) = self.callback {
            cb(&event);
        }
    }
}

impl InputDriver for Ps2Keyboard {
    fn name(&self) -> &'static str {
        "PS/2 Keyboard"
    }

    fn init(&mut self) -> Result<(), InputError> {
        // Initialize PS/2 controller
        self.write_command(0xAD);
        self.write_command(0xA7);
        let _ = self.read_data();
        self.write_command(0x20);
        let mut config = self.read_data();
        config |= 0x01; // Enable first port IRQ
        config &= !0x20;
        config |= 0x40; // Enable first port translation
        self.write_command(0x60);
        self.write_data(config);
        self.write_command(0xAA);
        if self.read_data() != 0x55 { return Err(InputError::HardwareFailure); }
        self.write_command(0xAE);
        self.write_data(0xFF);
        let _ = self.read_data();

        self.initialized = true;
        Ok(())
    }

    fn get_info(&self) -> InputDeviceInfo {
        InputDeviceInfo {
            name: "PS/2 Keyboard",
            device_type: InputDeviceType::Keyboard,
            capabilities: InputCapabilities::default(),
            max_events: 256,
            supported_event_types: {
                let mut types = arrayvec::ArrayVec::new();
                types.push(InputEventKind::KeyDown);
                types.push(InputEventKind::KeyUp);
                types.push(InputEventKind::KeyRepeat);
                types
            },
        }
    }

    fn get_event(&mut self) -> Option<InputEvent> {
        self.event_queue.lock().pop_at(0)
    }

    fn set_event_callback(&mut self, callback: Option<fn(&InputEvent)>) {
        self.callback = callback;
    }

    fn set_enabled(&mut self, _enabled: bool) -> Result<(), InputError> {
        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.initialized
    }

    fn calibrate(&mut self) -> Result<(), InputError> {
        Ok(())
    }

    fn get_calibration_status(&self) -> crate::input::CalibrationStatus {
        crate::input::CalibrationStatus::Calibrated
    }
}

impl Default for Ps2Keyboard {
    fn default() -> Self {
        Self::new()
    }
}
