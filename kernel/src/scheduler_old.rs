//! Scheduler - Phase 1: OS & Kernel Foundations
//! 
//! This module implements the thread scheduler for XPARQ OS, following the
//! Zircon scheduler model. It provides preemptive multitasking with priority
//! scheduling and fair CPU time allocation.
//! 
//! Phase 1: Basic scheduler structures and interfaces
//! Phase 2: Full scheduling algorithm with priority inheritance
//! Phase 3: Multi-CPU scheduling and load balancing

use crate::objects::{Thread, Process, Object};
use crate::arch::cpu;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Thread priority levels - Phase 1: Basic priority system
/// 
/// Higher numbers = higher priority. This follows the Zircon priority model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum ThreadPriority {
    Idle = 0,
    Lowest = 1,
    Low = 4,
    Normal = 8,
    Medium = 12,
    High = 16,
    Highest = 20,
    Critical = 24,
}

impl ThreadPriority {
    /// Convert from i32 to ThreadPriority
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => ThreadPriority::Idle,
            1 => ThreadPriority::Lowest,
            2 | 3 => ThreadPriority::Lowest,
            4 => ThreadPriority::Low,
            5..=7 => ThreadPriority::Low,
            8 => ThreadPriority::Normal,
            9..=11 => ThreadPriority::Normal,
            12 => ThreadPriority::Medium,
            13..=15 => ThreadPriority::Medium,
            16 => ThreadPriority::High,
            17..=19 => ThreadPriority::High,
            20 => ThreadPriority::Highest,
            21..=23 => ThreadPriority::Highest,
            24 => ThreadPriority::Critical,
            _ => ThreadPriority::Normal,
        }
    }
    
    /// Get priority as i32
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Thread state for scheduling
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum ThreadState {
    /// Thread is newly created but not started
    Created = 0,
    /// Thread is ready to run on CPU
    Ready = 1,
    /// Thread is currently running on CPU
    Running = 2,
    /// Thread is blocked (waiting for event, I/O, etc.)
    Blocked = 3,
    /// Thread is suspended (temporarily stopped)
    Suspended = 4,
    /// Thread is dying (in process of termination)
    Dying = 5,
    /// Thread is dead (terminated)
    Dead = 6,
}

/// Run queue per priority level
/// 
/// Each priority level has its own run queue to ensure O(1) scheduling.
#[derive(Debug)]
pub struct RunQueue {
    /// Queue of ready threads for this priority
    pub threads: arrayvec::ArrayVec<*mut Thread, 64>,
    /// Current length of queue
    pub length: usize,
}

impl RunQueue {
    /// Create new empty run queue
    pub const fn new() -> Self {
        Self {
            threads: arrayvec::ArrayVec::new(),
            length: 0,
        }
    }
    
    /// Add thread to run queue
    pub fn enqueue(&mut self, thread: *mut Thread) {
        if self.threads.push(thread).is_ok() {
            self.length += 1;
        }
    }
    
    /// Remove thread from run queue (FIFO)
    pub fn dequeue(&mut self) -> Option<*mut Thread> {
        if self.length > 0 {
            self.length -= 1;
            self.threads.remove(0)
        } else {
            None
        }
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
    
    /// Get queue length
    pub fn len(&self) -> usize {
        self.length
    }
}

/// CPU state for scheduling
#[derive(Debug)]
pub struct CpuState {
    /// Current CPU ID
    pub cpu_id: u32,
    /// Currently running thread on this CPU
    pub current_thread: Option<*mut Thread>,
    /// Idle thread for this CPU
    pub idle_thread: *mut Thread,
    /// Run queues for each priority level
    pub run_queues: [RunQueue; 25], // One for each priority level 0-24
    /// Preemption disabled flag
    pub preemption_disabled: bool,
    /// Last context switch time
    pub last_switch_time: u64,
    /// Time slice for current thread
    pub time_slice: u64,
}

impl CpuState {
    /// Create new CPU state
    pub fn new(cpu_id: u32) -> Self {
        Self {
            cpu_id,
            current_thread: None,
            idle_thread: core::ptr::null_mut(), // Will be set in init()
            run_queues: [RunQueue::new(); 25],
            preemption_disabled: false,
            last_switch_time: 0,
            time_slice: 0,
        }
    }
    
