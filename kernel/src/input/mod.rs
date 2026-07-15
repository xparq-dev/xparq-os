// XPARQ OS - Phase 10: Input Architecture

pub trait InputDevice {
    fn push_event(&self, data: u8);
    fn read_event(&self) -> Option<u8>;
}

// Fixed-size Circular Buffer
#[derive(Debug)]
pub struct CircularBuffer<T, const N: usize> {
    pub data: [T; N],
    pub head: usize,
    pub tail: usize,
    pub count: usize,
}

impl<T: Copy + Default, const N: usize> CircularBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            data: [T::default(); N],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.count < N {
            self.data[self.tail] = item;
            self.tail = (self.tail + 1) % N;
            self.count += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.count > 0 {
            let item = self.data[self.head];
            self.head = (self.head + 1) % N;
            self.count -= 1;
            Some(item)
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

pub struct KeyboardDevice {
    buffer: spin::Mutex<CircularBuffer<u8, 256>>,
    pub wait_queue: spin::Mutex<crate::task::wait_queue::WaitQueue<8>>,
}

impl KeyboardDevice {
    pub const fn new() -> Self {
        Self {
            buffer: spin::Mutex::new(CircularBuffer {
                data: [0; 256],
                head: 0,
                tail: 0,
                count: 0,
            }),
            wait_queue: spin::Mutex::new(crate::task::wait_queue::WaitQueue::new()),
        }
    }
}

impl InputDevice for KeyboardDevice {
    fn push_event(&self, data: u8) {
        self.buffer.lock().push(data);
    }
    
    fn read_event(&self) -> Option<u8> {
        self.buffer.lock().pop()
    }
}

pub static KEYBOARD_DEVICE: KeyboardDevice = KeyboardDevice::new();

pub struct InputManager {
    pub event_queue: spin::Mutex<CircularBuffer<xparq_hal::input::InputEvent, 256>>,
    pub wait_queue: spin::Mutex<crate::task::wait_queue::WaitQueue<8>>,
}

impl InputManager {
    pub const fn new() -> Self {
        Self {
            event_queue: spin::Mutex::new(CircularBuffer {
                data: [xparq_hal::input::InputEvent {
                    timestamp: 0,
                    device_type: xparq_hal::input::InputDeviceType::Keyboard,
                    event_kind: xparq_hal::input::InputEventKind::MouseMove,
                    data: xparq_hal::input::InputEventData::Mouse { buttons: xparq_hal::input::MouseButtons::empty(), x: 0, y: 0, wheel_delta: 0 },
                }; 256],
                head: 0,
                tail: 0,
                count: 0,
            }),
            wait_queue: spin::Mutex::new(crate::task::wait_queue::WaitQueue::new()),
        }
    }
}

pub static INPUT_MANAGER: InputManager = InputManager::new();

pub fn kernel_keyboard_callback(event: &xparq_hal::input::InputEvent) {
    INPUT_MANAGER.event_queue.lock().push(*event);
    INPUT_MANAGER.wait_queue.lock().wake_one();
}

pub fn kernel_mouse_callback(event: &xparq_hal::input::InputEvent) {
    INPUT_MANAGER.event_queue.lock().push(*event);
    INPUT_MANAGER.wait_queue.lock().wake_one();
}
