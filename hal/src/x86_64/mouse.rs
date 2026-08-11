// XPARQ OS - x86_64 PS/2 Mouse Driver
// Simple PS/2 mouse driver for QEMU x86_64

use crate::input::{InputDriver, InputError, InputDeviceInfo, InputDeviceType, InputCapabilities, InputEvent, InputEventKind, InputEventData, MouseButtons};
use core::ptr::read_volatile;
use core::ptr::write_volatile;
use arrayvec::ArrayVec;
use spin::Mutex;

const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;
const PS2_COMMAND_PORT: u16 = 0x64;

/// Simple PS/2 mouse driver
pub struct Ps2Mouse {
    initialized: bool,
    packet: [u8; 3],
    packet_idx: usize,
    buttons: MouseButtons,
    event_queue: Mutex<ArrayVec<InputEvent, 64>>,
    callback: Option<fn(&InputEvent)>,
}

impl Ps2Mouse {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            packet: [0u8; 3],
            packet_idx: 0,
            buttons: MouseButtons::empty(),
            event_queue: Mutex::new(ArrayVec::new_const()),
            callback: None,
        }
    }

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

    fn wait_ack(&self) -> bool {
        let byte = self.read_data();
        byte == 0xFA // ACK
    }

    /// Interrupt handler for mouse
    pub fn irq_handler(&mut self) -> bool {
        unsafe {
            if (crate::hal_inb(PS2_STATUS_PORT) & 0x01) == 0 {
                return false;
            }
        }

        self.packet[self.packet_idx] = self.read_data();
        self.packet_idx += 1;

        if self.packet_idx == 3 {
            self.packet_idx = 0;
            let [flags, dx, dy] = self.packet;
            let dx = dx as i8 as i32;
            let dy = dy as i8 as i32;
            
            let previous_buttons = self.buttons;
            self.buttons = MouseButtons::empty();
            if flags & 0x01 != 0 { self.buttons |= MouseButtons::LEFT; }
            if flags & 0x02 != 0 { self.buttons |= MouseButtons::RIGHT; }
            if flags & 0x04 != 0 { self.buttons |= MouseButtons::MIDDLE; }
            let pressed = self.buttons.bits() & !previous_buttons.bits();
            let released = previous_buttons.bits() & !self.buttons.bits();
            let event_kind = if pressed != 0 {
                InputEventKind::MouseDown
            } else if released != 0 {
                InputEventKind::MouseUp
            } else {
                InputEventKind::MouseMove
            };

            let event = InputEvent {
                timestamp: 0,
                device_type: InputDeviceType::Mouse,
                event_kind,
                data: InputEventData::Mouse {
                    x: dx, y: -dy, // PS/2 mouse Y is inverted
                    buttons: self.buttons,
                    wheel_delta: 0
                }
            };

            let mut queue = self.event_queue.lock();
            let _ = queue.try_push(event.clone());
            drop(queue);

            if let Some(callback) = self.callback {
                callback(&event);
            }
            return true;
        }
        false
    }
}

impl InputDriver for Ps2Mouse {
    fn name(&self) -> &'static str {
        "PS/2 Mouse"
    }

    fn init(&mut self) -> Result<(), InputError> {
        self.write_command(0xA9); // Test second PS/2 port.
        if self.read_data() != 0x00 {
            super::trace_log(b"XPARQ_TEST:FAIL:PS2_MOUSE_PORT_TEST\n");
            return Err(InputError::HardwareFailure);
        }

        // Enable the second port while keeping its IRQ masked during handshake.
        self.write_command(0xA8);
        self.write_command(0x20);
        let mut config = self.read_data();
        config &= !0x02;
        config &= !0x20;
        self.write_command(0x60);
        self.write_data(config);

        // Reset the device and consume ACK, BAT completion, and device ID.
        self.write_command(0xD4);
        self.write_data(0xFF);
        if !self.wait_ack() || self.read_data() != 0xAA || self.read_data() != 0x00 {
            super::trace_log(b"XPARQ_TEST:FAIL:PS2_MOUSE_RESET\n");
            return Err(InputError::HardwareFailure);
        }

        self.write_command(0xD4);
        self.write_data(0xF4); // Enable packet streaming.
        if !self.wait_ack() {
            super::trace_log(b"XPARQ_TEST:FAIL:PS2_MOUSE_ENABLE\n");
            return Err(InputError::HardwareFailure);
        }

        // Unmask IRQ12 only after all mouse responses have been consumed.
        self.write_command(0x20);
        let mut config = self.read_data();
        config |= 0x02;
        config &= !0x20;
        self.write_command(0x60);
        self.write_data(config);

        self.initialized = true;
        Ok(())
    }

    fn get_info(&self) -> InputDeviceInfo {
        InputDeviceInfo {
            name: "PS/2 Mouse",
            device_type: InputDeviceType::Mouse,
            capabilities: InputCapabilities::default(),
            max_events: 256,
            supported_event_types: {
                let mut types = arrayvec::ArrayVec::new();
                types.push(InputEventKind::MouseMove);
                types.push(InputEventKind::MouseDown);
                types.push(InputEventKind::MouseUp);
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

    fn set_enabled(&mut self, _enabled: bool) -> Result<(), InputError> { Ok(()) }

    fn is_enabled(&self) -> bool { self.initialized }

    fn calibrate(&mut self) -> Result<(), InputError> { Ok(()) }

    fn get_calibration_status(&self) -> crate::input::CalibrationStatus {
        crate::input::CalibrationStatus::Calibrated
    }
}

impl Default for Ps2Mouse {
    fn default() -> Self { Self::new() }
}