    /// Find highest priority non-empty run queue
    pub fn find_highest_priority_queue(&self) -> Option<usize> {
        // Search from highest priority (24) to lowest (0)
        for priority in (0..=24).rev() {
            if !self.run_queues[priority].is_empty() {
                return Some(priority);
            }
        }
        None
    }
    
    /// Add thread to appropriate run queue based on priority
    pub fn enqueue_thread(&mut self, thread: *mut Thread) {
        let priority = unsafe { (*thread).priority as usize };
        if priority <= 24 {
            self.run_queues[priority].enqueue(thread);
        }
    }
    
    /// Get next thread to run
    pub fn next_thread(&mut self) -> *mut Thread {
        if let Some(priority) = self.find_highest_priority_queue() {
            if let Some(thread) = self.run_queues[priority].dequeue() {
                return thread;
            }
        }
        
        // No threads ready, return idle thread
        self.idle_thread
    }
}

/// Main scheduler - Phase 1: Basic scheduler implementation
/// 
/// The scheduler manages thread execution across all CPUs in the system.
/// It implements priority-based preemptive scheduling.
#[derive(Debug)]
pub struct Scheduler {
    /// Array of CPU states (one per CPU)
    pub cpu_states: arrayvec::ArrayVec<CpuState, 64>,
    /// List of all threads in the system
    pub all_threads: arrayvec::ArrayVec<*mut Thread, 1024>,
    /// Next thread ID to allocate
    pub next_thread_id: AtomicU64,
    /// Current system time
    pub current_time: AtomicU64,
    /// Scheduler quantum (time slice) in nanoseconds
    pub quantum: u64,
    /// Scheduler running flag
    pub running: bool,
}

impl Scheduler {
    /// Initialize the scheduler
    /// 
    /// Sets up per-CPU state and creates idle threads.
    pub fn init() {
        println!("Initializing XPARQ OS Scheduler...");
        
        let cpu_count = cpu::cpu_count();
        let mut cpu_states = arrayvec::ArrayVec::new();
        
        // Create CPU state for each CPU
        for cpu_id in 0..cpu_count {
            let mut cpu_state = CpuState::new(cpu_id);
            
            // Create idle thread for this CPU
            let idle_thread = Self::create_idle_thread(cpu_id);
            cpu_state.idle_thread = idle_thread;
            cpu_state.current_thread = Some(idle_thread);
            
            cpu_states.push(cpu_state);
        }
        
        let scheduler = Scheduler {
            cpu_states,
            all_threads: arrayvec::ArrayVec::new(),
            next_thread_id: AtomicU64::new(1),
            current_time: AtomicU64::new(0),
            quantum: 10_000_000, // 10ms time slice
            running: false,
        };
        
        // Store global scheduler instance
        unsafe {
            SCHEDULER = Some(scheduler);
        }
        
        println!("Scheduler initialized for {} CPUs", cpu_count);
    }
    
    /// Create idle thread for a CPU
    fn create_idle_thread(cpu_id: u32) -> *mut Thread {
        let thread = unsafe {
            let thread_ptr = alloc_static::<Thread>();
            *thread_ptr = Thread {
                id: 0, // Will be set by register_thread
                name: {
                    let mut name = arrayvec::ArrayVec::new();
                    let idle_name = format_args!("idle-{}", cpu_id);
                    // For Phase 1, just use a simple name
                    name.extend_from_slice(b"idle");
                    name
                },
                process: core::ptr::null_mut(), // Idle thread has no process
                state: ThreadState::Running,
                cpu_affinity: 1u64 << cpu_id,
                priority: ThreadPriority::Idle.as_i32(),
                stack_pointer: 0, // Will be set by arch-specific code
                instruction_pointer: 0, // Will be set by arch-specific code
                ref_count: AtomicU32::new(1),
            };
            thread_ptr
        };
        
        thread
    }
    
    /// Start the scheduler
    /// 
    /// This never returns - it begins the scheduling loop.
    pub fn start() -> ! {
        println!("Starting scheduler...");
        
        let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };
        scheduler.running = true;
        
        // Start scheduling on the current CPU
        let cpu_id = cpu::current_cpu();
        Self::schedule_loop(cpu_id);
    }
    
    /// Main scheduling loop for a CPU
    fn schedule_loop(cpu_id: u32) -> ! {
        let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };
        let cpu_state = &mut scheduler.cpu_states[cpu_id as usize];
        
