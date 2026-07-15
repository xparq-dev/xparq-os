// XPARQ OS - Phase 6/18: Kernel Clock & Timer Manager
// Manages system uptime and a Hashed Timer Wheel for O(1) timer processing.
// Configurable and no_alloc compliant.

use spin::Mutex;

pub struct KernelClock {
    pub ticks: u64,
}

impl KernelClock {
    pub const fn new() -> Self {
        Self { ticks: 0 }
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
    }

    pub fn get_ticks(&self) -> u64 {
        self.ticks
    }
}

pub static KERNEL_CLOCK: Mutex<KernelClock> = Mutex::new(KernelClock::new());

const WHEEL_SIZE: usize = 256;
const WHEEL_MASK: u64 = (WHEEL_SIZE - 1) as u64;
pub const MAX_TIMERS: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerType {
    TcpRto,
    TcpTimeWait,
    Sleep,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct TimerEvent {
    pub timer_type: TimerType,
    pub id: u64, // e.g., socket ID or task ID
    pub arg: u64,
}

#[derive(Clone, Copy)]
struct TimerEntry {
    in_use: bool,
    expires_at: u64,
    event: TimerEvent,
    next: Option<u16>,
}

impl TimerEntry {
    const fn empty() -> Self {
        Self {
            in_use: false,
            expires_at: 0,
            event: TimerEvent {
                timer_type: TimerType::TcpRto,
                id: 0,
                arg: 0,
            },
            next: None,
        }
    }
}

pub struct TimerManager {
    wheel: [Option<u16>; WHEEL_SIZE],
    timers: [TimerEntry; MAX_TIMERS],
    current_tick: u64,
}

impl TimerManager {
    pub const fn new() -> Self {
        Self {
            wheel: [None; WHEEL_SIZE],
            timers: [TimerEntry::empty(); MAX_TIMERS],
            current_tick: 0,
        }
    }

    /// Schedule a new timer. Returns a timer handle (index) or None if out of slots.
    pub fn schedule(&mut self, delay_ticks: u64, event: TimerEvent) -> Option<u16> {
        let expires_at = self.current_tick + delay_ticks;
        let slot = (expires_at & WHEEL_MASK) as usize;

        let mut free_idx = None;
        for i in 0..MAX_TIMERS {
            if !self.timers[i].in_use {
                free_idx = Some(i as u16);
                break;
            }
        }

        let idx = free_idx?;

        self.timers[idx as usize] = TimerEntry {
            in_use: true,
            expires_at,
            event,
            next: self.wheel[slot],
        };

        self.wheel[slot] = Some(idx);
        Some(idx)
    }

    /// Cancel an active timer by handle.
    pub fn cancel(&mut self, handle: u16) {
        let idx = handle as usize;
        if idx >= MAX_TIMERS || !self.timers[idx].in_use {
            return;
        }

        let slot = (self.timers[idx].expires_at & WHEEL_MASK) as usize;
        
        let mut prev: Option<u16> = None;
        let mut curr = self.wheel[slot];

        while let Some(c) = curr {
            if c == handle {
                if let Some(p) = prev {
                    self.timers[p as usize].next = self.timers[idx].next;
                } else {
                    self.wheel[slot] = self.timers[idx].next;
                }
                self.timers[idx].in_use = false;
                break;
            }
            prev = Some(c);
            curr = self.timers[c as usize].next;
        }
    }

    /// Process the current tick and populate `expired_events` array.
    /// Returns the number of events triggered.
    pub fn tick(&mut self, abs_tick: u64, expired_events: &mut [TimerEvent]) -> usize {
        let mut count = 0;
        
        while self.current_tick <= abs_tick {
            let slot = (self.current_tick & WHEEL_MASK) as usize;
            
            let mut prev: Option<u16> = None;
            let mut curr = self.wheel[slot];

            while let Some(c) = curr {
                let idx = c as usize;
                let next = self.timers[idx].next;

                if self.timers[idx].expires_at <= self.current_tick {
                    if count < expired_events.len() {
                        expired_events[count] = self.timers[idx].event;
                        count += 1;
                    }

                    if let Some(p) = prev {
                        self.timers[p as usize].next = next;
                    } else {
                        self.wheel[slot] = next;
                    }
                    self.timers[idx].in_use = false;
                } else {
                    prev = Some(c);
                }

                curr = next;
            }
            
            self.current_tick += 1;
        }
        
        count
    }
}

pub static TIMER_MANAGER: Mutex<TimerManager> = Mutex::new(TimerManager::new());

/// Global API to easily schedule timers
pub fn schedule_timer(delay_ticks: u64, event: TimerEvent) -> Option<u16> {
    TIMER_MANAGER.lock().schedule(delay_ticks, event)
}

pub fn cancel_timer(handle: u16) {
    TIMER_MANAGER.lock().cancel(handle)
}
