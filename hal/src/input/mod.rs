// XPARQ OS - Phase 01: OS & Kernel Foundations
// HAL Input module - Phase 3: Hardware Abstraction Layer
// Provides unified input interface across ARM and x86 architectures

use bitflags::bitflags;
use arrayvec::ArrayVec;
use libm;

/// Input driver trait
pub trait InputDriver {
    /// Get driver name
    fn name(&self) -> &'static str;
    
    /// Initialize input driver
    fn init(&mut self) -> Result<(), InputError>;
    
    /// Get input device information
    fn get_info(&self) -> InputDeviceInfo;
    
    /// Get next input event (non-blocking)
    fn get_event(&mut self) -> Option<InputEvent>;
    
    /// Set event callback
    fn set_event_callback(&mut self, callback: Option<InputEventCallback>);
    
    /// Enable/disable device
    fn set_enabled(&mut self, enabled: bool) -> Result<(), InputError>;
    
    /// Check if device is enabled
    fn is_enabled(&self) -> bool;
    
    /// Calibrate device
    fn calibrate(&mut self) -> Result<(), InputError>;
    
    /// Get calibration status
    fn get_calibration_status(&self) -> CalibrationStatus;
}

/// Input event callback type
pub type InputEventCallback = fn(&InputEvent);

/// Input device information
#[derive(Debug, Clone)]
pub struct InputDeviceInfo {
    pub name: &'static str,
    pub device_type: InputDeviceType,
    pub capabilities: InputCapabilities,
    pub max_events: usize,
    pub supported_event_types: ArrayVec<InputEventKind, 16>,
}

/// Input device types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputDeviceType {
    Keyboard,
    Mouse,
    Touchscreen,
    Stylus,
    Gamepad,
    Joystick,
    Trackpad,
    Button,
    Switch,
    Sensor,
}

/// Input capabilities
#[derive(Debug, Clone, Copy)]
pub struct InputCapabilities {
    pub multi_touch: bool,
    pub pressure_sensitivity: bool,
    pub gesture_recognition: bool,
    pub haptic_feedback: bool,
    pub wireless: bool,
    pub programmable: bool,
}

/// Input event
#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    pub timestamp: u64,
    pub device_type: InputDeviceType,
    pub event_kind: InputEventKind,
    pub data: InputEventData,
}

/// Input event kinds
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEventKind {
    KeyDown,
    KeyUp,
    KeyRepeat,
    MouseMove,
    MouseDown,
    MouseUp,
    MouseWheel,
    TouchDown,
    TouchUp,
    TouchMove,
    StylusDown,
    StylusUp,
    StylusMove,
    ButtonPress,
    ButtonRelease,
    SwitchToggle,
    GestureStart,
    GestureEnd,
    SensorData,
}

/// Input event data
#[derive(Debug, Clone, Copy)]
pub enum InputEventData {
    Key { keycode: u32, scancode: u32, modifiers: Modifiers },
    Mouse { x: i32, y: i32, buttons: MouseButtons, wheel_delta: i8 },
    Touch { id: u32, x: i32, y: i32, pressure: u8, major: u16, minor: u16 },
    Stylus { x: i32, y: i32, pressure: u16, tilt_x: i8, tilt_y: i8, buttons: StylusButtons },
    Button { button_id: u32, state: ButtonState },
    Switch { switch_id: u32, state: SwitchState },
    Gesture { gesture_type: GestureType, data: GestureData },
    Sensor { sensor_type: SensorType, data: SensorData },
}

/// Keyboard modifiers
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Modifiers: u32 {
        const SHIFT = 0x0001;
        const CTRL = 0x0002;
        const ALT = 0x0004;
        const SUPER = 0x0008;
        const CAPS_LOCK = 0x0010;
        const NUM_LOCK = 0x0020;
        const SCROLL_LOCK = 0x0040;
    }
}

/// Mouse buttons
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MouseButtons: u32 {
        const LEFT = 0x0001;
        const RIGHT = 0x0002;
        const MIDDLE = 0x0004;
        const BACK = 0x0008;
        const FORWARD = 0x0010;
    }
}

/// Stylus buttons
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct StylusButtons: u32 {
        const TIP = 0x0001;
        const BARREL = 0x0002;
        const ERASER = 0x0004;
    }
}

