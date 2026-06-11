// XPARQ OS - x86-64 Interrupt Descriptor Table (IDT)

use core::mem;
use core::ptr::write_volatile;
use crate::x86_64::apic;
use spin::Mutex;
use crate::x86_64::apic::timer_handler;

/// IDT entry flags
bitflags::bitflags! {
    pub struct IdtFlags: u8 {
        const PRESENT = 1 << 7;
        const RING_0 = 0 << 5;
        const INTERRUPT_GATE = 0b1110;
    }
}

/// IDT entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    flags: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    /// Create an empty IDT entry
    pub const fn empty() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            flags: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    /// Set handler address
    pub fn set_handler(&mut self, handler: u64, flags: IdtFlags) {
        self.selector = 0x08; // kernel code segment selector
        self.flags = flags.bits();
        self.offset_low = (handler & 0xFFFF) as u16;
        self.offset_mid = ((handler >> 16) & 0xFFFF) as u16;
        self.offset_high = (handler >> 32) as u32;
    }
}

/// IDT pointer (for lidt)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

/// IDT (256 entries)
static mut IDT: [IdtEntry; 256] = [IdtEntry::empty(); 256];

/// Interrupt handler type
pub type IrqHandler = fn();

/// Registered IRQ handlers (vector -> handler)
static IRQ_HANDLERS: Mutex<[Option<IrqHandler>; 256]> = Mutex::new([None; 256]);

/// Register an interrupt handler for a specific vector
pub fn register_irq_handler(vector: u8, handler: IrqHandler) {
    let mut handlers = IRQ_HANDLERS.lock();
    handlers[vector as usize] = Some(handler);
}

/// Dispatch an interrupt to the registered handler
#[no_mangle]
pub extern "C" fn irq_dispatch(vector: u8) {
    let handlers = IRQ_HANDLERS.lock();
    if let Some(handler) = handlers[vector as usize] {
        handler();
    }
    // Send EOI
    unsafe {
        if let Some(lapic) = &apic::LOCAL_APIC {
            lapic.eoi();
        }
    }
}

// --- Interrupt Handler Stubs ---

macro_rules! exception_handler {
    ($name:ident) => {
        #[naked]
        pub unsafe extern "C" fn $name() {
            core::arch::asm!(
                "cli",
                "push rax",
                "push rbx",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push rbp",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "push r12",
                "push r13",
                "push r14",
                "push r15",
                "pop r15",
                "pop r14",
                "pop r13",
                "pop r12",
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rbp",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rbx",
                "pop rax",
                "iretq",
                options(noreturn)
            );
        }
    };
}

macro_rules! irq_handler {
    ($name:ident, $vector:expr) => {
        #[naked]
        pub unsafe extern "C" fn $name() {
            core::arch::asm!(
                "cli",
                "push rax",
                "push rbx",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push rbp",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "push r12",
                "push r13",
                "push r14",
                "push r15",
                "mov rdi, {vector}",
                "call irq_dispatch",
                "pop r15",
                "pop r14",
                "pop r13",
                "pop r12",
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rbp",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rbx",
                "pop rax",
                "iretq",
                vector = const $vector,
                options(noreturn)
            );
        }
    };
}

// Define handlers for common CPU exceptions
exception_handler!(divide_by_zero);
exception_handler!(debug_exception);
exception_handler!(non_maskable_interrupt);
exception_handler!(breakpoint);
exception_handler!(overflow);
exception_handler!(bound_range_exceeded);
exception_handler!(invalid_opcode);
exception_handler!(device_not_available);
exception_handler!(double_fault);
exception_handler!(coprocessor_segment_overrun);
exception_handler!(invalid_tss);
exception_handler!(segment_not_present);
exception_handler!(stack_segment_fault);
exception_handler!(general_protection_fault);
exception_handler!(page_fault);

// Define handlers for IRQs (vectors 32-47)
irq_handler!(irq0, 32);
irq_handler!(irq1, 33);
irq_handler!(irq2, 34);
irq_handler!(irq3, 35);
irq_handler!(irq4, 36);
irq_handler!(irq5, 37);
irq_handler!(irq6, 38);
irq_handler!(irq7, 39);
irq_handler!(irq8, 40);
irq_handler!(irq9, 41);
irq_handler!(irq10, 42);
irq_handler!(irq11, 43);
irq_handler!(irq12, 44);
irq_handler!(irq13, 45);
irq_handler!(irq14, 46);
irq_handler!(irq15, 47);

/// Initialize the IDT
pub fn init() {
    unsafe {
        // Set up exception handlers (vectors 0-14)
        IDT[0].set_handler(divide_by_zero as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[1].set_handler(debug_exception as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[2].set_handler(non_maskable_interrupt as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[3].set_handler(breakpoint as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[4].set_handler(overflow as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[5].set_handler(bound_range_exceeded as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[6].set_handler(invalid_opcode as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[7].set_handler(device_not_available as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[8].set_handler(double_fault as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[9].set_handler(coprocessor_segment_overrun as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[10].set_handler(invalid_tss as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[11].set_handler(segment_not_present as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[12].set_handler(stack_segment_fault as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[13].set_handler(general_protection_fault as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[14].set_handler(page_fault as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);

        // Set up IRQ handlers (vectors 32-47)
        IDT[32].set_handler(irq0 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[33].set_handler(irq1 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[34].set_handler(irq2 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[35].set_handler(irq3 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[36].set_handler(irq4 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[37].set_handler(irq5 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[38].set_handler(irq6 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[39].set_handler(irq7 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[40].set_handler(irq8 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[41].set_handler(irq9 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[42].set_handler(irq10 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[43].set_handler(irq11 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[44].set_handler(irq12 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[45].set_handler(irq13 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[46].set_handler(irq14 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);
        IDT[47].set_handler(irq15 as u64, IdtFlags::PRESENT | IdtFlags::RING_0 | IdtFlags::INTERRUPT_GATE);

        let idt_ptr = IdtPointer {
            limit: (mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: &IDT as *const _ as u64,
        };

        // Load the IDT
        core::arch::asm!("lidt [{}]", in(reg) &idt_ptr, options(nostack));
    }
}
