// XPARQ OS - Phase 20: Full Kernel Entry
// Boot → HAL → Scheduler → GUI → User Space Shell
#![no_std]
#![no_main]

use core::fmt::Write;
#[macro_use]
extern crate xparq_hal as hal;

// Kernel modules
mod arch;
mod capability;
mod cpu;
mod desktop;
mod fs;
mod input;
mod ipc;
mod memory;
mod net;
mod objects;
mod smp;
mod storage;
mod sync;
mod syscall;
mod task;
mod time;

// Box re-export from alloc
extern crate alloc;
pub use alloc::boxed::Box;

// Boot information structures passed to kernel
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    pub memory_regions: &'static [MemoryRegion],
    pub framebuffer: Option<FramebufferInfo>,
    pub arch_specific: ArchBootInfo,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub base: usize,
    pub size: usize,
    pub kind: MemoryRegionKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionKind {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
    Mmio,
}

#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub address: usize,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelFormat {
    Rgb32,
    Bgr32,
}

#[derive(Debug, Clone, Copy)]
pub struct ArchBootInfo {
    pub rsdp: usize,
    pub bootloader_brand: &'static str,
}

/// UART base address for x86_64
const UART_BASE: usize = 0x03F8;

#[cfg(target_arch = "x86_64")]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

#[cfg(target_arch = "x86_64")]
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
unsafe fn serial_init() {
    let base = UART_BASE as u16;
    outb(base + 1, 0x00);
    outb(base + 3, 0x80);
    outb(base + 0, 0x01);
    outb(base + 1, 0x00);
    outb(base + 3, 0x03);
    outb(base + 2, 0xC7);
    outb(base + 4, 0x0B);
}

unsafe fn uart_putc(c: u8) {
    while (inb((UART_BASE + 5) as u16) & 0x20) == 0 {}
    outb(UART_BASE as u16, c);
}

unsafe fn uart_puts(s: &[u8]) {
    for &byte in s {
        if byte == b'\n' {
            uart_putc(b'\r');
        }
        uart_putc(byte);
    }
}

fn u8_to_hex(byte: u8) -> [u8; 2] {
    let hex_chars = b"0123456789ABCDEF";
    [hex_chars[(byte >> 4) as usize], hex_chars[(byte & 0x0F) as usize]]
}

fn u32_to_hex(val: u32) -> [u8; 8] {
    let b3 = u8_to_hex((val >> 24) as u8);
    let b2 = u8_to_hex((val >> 16) as u8);
    let b1 = u8_to_hex((val >> 8) as u8);
    let b0 = u8_to_hex(val as u8);
    [b3[0], b3[1], b2[0], b2[1], b1[0], b1[1], b0[0], b0[1]]
}

// ─── IRQ Handlers ───────────────────────────────────────────────────────

#[no_mangle]
pub fn timer_interrupt_handler() {
    unsafe { uart_puts(b"    -> [timer_irq] entered\n"); }
    // 1. Send EOI
    crate::hal::x86_64::apic::timer_handler();

    let cpu_id = crate::cpu::id::current_cpu_id();

    // 2. Tick the scheduler/clock (BSP only)
    let mut manager = crate::task::TASK_MANAGER.lock();
    if cpu_id == 0 {
        manager.tick();

        crate::time::KERNEL_CLOCK.lock().tick();

        let abs_tick = crate::time::KERNEL_CLOCK.lock().get_ticks();
        let mut expired_events = [crate::time::TimerEvent {
            timer_type: crate::time::TimerType::Other,
            id: 0,
            arg: 0,
        }; 16];

        let mut timer_mgr = crate::time::TIMER_MANAGER.lock();
        let triggered = timer_mgr.tick(abs_tick, &mut expired_events);
        drop(timer_mgr);

        for i in 0..triggered {
            let _event = expired_events[i];
        }
    }

    let current_id_opt = crate::cpu::CPUS[cpu_id].lock().current_task;

    if let Some(current_id) = current_id_opt {
        let ptr = &mut manager.pool.get_task_mut(current_id).unwrap().stack_ptr as *mut u64;
        let next_sp = manager.schedule_next_for_cpu(cpu_id);

        drop(manager);

        unsafe {
            if *ptr != next_sp {
                unsafe { 
                    uart_puts(b"    -> [timer_irq] switching to next_sp: "); 
                    crate::hal::x86_64::idt::df_print_hex(next_sp);
                    let rip_to_jump = *( (next_sp + 56) as *const u64 );
                    uart_puts(b"    -> [timer_irq] target RIP: ");
                    crate::hal::x86_64::idt::df_print_hex(rip_to_jump);
                }
                crate::task::switch::switch_context(ptr, next_sp);
                unsafe { uart_puts(b"    -> [timer_irq] returned from switch\n"); }
            } else {
                unsafe { uart_puts(b"    -> [timer_irq] no switch needed\n"); }
            }
        }
    } else {
        drop(manager);
    }
}

