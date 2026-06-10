// XPARQ OS - x86_64 PS/2 Keyboard Driver
// Simple PS/2 keyboard driver for QEMU x86_64

use crate::input::{InputDriver, InputError, InputDeviceInfo, InputDeviceType, InputCapabilities, InputEvent, InputEventKind, InputEventData, Modifiers};
use core::ptr::write_volatile;
use core::ptr::read_volatile;

/// Port addresses for PS/2 controller
const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;
const PS2_COMMAND_PORT: u16 = 0x64;

/// PS/2 keyboard driver
pub struct Ps2Keyboard {
    initialized: bool,
    modifiers: Modifiers,
}

impl Ps2Keyboard {
    /// Create a new PS/2 keyboard driver
    pub fn new() -> Self {
        Self {
            initialized: false,
            modifiers: Modifiers::empty(),
        }
    }

    /// Read a byte from PS/2 data port
    fn read_data(&self) -> u8 {
        unsafe {
            while (read_volatile(PS2_STATUS_PORT as *const u8) & 0x01) == 0 {}
            read_volatile(PS2_DATA_PORT as *const u8)
        }
    }

    /// Write a byte to PS/2 data port
    fn write_data(&self, data: u8) {
        unsafe {
            while (read_volatile(PS2_STATUS_PORT as *const u8) & 0x02) != 0 {}
            write_volatile(PS2_DATA_PORT as *mut u8, data);
        }
    }

    /// Write a command to PS/2 command port
    fn write_command(&self, cmd: u8) {
        unsafe {
            while (read_volatile(PS2_STATUS_PORT as *const u8) & 0x02) != 0 {}
            write_volatile(PS2_COMMAND_PORT as *mut u8, cmd);
        }
    }
}

impl InputDriver for Ps2Keyboard {
    fn name(&self) -> &'static str {
        "PS/2 Keyboard"
    }

    fn init(&mut self) -> Result<(), InputError> {
        // Initialize PS/2 controller
        // Step 1: Disable first and second PS/2 ports
        self.write_command(0xAD); // Disable first port
        self.write_command(0xA7); // Disable second port

        // Step 2: Flush output buffer
        let _ = self.read_data();

        // Step 3: Set configuration byte
        self.write_command(0x20); // Read configuration byte
        let mut config = self.read_data();
        config &= !0x10; // Disable first port IRQ
        config &= !0x20; // Disable second port IRQ
        config &= !0x40; // Disable translation
        self.write_command(0x60); // Write configuration byte
        self.write_data(config);

        // Step 4: Perform controller self-test
        self.write_command(0xAA);
        if self.read_data() != 0x55 {
            return Err(InputError::HardwareFailure);
        }

        // Step 5: Enable first PS/2 port
        self.write_command(0xAE);

        // Step 6: Reset keyboard
        self.write_data(0xFF); // Reset
        let _ = self.read_data(); // Read ACK (0xFA)

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
        // Check if there's data available
        unsafe {
            if (read_volatile(PS2_STATUS_PORT as *const u8) & 0x01) == 0 {
                return None;
            }
        }

        let scancode = self.read_data();

        // Very basic scancode handling (for demonstration)
        let keycode = scancode & 0x7F;
        let pressed = (scancode & 0x80) == 0;

        // Update modifiers
        match keycode {
            0x1C => {
                // Left Shift
                if pressed {
                    self.modifiers |= Modifiers::SHIFT;
                } else {
                    self.modifiers &= !Modifiers::SHIFT;
                }
            }
            0x1D => {
                // Left Ctrl
                if pressed {
                    self.modifiers |= Modifiers::CTRL;
                } else {
                    self.modifiers &= !Modifiers::CTRL;
                }
            }
            0x38 => {
                // Left Alt
                if pressed {
                    self.modifiers |= Modifiers::ALT;
                } else {
                    self.modifiers &= !Modifiers::ALT;
                }
            }
            _ => {}
        }

        Some(InputEvent {
            timestamp: 0, // TODO: Implement proper timestamp
            device_type: InputDeviceType::Keyboard,
            event_kind: if pressed { InputEventKind::KeyDown } else { InputEventKind::KeyUp },
            data: InputEventData::Key {
                keycode: keycode as u32,
                scancode: scancode as u32,
                modifiers: self.modifiers,
            },
        })
    }

    fn set_event_callback(&mut self, _callback: Option<fn(&InputEvent)>) {
        // TODO: Implement callbacks
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