/// Button states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Pressed,
    Released,
    Held,
}

/// Switch states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwitchState {
    On,
    Off,
    Toggling,
}

/// Gesture types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GestureType {
    Tap,
    DoubleTap,
    LongPress,
    Swipe,
    Pinch,
    Rotate,
    Pan,
    Custom { gesture_id: u32 },
}

/// Gesture data
#[derive(Debug, Clone, Copy)]
pub struct GestureData {
    pub x: i32,
    pub y: i32,
    pub parameters: [f32; 4], // Gesture-specific parameters
}

/// Sensor types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorType {
    Accelerometer,
    Gyroscope,
    Magnetometer,
    Light,
    Proximity,
    Temperature,
    Humidity,
    Pressure,
}

/// Sensor data
#[derive(Debug, Clone, Copy)]
pub struct SensorData {
    pub values: [f32; 3], // X, Y, Z (or other sensor-specific values)
    pub accuracy: SensorAccuracy,
    pub timestamp: u64,
}

/// Sensor accuracy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorAccuracy {
    Unreliable,
    Low,
    Medium,
    High,
}

/// Calibration status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalibrationStatus {
    NotCalibrated,
    Calibrating,
    Calibrated,
    Failed,
}

/// Input errors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputError {
    /// Device not found
    DeviceNotFound,
    /// Unsupported operation
    Unsupported,
    /// Invalid parameter
    InvalidParameter,
    /// Hardware failure
    HardwareFailure,
    /// Timeout
    Timeout,
    /// Permission denied
    PermissionDenied,
    /// Calibration failed
    CalibrationFailed,
    /// Buffer overflow
    BufferOverflow,
}

/// Input manager
pub struct InputManager {
    /// Registered input drivers - simplified for no_std
    drivers: ArrayVec<*const (), 16>,
    /// Active input devices
    devices: ArrayVec<InputDeviceHandle, 32>,
    /// Event queue
    event_queue: ArrayVec<InputEvent, 256>,
    /// Next device ID
    next_id: u32,
}

/// Input device handle
#[derive(Debug, Clone)]
pub struct InputDeviceHandle {
    pub id: u32,
    pub driver_name: &'static str,
    pub info: InputDeviceInfo,
    pub enabled: bool,
    pub calibration_status: CalibrationStatus,
}

impl InputManager {
    /// Create new input manager
    pub fn new() -> Self {
        Self {
            drivers: ArrayVec::new(),
            devices: ArrayVec::new(),
            event_queue: ArrayVec::new(),
            next_id: 1,
        }
    }
    
    /// Register input driver - simplified for no_std
    pub fn register_driver(&mut self, _driver: *const ()) -> Result<(), InputError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Initialize all drivers - simplified for no_std
    pub fn init_all(&mut self) -> Result<(), InputError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Get device by ID
    pub fn get_device(&self, id: u32) -> Option<&InputDeviceHandle> {
        self.devices.iter().find(|device| device.id == id)
    }
    
    /// Get device by name
    pub fn get_device_by_name(&self, name: &str) -> Option<&InputDeviceHandle> {
        self.devices.iter().find(|device| device.info.name == name)
    }
    
    /// Enable/disable device - simplified for no_std
    pub fn set_enabled(&mut self, _id: u32, _enabled: bool) -> Result<(), InputError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Calibrate device - simplified for no_std
    pub fn calibrate_device(&mut self, _id: u32) -> Result<(), InputError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Collect events from all devices - simplified for no_std
    pub fn collect_events(&mut self) -> Result<(), InputError> {
        // Phase 1: Dummy implementation - no dynamic dispatch in no_std
        // Phase 2: Use trait objects without heap allocation
        Ok(())
    }
    
    /// Get next event (non-blocking)
    pub fn get_event(&mut self) -> Option<InputEvent> {
        self.collect_events().ok();
        self.event_queue.pop_at(0)
    }
    
    /// Get next event (blocking with timeout)
    pub fn get_event_timeout(&mut self, timeout_ms: u32) -> Option<InputEvent> {
        // Phase 1: Simple polling
        // Phase 2: Use proper timeout mechanism
        
        let start_time = get_timestamp();
        
        loop {
            if let Some(event) = self.get_event() {
                return Some(event);
            }
            
            let current_time = get_timestamp();
            if current_time >= start_time + timeout_ms as u64 {
                return None;
            }
            
            // Small delay to avoid busy-waiting
            delay_ms(1);
        }
    }
    