#[no_mangle]
pub fn resched_ipi_handler() {
    unsafe { uart_puts(b"    -> [resched_ipi] entered\n"); }
    crate::hal::x86_64::apic::LocalApic::init(0xFEE00000).eoi();

    let cpu_id = crate::cpu::id::current_cpu_id();
    let current_id_opt = crate::cpu::CPUS[cpu_id].lock().current_task;

    if let Some(current_id) = current_id_opt {
        unsafe { uart_puts(b"    -> [resched_ipi] locking TASK_MANAGER\n"); }
        let mut manager = crate::task::TASK_MANAGER.lock();
        let ptr = &mut manager.pool.get_task_mut(current_id).unwrap().stack_ptr as *mut u64;
        unsafe { uart_puts(b"    -> [resched_ipi] getting next sp\n"); }
        let next_sp = manager.schedule_next_for_cpu(cpu_id);

        drop(manager);
        unsafe { uart_puts(b"    -> [resched_ipi] dropped TASK_MANAGER\n"); }

        unsafe {
            if *ptr != next_sp {
                unsafe { uart_puts(b"    -> [resched_ipi] switching context!\n"); }
                crate::task::switch::switch_context(ptr, next_sp);
                unsafe { uart_puts(b"    -> [resched_ipi] returned from switch!\n"); }
            } else {
                unsafe { uart_puts(b"    -> [resched_ipi] no switch needed\n"); }
            }
        }
    }
}

#[no_mangle]
pub fn wake_ipi_handler() {
    crate::hal::x86_64::apic::LocalApic::init(0xFEE00000).eoi();
}

fn dbg_serial(msg: &[u8]) {
    unsafe {
        for &byte in msg {
            uart_putc(byte);
        }
    }
}

// ─── Task Entry Points ─────────────────────────────────────────────────

pub fn gui_task_entry() {
    unsafe { uart_puts(b"[GUI] gui_task_entry started\n"); }
    use hal::display::DisplayDriver;
    let mut display = hal::x86_64::display::X86Display::new();
    if display.init().is_ok() {
        unsafe { uart_puts(b"[GUI] display init OK\n"); }
        crate::desktop::DESKTOP_MANAGER.lock().init(&mut display);
        unsafe { uart_puts(b"[GUI] desktop manager init OK\n"); }
        loop {
            // Process all pending input events
            loop {
                let event_opt = crate::input::INPUT_MANAGER.event_queue.lock().pop();
                if let Some(event) = event_opt {
                    crate::desktop::DESKTOP_MANAGER.lock().process_event(event);
                } else {
                    break;
                }
            }

            // Draw if needed
            let needs_redraw = crate::desktop::DESKTOP_MANAGER.lock().needs_redraw;
            if needs_redraw {
                unsafe { uart_puts(b"[GUI] draw start\n"); }
                crate::desktop::DESKTOP_MANAGER.lock().draw(&mut display);
                unsafe { uart_puts(b"[GUI] draw done\n"); }
            }

            // Block until new input
            crate::input::INPUT_MANAGER.wait_queue.lock().block_current(crate::task::state::BlockReason::Event);
        }
    } else {
        unsafe { uart_puts(b"[GUI] display init FAILED\n"); }
        loop { crate::syscall::dispatcher::sys_sleep(1000); }
    }
}

pub fn task1_entry() {
    loop {
        crate::syscall::dispatcher::sys_sleep(100);
    }
}

pub fn task2_entry() {
    loop {
        crate::syscall::dispatcher::sys_sleep(200);
    }
}

