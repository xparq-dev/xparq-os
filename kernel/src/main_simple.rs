// XPARQ OS - Phase 3 Kernel with HAL VGA Driver
// Uses HAL's x86_64 VBE display driver, minimal HAL init
#![no_std]
#![no_main]

use core::fmt::Write;
use xparq_hal as hal;

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
    outb(base + 1, 0x00); // Disable interrupts
    outb(base + 3, 0x80); // Enable DLAB
    outb(base + 0, 0x01); // Divisor low byte (115200 baud)
    outb(base + 1, 0x00); // Divisor high byte
    outb(base + 3, 0x03); // 8 bits, no parity, one stop bit
    outb(base + 2, 0xC7); // Enable FIFO, clear them, 14-byte threshold
    outb(base + 4, 0x0B); // IRQs enabled, RTS/DSR set
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

/// Helper to print hex bytes
fn u8_to_hex(byte: u8) -> [u8; 2] {
    let hex_chars = b"0123456789ABCDEF";
    [hex_chars[(byte >> 4) as usize], hex_chars[(byte & 0x0F) as usize]]
}

fn u16_to_hex(word: u16) -> [u8; 4] {
    let b0 = u8_to_hex((word >> 8) as u8);
    let b1 = u8_to_hex(word as u8);
    [b0[0], b0[1], b1[0], b1[1]]
}

/// Kernel main function - called from bootloader
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
        serial_init();
        uart_puts(b"[XPARQ OS] Booting on x86_64...\n");

        // Initialize just HAL x86_64 arch specific (only VGA display)
        uart_puts(b"[XPARQ OS] Initializing HAL x86_64 arch...\n");
        if hal::x86_64::init_arch_specific().is_ok() {
            uart_puts(b"[XPARQ OS] HAL arch initialized!\n");

            // Use HAL's display
            if let Some(mut display) = hal::x86_64::DISPLAY.lock().take() {
                uart_puts(b"[XPARQ OS] Display obtained!\n");
                writeln!(&mut display, "XPARQ OS - Phase 3 with HAL!").unwrap();
                writeln!(&mut display, "Booted Successfully!").unwrap();
                writeln!(&mut display, "").unwrap();

                // Enumerate PCI devices
                uart_puts(b"[XPARQ OS] Enumerating PCI devices...\n");
                writeln!(&mut display, "PCI Devices:").unwrap();

                let devices = hal::x86_64::pci::get_devices();
                for dev in devices {
                    // Print bus:dev:func
                    let bus_hex = u8_to_hex(dev.func.bus);
                    let dev_hex = u8_to_hex(dev.func.device);
                    let func_hex = u8_to_hex(dev.func.function);
                    let vendor_hex = u16_to_hex(dev.device_id.vendor_id);
                    let device_hex = u16_to_hex(dev.device_id.device_id);
                    let class_base_hex = u8_to_hex(dev.class_code.base);
                    let class_sub_hex = u8_to_hex(dev.class_code.sub);
                    let class_if_hex = u8_to_hex(dev.class_code.interface);

                    // Print to display
                    writeln!(&mut display, 
                        "[{}{}:{}{}:{}{}] {}{}{}{}:{}{}{}{} {}{}:{}{}:{}{}",
                        bus_hex[0] as char, bus_hex[1] as char,
                        dev_hex[0] as char, dev_hex[1] as char,
                        func_hex[0] as char, func_hex[1] as char,
                        vendor_hex[0] as char, vendor_hex[1] as char, vendor_hex[2] as char, vendor_hex[3] as char,
                        device_hex[0] as char, device_hex[1] as char, device_hex[2] as char, device_hex[3] as char,
                        class_base_hex[0] as char, class_base_hex[1] as char,
                        class_sub_hex[0] as char, class_sub_hex[1] as char,
                        class_if_hex[0] as char, class_if_hex[1] as char,
                    ).unwrap();

                    // Print to UART
                    uart_putc(b'[');
                    uart_putc(bus_hex[0]);
                    uart_putc(bus_hex[1]);
                    uart_putc(b':');
                    uart_putc(dev_hex[0]);
                    uart_putc(dev_hex[1]);
                    uart_putc(b':');
                    uart_putc(func_hex[0]);
                    uart_putc(func_hex[1]);
                    uart_putc(b']');
                    uart_putc(b' ');
                    uart_putc(vendor_hex[0]);
                    uart_putc(vendor_hex[1]);
                    uart_putc(vendor_hex[2]);
                    uart_putc(vendor_hex[3]);
                    uart_putc(b':');
                    uart_putc(device_hex[0]);
                    uart_putc(device_hex[1]);
                    uart_putc(device_hex[2]);
                    uart_putc(device_hex[3]);
                    uart_putc(b'\n');
                }

                // Put display back
                *hal::x86_64::DISPLAY.lock() = Some(display);
                uart_puts(b"[XPARQ OS] PCI enumeration complete!\n");
            }
        } else {
            uart_puts(b"[XPARQ OS] HAL init failed!\n");
        }

        uart_puts(b"[XPARQ OS] System in idle loop.\n");

        // Infinite idle loop
        loop {
            core::arch::asm!("cli; hlt", options(nomem, nostack));
        }
    }
}

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
