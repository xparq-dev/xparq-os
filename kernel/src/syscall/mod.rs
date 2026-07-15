// XPARQ OS - Phase 6: Syscall Gateway
// Manages system calls and MSR configuration

pub mod dispatcher;

pub fn init_syscalls() {
    // Configure STAR, LSTAR, SFMASK MSRs
    // Needs to be done per CPU, but for now we do it here
    let mut syscall = xparq_hal::x86_64::syscall::SYSCALL_MANAGER.lock();
    extern "C" {
        fn syscall_entry();
    }
    syscall.init(syscall_entry as *const () as u64);
    dispatcher::init_syscalls();
}