        loop {
            // Update system time
            let current_time = crate::arch::timer::current_time();
            scheduler.current_time.store(current_time, Ordering::Relaxed);
            
            // Check if we need to preempt current thread
            if Self::should_preempt(cpu_state, current_time) {
                Self::schedule_next(cpu_state, current_time);
            }
            
            // Yield CPU briefly (in real implementation, this would be
            // replaced with proper interrupt-driven preemption)
            for _ in 0..1000 {
                core::hint::spin_loop();
            }
        }
    }
    
    /// Check if current thread should be preempted
    fn should_preempt(cpu_state: &CpuState, current_time: u64) -> bool {
        if cpu_state.preemption_disabled {
            return false;
        }
        
        // Check if time slice expired
        if current_time - cpu_state.last_switch_time >= cpu_state.time_slice {
            return true;
        }
        
        // Check if higher priority thread is ready
        if let Some(current) = cpu_state.current_thread {
            let current_priority = unsafe { (*current).priority };
            if let Some(highest_priority) = cpu_state.find_highest_priority_queue() {
                if highest_priority as i32 > current_priority {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Schedule next thread to run
    fn schedule_next(cpu_state: &mut CpuState, current_time: u64) {
        // Put current thread back in run queue if it's still ready
        if let Some(current_thread) = cpu_state.current_thread {
            let current_state = unsafe { (*current_thread).state };
            if current_state == ThreadState::Running {
                cpu_state.enqueue_thread(current_thread);
            }
        }
        
        // Get next thread to run
        let next_thread = cpu_state.next_thread();
        
        // Perform context switch if thread changed
        if cpu_state.current_thread != Some(next_thread) {
            Self::context_switch(cpu_state, next_thread, current_time);
        }
    }
    
    /// Perform context switch between threads
    fn context_switch(cpu_state: &mut CpuState, next_thread: *mut Thread, current_time: u64) {
        let prev_thread = cpu_state.current_thread.unwrap_or(cpu_state.idle_thread);
        
        // Update thread states
        unsafe {
            (*prev_thread).state = ThreadState::Ready;
            (*next_thread).state = ThreadState::Running;
        }
        
        // Update CPU state
        cpu_state.current_thread = Some(next_thread);
        cpu_state.last_switch_time = current_time;
        cpu_state.time_slice = Self::calculate_time_slice(next_thread);
        
        // Architecture-specific context switch
        #[cfg(feature = "arm64")]
        crate::arm64::context::switch(prev_thread, next_thread);
        
        #[cfg(feature = "x86_64")]
        crate::x86_64::context::switch(prev_thread, next_thread);
        
        println!("Context switch: Thread {} -> Thread {} on CPU {}", 
                unsafe { (*prev_thread).id }, 
                unsafe { (*next_thread).id }, 
                cpu_state.cpu_id);
    }
    
    /// Calculate time slice for a thread based on priority
    fn calculate_time_slice(thread: *mut Thread) -> u64 {
        let priority = unsafe { (*thread).priority };
        let base_quantum = unsafe { SCHEDULER.as_ref().unwrap().quantum };
        
        // Higher priority threads get longer time slices
        match ThreadPriority::from_i32(priority) {
            ThreadPriority::Idle => 100_000,      // 100µs
            ThreadPriority::Lowest => 1_000_000,  // 1ms
            ThreadPriority::Low => 2_000_000,     // 2ms
            ThreadPriority::Normal => 10_000_000, // 10ms
            ThreadPriority::Medium => 20_000_000, // 20ms
            ThreadPriority::High => 50_000_000,   // 50ms
            ThreadPriority::Highest => 100_000_000, // 100ms
            ThreadPriority::Critical => 200_000_000, // 200ms
        }
    }
    
    /// Create new thread
    pub fn create_thread(
        process: *mut Process,
        name: &[u8],
        priority: ThreadPriority,
        entry: usize,
        stack: usize,
    ) -> Result<*mut Thread, ThreadError> {
        let scheduler = unsafe { SCHEDULER.as_ref().unwrap() };
        
        let thread_id = scheduler.next_thread_id.fetch_add(1, Ordering::AcqRel);
        
        let thread = unsafe {
            let thread_ptr = alloc_static::<Thread>();
            let mut name_array = arrayvec::ArrayVec::new();
            name_array.extend_from_slice(name);
            
            *thread_ptr = Thread {
                id: thread_id,
                name: name_array,
                process,
                state: ThreadState::Created,
                cpu_affinity: u64::MAX, // Can run on any CPU
                priority: priority.as_i32(),
                stack_pointer: stack,
                instruction_pointer: entry,
                ref_count: AtomicU32::new(1),
            };
            thread_ptr
        };
        
        // Register thread in scheduler
        unsafe {
            let scheduler = SCHEDULER.as_mut().unwrap();
            scheduler.all_threads.push(thread);
        }
        
        println!("Created thread {}: priority {}", thread_id, priority.as_i32());
        Ok(thread)
    }
    
    /// Make thread ready to run
    pub fn ready_thread(thread: *mut Thread, cpu_id: Option<u32>) {
        let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };
        
        // Set thread state to ready
        unsafe {
            (*thread).state = ThreadState::Ready;
        }
        
        // Add to appropriate CPU run queue
        if let Some(target_cpu) = cpu_id {
            if (target_cpu as usize) < scheduler.cpu_states.len() {
                scheduler.cpu_states[target_cpu as usize].enqueue_thread(thread);
            }
        } else {
            // Add to current CPU's run queue
            let current_cpu = cpu::current_cpu();
            scheduler.cpu_states[current_cpu as usize].enqueue_thread(thread);
        }
    }
    
    /// Block current thread
    pub fn block_thread(thread: *mut Thread) {
        unsafe {
            (*thread).state = ThreadState::Blocked;
        }
        
        // Remove from current CPU's run queue
        let current_cpu = cpu::current_cpu();
        let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };
        let cpu_state = &mut scheduler.cpu_states[current_cpu as usize];
        
        // Remove from run queue (simplified - in Phase 2 we'll have proper removal)
        if cpu_state.current_thread == Some(thread) {
            // This is the current thread, trigger reschedule
            cpu_state.current_thread = None;
        }
    }
    
    /// Yield current CPU to another thread
    pub fn yield_cpu() {
        let current_cpu = cpu::current_cpu();
        let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };
        let cpu_state = &mut scheduler.cpu_states[current_cpu as usize];
        
        // Force reschedule on next iteration
        cpu_state.last_switch_time = 0;
    }
}