    /// List all devices
    pub fn list_devices(&self) -> ArrayVec<&InputDeviceHandle, 32> {
        self.devices.iter().collect()
    }
    
    /// Get devices by type
    pub fn get_devices_by_type(&self, device_type: InputDeviceType) -> ArrayVec<&InputDeviceHandle, 32> {
        self.devices.iter()
            .filter(|device| device.info.device_type == device_type)
            .collect()
    }
}

/// Global input manager
static mut INPUT_MANAGER: Option<InputManager> = None;
static mut INPUT_MANAGER_INITIALIZED: bool = false;

/// Initialize input subsystem
pub fn init() -> Result<(), super::HalError> {
    unsafe {
        if INPUT_MANAGER_INITIALIZED {
            return Ok(());
        }
        
        INPUT_MANAGER = Some(InputManager::new());
        INPUT_MANAGER_INITIALIZED = true;
        
        // Initialize architecture-specific input drivers
        #[cfg(target_arch = "x86_64")]
        {
            // Phase 1: Real PS/2 keyboard/mouse drivers
            let mut _keyboard = crate::x86_64::keyboard::Ps2Keyboard::new();
            _ = _keyboard.init();
            
            let mut _mouse = crate::x86_64::mouse::Ps2Mouse::new();
            _ = _mouse.init();
        }
        
        if let Some(manager) = &mut INPUT_MANAGER {
            manager.init_all()?;
            // manager.enumerate_devices()?; // Simplified for no_std
        }
    }
    
    Ok(())
}

/// Get global input manager
pub fn get_input_manager() -> Option<&'static InputManager> {
    unsafe { INPUT_MANAGER.as_ref() }
}

/// Get mutable global input manager
pub fn get_input_manager_mut() -> Option<&'static mut InputManager> {
    unsafe { INPUT_MANAGER.as_mut() }
}

/// Get timestamp in milliseconds
fn get_timestamp() -> u64 {
    // Phase 1: Dummy timestamp
    // Phase 2: Use actual timer
    0
}

/// Simple delay
fn delay_ms(_ms: u32) {
    // Phase 1: Dummy delay
    // Phase 2: Use proper delay mechanism
}

/// Input utilities
pub mod utils {
    use super::*;
    
    /// Convert keycode to character (PS/2 scancodes, set 1)
    pub fn keycode_to_char(keycode: u32, modifiers: Modifiers) -> Option<char> {
        // Phase 1: Basic US keyboard layout
        // Phase 2: Full international keyboard support
        
        let shift = modifiers.contains(Modifiers::SHIFT);
        
        match keycode as u8 {
            0x02 => Some(if shift { '!' } else { '1' }),
            0x03 => Some(if shift { '@' } else { '2' }),
            0x04 => Some(if shift { '#' } else { '3' }),
            0x05 => Some(if shift { '$' } else { '4' }),
            0x06 => Some(if shift { '%' } else { '5' }),
            0x07 => Some(if shift { '^' } else { '6' }),
            0x08 => Some(if shift { '&' } else { '7' }),
            0x09 => Some(if shift { '*' } else { '8' }),
            0x0A => Some(if shift { '(' } else { '9' }),
            0x0B => Some(if shift { ')' } else { '0' }),
            0x0C => Some(if shift { '_' } else { '-' }),
            0x0D => Some(if shift { '+' } else { '=' }),
            0x10 => Some(if shift { 'Q' } else { 'q' }),
            0x11 => Some(if shift { 'W' } else { 'w' }),
            0x12 => Some(if shift { 'E' } else { 'e' }),
            0x13 => Some(if shift { 'R' } else { 'r' }),
            0x14 => Some(if shift { 'T' } else { 't' }),
            0x15 => Some(if shift { 'Y' } else { 'y' }),
            0x16 => Some(if shift { 'U' } else { 'u' }),
            0x17 => Some(if shift { 'I' } else { 'i' }),
            0x18 => Some(if shift { 'O' } else { 'o' }),
            0x19 => Some(if shift { 'P' } else { 'p' }),
            0x1A => Some(if shift { '{' } else { '[' }),
            0x1B => Some(if shift { '}' } else { ']' }),
            0x1E => Some(if shift { 'A' } else { 'a' }),
            0x1F => Some(if shift { 'S' } else { 's' }),
            0x20 => Some(if shift { 'D' } else { 'd' }),
            0x21 => Some(if shift { 'F' } else { 'f' }),
            0x22 => Some(if shift { 'G' } else { 'g' }),
            0x23 => Some(if shift { 'H' } else { 'h' }),
            0x24 => Some(if shift { 'J' } else { 'j' }),
            0x25 => Some(if shift { 'K' } else { 'k' }),
            0x26 => Some(if shift { 'L' } else { 'l' }),
            0x27 => Some(if shift { ':' } else { ';' }),
            0x28 => Some(if shift { '"' } else { '\'' }),
            0x29 => Some(if shift { '~' } else { '`' }),
            0x2B => Some(if shift { '|' } else { '\\' }),
            0x2C => Some(if shift { 'Z' } else { 'z' }),
            0x2D => Some(if shift { 'X' } else { 'x' }),
            0x2E => Some(if shift { 'C' } else { 'c' }),
            0x2F => Some(if shift { 'V' } else { 'v' }),
            0x30 => Some(if shift { 'B' } else { 'b' }),
            0x31 => Some(if shift { 'N' } else { 'n' }),
            0x32 => Some(if shift { 'M' } else { 'm' }),
            0x33 => Some(if shift { '<' } else { ',' }),
            0x34 => Some(if shift { '>' } else { '.' }),
            0x35 => Some(if shift { '?' } else { '/' }),
            0x39 => Some(' '),
            _ => None,
        }
    }
    
