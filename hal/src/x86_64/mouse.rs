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
}

impl Ps2Mouse {
    pub fn new() -> Self {
        Self {
            initialized: false,
            packet: [0u8; 3],
            packet_idx: 0,
            buttons: MouseButtons::empty(),
            event_queue: Mutex::new(ArrayVec::new()),
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
    pub fn irq_handler(&mut self) {
        unsafe {
            if (crate::hal_inb(PS2_STATUS_PORT) & 0x01) == 0 {
                return;
            }
        }

        self.packet[self.packet_idx] = self.read_data();
        self.packet_idx += 1;

        if self.packet_idx == 3 {
            self.packet_idx = 0;
            let [flags, dx, dy] = self.packet;
            let dx = dx as i8 as i32;
            let dy = dy as i8 as i32;
            
            self.buttons = MouseButtons::empty();
            if flags & 0x01 != 0 { self.buttons |= MouseButtons::LEFT; }
            if flags & 0x02 != 0 { self.buttons |= MouseButtons::RIGHT; }
            if flags & 0x04 != 0 { self.buttons |= MouseButtons::MIDDLE; }

            let event = InputEvent {
                timestamp: 0,
                device_type: InputDeviceType::Mouse,
                event_kind: InputEventKind::MouseMove,
                data: InputEventData::Mouse {
                    x: dx, y: -dy, // PS/2 mouse Y is inverted
                    buttons: self.buttons,
                    wheel_delta: 0
                }
            };

            let mut queue = self.event_queue.lock();
            let _ = queue.try_push(event);
        }
    }
}

impl InputDriver for Ps2Mouse {
    fn name(&self) -> &'static str {
        "PS/2 Mouse"
    }

    fn init(&mut self) -> Result<(), InputError> {
        // Enable second PS/2 port
        self.write_command(0xA8);

        // Enable interrupts for second port
        self.write_command(0x20); // Read config byte
        let mut config = self.read_data();
        config |= 0x02; // Enable second port IRQ
        self.write_command(0x60);
        self.write_data(config);

        // Enable mouse packet streaming
        self.write_command(0xD4); // Send to second port
        self.write_data(0xF4); // Enable streaming
        if !self.wait_ack() {
            return Err(InputError::HardwareFailure);
        }

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

    fn set_event_callback(&mut self, _callback: Option<fn(&InputEvent)>) {}

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
