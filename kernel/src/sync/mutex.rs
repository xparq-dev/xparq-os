// XPARQ OS - Phase 20.5: Synchronization
// IRQ-Safe Mutex for Kernel Hardening

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// Reads RFLAGS
#[inline(always)]
fn read_rflags() -> u64 {
    let mut flags: u64;
    unsafe {
        core::arch::asm!("pushfq; pop {}", out(reg) flags, options(nomem, preserves_flags));
    }
    flags
}

/// Disables interrupts
#[inline(always)]
fn disable_interrupts() {
    unsafe { core::arch::asm!("cli", options(nomem, preserves_flags)); }
}

/// Enables interrupts
#[inline(always)]
fn enable_interrupts() {
    unsafe { core::arch::asm!("sti", options(nomem, preserves_flags)); }
}

/// Restores interrupts based on previous RFLAGS
#[inline(always)]
fn restore_interrupts(flags: u64) {
    if (flags & (1 << 9)) != 0 {
        enable_interrupts();
    }
}

pub struct IrqSafeMutex<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for IrqSafeMutex<T> {}
unsafe impl<T: Send> Send for IrqSafeMutex<T> {}

impl<T> IrqSafeMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> IrqSafeMutexGuard<'_, T> {
        let flags = read_rflags();
        disable_interrupts();

        while self.lock.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            // Spin loop hint
            unsafe { core::arch::asm!("pause"); }
        }

        IrqSafeMutexGuard {
            mutex: self,
            flags,
        }
    }
}

pub struct IrqSafeMutexGuard<'a, T> {
    mutex: &'a IrqSafeMutex<T>,
    flags: u64,
}

impl<T> Deref for IrqSafeMutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for IrqSafeMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for IrqSafeMutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.lock.store(false, Ordering::Release);
        restore_interrupts(self.flags);
    }
}