// ─── Kernel Main ────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
        extern "C" {
            static mut __bss_start: u8;
            static mut __bss_end: u8;
        }
        let bss_start = &raw mut __bss_start as *mut u8;
        let bss_end = &raw mut __bss_end as *mut u8;
        let bss_len = (bss_end as usize).saturating_sub(bss_start as usize);

        core::arch::asm!(
            "rep stosb",
            inout("rdi") bss_start => _,
            inout("rcx") bss_len => _,
            in("al") 0u8,
            options(nostack, preserves_flags)
        );

        serial_init();
        uart_puts(b"[XPARQ OS] Booting on x86_64...\n");

        // Initialize Virtual Memory (Phase 7)
        uart_puts(b"[DEBUG] Calling memory::init()\n");
        crate::memory::init();
        uart_puts(b"[DEBUG] Returned from memory::init()\n");

        // Initialize HAL
        uart_puts(b"[DEBUG] Calling init_arch_specific()\n");
        if hal::x86_64::init_arch_specific().is_ok() {
            uart_puts(b"[DEBUG] init_arch_specific() OK\n");

            // Register input callbacks so keyboard/mouse IRQs route to kernel's INPUT_MANAGER
            {
                use hal::input::InputDriver;
                if let Some(keyboard) = hal::x86_64::PS2_KEYBOARD.lock().as_mut() {
                    keyboard.set_event_callback(Some(crate::input::kernel_keyboard_callback));
                }
                if let Some(mouse) = hal::x86_64::PS2_MOUSE.lock().as_mut() {
                    mouse.set_event_callback(Some(crate::input::kernel_mouse_callback));
                }
            }
            uart_puts(b"[DEBUG] Input callbacks registered\n");

            // Init Networking Stack
            crate::net::NETWORK_MANAGER.lock().init();
            uart_puts(b"[DEBUG] Network Manager initialized\n");

            // Init Syscalls
            crate::syscall::init_syscalls();
            uart_puts(b"[DEBUG] Syscalls initialized\n");

            // Allocate task stacks using the frame allocator.
            // This puts them at safe physical addresses (>= 16 MB),
            // completely separated from the kernel boot stack which
            // descends from ~0x14E800 and could otherwise overlap
            // with BSS static arrays declared near that address.
            uart_puts(b"[DEBUG] Allocating task stacks\n");
            let stack_idle = crate::memory::frame::FRAME_ALLOCATOR.lock()
                .allocate_frames(2).expect("no frame for idle stack");
            let stack1 = crate::memory::frame::FRAME_ALLOCATOR.lock()
                .allocate_frames(2).expect("no frame for task1 stack");
            let stack2 = crate::memory::frame::FRAME_ALLOCATOR.lock()
                .allocate_frames(2).expect("no frame for task2 stack");
            let stack_gui = crate::memory::frame::FRAME_ALLOCATOR.lock()
                .allocate_frames(4).expect("no frame for gui stack");
            // Zero the stacks so there is no garbage data
            core::ptr::write_bytes(stack_idle as *mut u8, 0, 8192);
            core::ptr::write_bytes(stack1    as *mut u8, 0, 8192);
            core::ptr::write_bytes(stack2    as *mut u8, 0, 8192);
            core::ptr::write_bytes(stack_gui as *mut u8, 0, 16384);

            uart_puts(b"[DEBUG] Spawning Idle Task\n");
            core::arch::asm!("cli", options(nomem, nostack));
            let mut manager = crate::task::TASK_MANAGER.lock();
            let idle_id = manager.spawn_task(crate::task::idle::idle_task_entry, stack_idle, 8192).unwrap();

            // Assign idle task to CPU 0
            uart_puts(b"[DEBUG] Assigning to CPU0\n");
            let mut cpu0 = crate::cpu::CPUS[0].lock();
            cpu0.scheduler.idle_task = Some(idle_id);
            cpu0.scheduler.ready_queue.remove(&mut manager.pool, idle_id);
            drop(cpu0);

            uart_puts(b"[DEBUG] Spawning User Tasks\n");
            let _ = manager.spawn_task(task1_entry,     stack1,    8192);
            let _ = manager.spawn_task(task2_entry,     stack2,    8192);

            // Phase 20: GUI Task Enablement
            let _ = manager.spawn_task(gui_task_entry,  stack_gui, 16384);

            // Register the kernel boot thread as a proper task.
            // The kernel boot stack was set up by the bootloader; we probe
            // the current RSP to record a safe kernel_stack_top value.
            uart_puts(b"[DEBUG] Setting first task\n");
            let kernel_stack_top: u64;
            core::arch::asm!("mov {}, rsp", out(reg) kernel_stack_top, options(nomem, nostack));
            // Round up to next 4KB boundary to give a generous top
            let kernel_stack_top = (kernel_stack_top + 4095) & !4095u64;

            let kernel_task_id = manager.pool.allocate_task().unwrap();
            let task = manager.pool.get_task_mut(kernel_task_id).unwrap();
            task.kernel_stack_top = kernel_stack_top;
            task.state = crate::task::state::TaskState::Running;

            unsafe {
                crate::syscall::dispatcher::CPU_LOCAL.kernel_rsp = task.kernel_stack_top;
                crate::hal::x86_64::gdt::set_kernel_stack(task.kernel_stack_top);
            }

            let mut cpu0 = crate::cpu::CPUS[0].lock();
            cpu0.current_task = Some(kernel_task_id);
            drop(cpu0);
            drop(manager);
            core::arch::asm!("sti", options(nomem, nostack));


            uart_puts(b"[DEBUG] Registering IPI handlers\n");
            hal::x86_64::idt::register_irq_handler(32, timer_interrupt_handler);
            hal::x86_64::idt::register_irq_handler(0xF0, resched_ipi_handler);
            hal::x86_64::idt::register_irq_handler(0xF1, wake_ipi_handler);

            uart_puts(b"[XPARQ OS] Multitasking & SMP Enabled!\n");

            // Mount FAT32 VFS from Storage
            crate::storage::STORAGE_MANAGER.lock().init();
            if let Some(vol) = crate::storage::STORAGE_MANAGER.lock().volumes.first() {
                if vol.fs_type == 0x0C || vol.fs_type == 0x0B { // FAT32
                    let mut buf = [0u8; 512];
                    if let Some(storage) = hal::x86_64::STORAGE.lock().as_mut() {
                        use hal::storage::StorageDriver;
                        if storage.read(vol.device_id, vol.start_lba, &mut buf).is_ok() {
                            let bpb_ptr = buf.as_ptr() as *const xparq_hal::fs::fat32::Fat32Bpb;
                            let bpb = unsafe { *bpb_ptr };
                            let part = xparq_hal::fs::mbr::MbrPartitionEntry {
                                bootable: 0,
                                start_chs: [0; 3],
                                partition_type: 0x0B,
                                end_chs: [0; 3],
                                start_lba: vol.start_lba as u32,
                                sector_count: vol.sector_count as u32,
                            };
                            let fat32_fs = xparq_hal::fs::fat32::Fat32Fs::new(bpb, part);
                            let fat32_vfs = crate::fs::fat32_vfs::Fat32Vfs {
                                fs: fat32_fs,
                                volume_id: 0,
                                device_id: vol.device_id,
                            };
                            crate::fs::VFS_MANAGER.lock().mount_root(crate::fs::FileSystemVariant::Fat32(fat32_vfs));
                            uart_puts(b"[XPARQ OS] FAT32 VFS Mounted!\n");
                        }
                    }
                }
            }

            // Load init.elf into user space
            uart_puts(b"[XPARQ OS] Loading init.elf...\n");
            let result = crate::syscall::dispatcher::sys_execve(
                b"INIT.ELF" as *const u8 as u64,
                8,
            );

            if result < 0 {
                uart_puts(b"[XPARQ OS] Failed to load init.elf!\n");
                loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
            }

            uart_puts(b"[XPARQ OS] Jumping to PID 1 (Init Shell)\n");

            // Disable interrupts for the critical section where we read and jump
            core::arch::asm!("cli", options(nomem, nostack));
            {
                let manager = crate::task::TASK_MANAGER.lock();
                // manager is used to keep the lock for a moment
                drop(manager);
            }

            let user_rip = crate::syscall::dispatcher::CPU_LOCAL.user_rip;
            let user_rsp = crate::syscall::dispatcher::CPU_LOCAL.user_rsp;

            if user_rip == 0 || user_rsp == 0 {
                uart_puts(b"[XPARQ OS] ERROR: user_rip or user_rsp is zero!\n");
                loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
            }

            // Jump to Ring 3
            crate::task::switch::jump_to_ring3(user_rip, user_rsp);

            loop {
                core::arch::asm!("cli; hlt", options(nomem, nostack));
            }
        } else {
            uart_puts(b"[XPARQ OS] HAL init failed!\n");
            loop {
                core::arch::asm!("cli; hlt", options(nomem, nostack));
            }
        }
    }
}

// ─── Boot Entry ─────────────────────────────────────────────────────────

#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[link_section = ".text.init"]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "lea rsp, [rip + STACK + 16384]",
        "call {kernel_main}",
        kernel_main = sym kernel_main,
    );
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[link_section = ".bss.stack"]
static mut STACK: [u8; 16384] = [0; 16384];

// Global allocator placeholder
#[global_allocator]
static DUMMY_ALLOCATOR: DummyAllocator = DummyAllocator;

struct DummyAllocator;

unsafe impl core::alloc::GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        let ptr = unsafe { self.alloc(layout) };
        if !ptr.is_null() {
            unsafe { core::ptr::write_bytes(ptr, 0, layout.size()) };
        }
        ptr
    }

    unsafe fn realloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout, _new_size: usize) -> *mut u8 {
        core::ptr::null_mut()
    }
}