    /// Calculate touch distance
    pub fn calculate_touch_distance(touch1: &InputEventData, touch2: &InputEventData) -> f32 {
        if let (InputEventData::Touch { x: x1, y: y1, .. }, InputEventData::Touch { x: x2, y: y2, .. }) = (touch1, touch2) {
            let dx = (x2 - x1) as f32;
            let dy = (y2 - y1) as f32;
            libm::sqrtf(dx * dx + dy * dy)
        } else {
            0.0
        }
    }
    
    /// Detect gesture from touch sequence
    pub fn detect_gesture(touches: &[InputEvent]) -> Option<GestureType> {
        // Phase 1: Basic gesture detection
        // Phase 2: Advanced gesture recognition
        
        if touches.len() < 2 {
            return None;
        }
        
        let first = &touches[0];
        let last = &touches[touches.len() - 1];
        
        // Detect swipe
        if let (InputEventData::Touch { x: x1, y: y1, .. }, InputEventData::Touch { x: x2, y: y2, .. }) = (&first.data, &last.data) {
            let dx = x2 - x1;
            let dy = y2 - y1;
            
            if dx.abs() > 100 && dy.abs() < 50 {
                return Some(GestureType::Swipe);
            }
            
            if dy.abs() > 100 && dx.abs() < 50 {
                return Some(GestureType::Swipe);
            }
        }
        
        // Detect tap (short duration, small movement)
        let duration = last.timestamp - first.timestamp;
        if duration < 500 {
            if let (InputEventData::Touch { x: x1, y: y1, .. }, InputEventData::Touch { x: x2, y: y2, .. }) = (&first.data, &last.data) {
                let dx = (x2 - x1).abs();
                let dy = (y2 - y1).abs();
                
                if dx < 10 && dy < 10 {
                    return Some(GestureType::Tap);
                }
            }
        }
        
        None
    }
}

impl Default for InputCapabilities {
    fn default() -> Self {
        Self {
            multi_touch: false,
            pressure_sensitivity: false,
            gesture_recognition: false,
            haptic_feedback: false,
            wireless: false,
            programmable: false,
        }
    }
}

impl Default for Modifiers {
    fn default() -> Self {
        Self::empty()
    }
}

impl Default for MouseButtons {
    fn default() -> Self {
        Self::empty()
    }
}

impl Default for StylusButtons {
    fn default() -> Self {
        Self::empty()
    }
}

impl Default for GestureData {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            parameters: [0.0; 4],
        }
    }
}

impl Default for SensorData {
    fn default() -> Self {
        Self {
            values: [0.0; 3],
            accuracy: SensorAccuracy::Medium,
            timestamp: 0,
        }
    }
}