/// Thread creation and management errors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThreadError {
    OutOfMemory,
    InvalidPriority,
    InvalidProcess,
    TooManyThreads,
}

/// Global scheduler instance
static mut SCHEDULER: Option<Scheduler> = None;

/// Allocate static memory for scheduler structures
unsafe fn alloc_static<T>() -> *mut T {
    extern "C" {
        static mut __bss_end: u8;
    }
    
    static mut BUMP_PTR: Option<*mut u8> = None;
    
    let ptr = if let Some(ptr) = BUMP_PTR {
        ptr
    } else {
        BUMP_PTR = Some(&mut __bss_end as *mut _ as *mut u8);
        BUMP_PTR.unwrap()
    };
    
    let aligned_ptr = (ptr as usize + core::mem::align_of::<T>() - 1) & !(core::mem::align_of::<T>() - 1);
    let result = aligned_ptr as *mut T;
    BUMP_PTR = Some((aligned_ptr as *mut u8).add(core::mem::size_of::<T>()));
    
    result
}

/// Get global scheduler
pub fn get_scheduler() -> &'static Scheduler {
    unsafe { SCHEDULER.as_ref().unwrap() }
}

/// Public API for thread management
pub mod api {
    use super::*;
    
    /// Create new thread
    pub fn create_thread(
        process: *mut Process,
        name: &[u8],
        priority: ThreadPriority,
        entry: usize,
        stack: usize,
    ) -> Result<*mut Thread, ThreadError> {
        Scheduler::create_thread(process, name, priority, entry, stack)
    }
    
    /// Start thread execution
    pub fn start_thread(thread: *mut Thread) -> Result<(), ThreadError> {
        Scheduler::ready_thread(thread, None);
        Ok(())
    }
    
    /// Yield current thread
    pub fn yield_current() {
        Scheduler::yield_cpu();
    }
    
    /// Block current thread
    pub fn block_current() {
        let current_cpu = cpu::current_cpu();
        let scheduler = unsafe { SCHEDULER.as_mut().unwrap() };
        let current_thread = scheduler.cpu_states[current_cpu as usize].current_thread.unwrap();
        Scheduler::block_thread(current_thread);
    }
}
