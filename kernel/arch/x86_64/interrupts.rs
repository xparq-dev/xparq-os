// XPARQ OS - Phase 01: OS & Kernel Foundations
// x86-64 interrupt management
// Handles IDT, PIC controller, and interrupt handling

#![no_std]

use core::sync::atomic::{AtomicU64, Ordering};

/// Interrupt Descriptor Table (IDT) entry
#[repr(C, align(16))]
pub struct IDTEntry {
    pub offset_low: u16,
    pub selector: u16,
    pub ist: u8,
    pub type_attr: u8,
    pub offset_mid: u16,
    pub offset_high: u32,
    pub reserved: u32,
}

/// IDT pointer structure
#[repr(C, packed)]
pub struct IDTPointer {
    pub limit: u16,
    pub base: u64,
}

/// Exception types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExceptionType {
    DivideError,
    Debug,
    NMI,
    Breakpoint,
    Overflow,
    BoundRange,
    InvalidOpcode,
    DeviceNotAvailable,
    DoubleFault,
    InvalidTSS,
    SegmentNotPresent,
    StackSegmentFault,
    GeneralProtectionFault,
    PageFault,
    Reserved15,
    X87FloatingPoint,
    AlignmentCheck,
    MachineCheck,
    SimdFloatingPoint,
    Virtualization,
    Security,
    Reserved25,
    Reserved26,
    Reserved27,
    Reserved28,
    Reserved29,
    Reserved30,
    Reserved31,
}

/// Interrupt types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterruptType {
    Timer,
    Keyboard,
    Cascade,
    COM2,
    COM1,
    LPT2,
    Floppy,
    LPT1,
    CMOSRTC,
    Free,
    Free,
    Free,
    Mouse,
    FPU,
    PrimaryATA,
    SecondaryATA,
}

/// PIC (Programmable Interrupt Controller) interface
pub struct PIC {
    /// Master PIC base address
    pub master_base: u16,
    /// Slave PIC base address
    pub slave_base: u16,
}

/// PIC registers
#[repr(C)]
pub struct PICRegisters {
    pub command: volatile::Volatile<u8>,
    pub data: volatile::Volatile<u8>,
}

/// Global IDT
static mut IDT: [IDTEntry; 256] = [IDTEntry {
    offset_low: 0,
    selector: 0,
    ist: 0,
    type_attr: 0,
    offset_mid: 0,
    offset_high: 0,
    reserved: 0,
}; 256];

/// Global PIC instance
static mut PIC_INSTANCE: Option<PIC> = None;

/// Interrupt statistics
static INTERRUPT_COUNT: AtomicU64 = AtomicU64::new(0);
static EXCEPTION_COUNT: AtomicU64 = AtomicU64::new(0);

impl IDTEntry {
    /// Create a new IDT entry
    pub fn new(offset: u64, selector: u16, type_attr: u8, ist: u8) -> Self {
        Self {
            offset_low: (offset & 0xFFFF) as u16,
            selector,
            ist,
            type_attr,
            offset_mid: ((offset >> 16) & 0xFFFF) as u16,
            offset_high: ((offset >> 32) & 0xFFFFFFFF) as u32,
            reserved: 0,
        }
    }
    
    /// Create an interrupt gate
    pub fn interrupt_gate(offset: u64, selector: u16, dpl: u8) -> Self {
        let type_attr = 0x8E | ((dpl & 0x3) << 5); // Present, DPL, 64-bit interrupt gate
        Self::new(offset, selector, type_attr, 0)
    }
    
    /// Create a trap gate
    pub fn trap_gate(offset: u64, selector: u16, dpl: u8) -> Self {
        let type_attr = 0x8F | ((dpl & 0x3) << 5); // Present, DPL, 64-bit trap gate
        Self::new(offset, selector, type_attr, 0)
    }
    
    /// Create a task gate
    pub fn task_gate(selector: u16, dpl: u8) -> Self {
        let type_attr = 0xE5 | ((dpl & 0x3) << 5); // Present, DPL, task gate
        Self::new(0, selector, type_attr, 0)
    }
}

impl PIC {
    /// Create new PIC instance
    pub fn new(master_base: u16, slave_base: u16) -> Self {
        Self {
            master_base,
            slave_base,
        }
    }
    
    /// Initialize PIC
    pub fn init(&mut self) {
        println!("Initializing PIC...");
        
        // Save current interrupt masks
        let master_mask = self.read_command(self.master_base);
        let slave_mask = self.read_command(self.slave_base);
        
        // Start initialization sequence
        self.write_command(self.master_base, 0x11); // ICW1: ICW4 needed
        self.write_command(self.slave_base, 0x11);
        
        // Set interrupt vector offsets
        self.write_data(self.master_base, 0x20); // Master PIC: vectors 0x20-0x27
        self.write_data(self.slave_base, 0x28);  // Slave PIC: vectors 0x28-0x2F
        
        // Configure cascade
        self.write_data(self.master_base, 0x04); // Master PIC: IRQ2 is connected to slave
        self.write_data(self.slave_base, 0x02);  // Slave PIC: cascade identity
        
        // Set 8086 mode
        self.write_data(self.master_base, 0x01); // ICW4: 8086 mode
        self.write_data(self.slave_base, 0x01);
        
        // Restore interrupt masks
        self.write_data(self.master_base, master_mask);
        self.write_data(self.slave_base, slave_mask);
        
        println!("PIC initialized");
    }
    
    /// Write command to PIC
    fn write_command(&self, base: u16, command: u8) {
        unsafe {
            let pic = &mut *(base as *mut PICRegisters);
            pic.command.write(command);
        }
    }
    
    /// Write data to PIC
    fn write_data(&self, base: u16, data: u8) {
        unsafe {
            let pic = &mut *(base as *mut PICRegisters);
            pic.data.write(data);
        }
    }
    
    /// Read command from PIC
    fn read_command(&self, base: u16) -> u8 {
        unsafe {
            let pic = &*(base as *const PICRegisters);
            pic.command.read()
        }
    }
    
    /// Read data from PIC
    fn read_data(&self, base: u16) -> u8 {
        unsafe {
            let pic = &*(base as *const PICRegisters);
            pic.data.read()
        }
    }
    
    /// Send End of Interrupt (EOI)
    pub fn send_eoi(&mut self, irq: u8) {
        if irq >= 8 {
            // Send EOI to slave PIC
            self.write_command(self.slave_base, 0x20);
        }
        // Always send EOI to master PIC
        self.write_command(self.master_base, 0x20);
    }
    
    /// Enable IRQ
    pub fn enable_irq(&mut self, irq: u8) {
        let mask = 1 << (irq % 8);
        let base = if irq < 8 { self.master_base } else { self.slave_base };
        let current_mask = self.read_data(base);
        self.write_data(base, current_mask & !mask);
    }
    
    /// Disable IRQ
    pub fn disable_irq(&mut self, irq: u8) {
        let mask = 1 << (irq % 8);
        let base = if irq < 8 { self.master_base } else { self.slave_base };
        let current_mask = self.read_data(base);
        self.write_data(base, current_mask | mask);
    }
    
    /// Get IRQ in service
    pub fn get_isr(&self) -> u16 {
        unsafe {
            let master_isr = self.read_command(self.master_base | 0x0B);
            let slave_isr = self.read_command(self.slave_base | 0x0B);
            ((slave_isr as u16) << 8) | (master_isr as u16)
        }
    }
    
    /// Get IRQ request
    pub fn get_irr(&self) -> u16 {
        unsafe {
            let master_irr = self.read_command(self.master_base | 0x0A);
            let slave_irr = self.read_command(self.slave_base | 0x0A);
            ((slave_irr as u16) << 8) | (master_irr as u16)
        }
    }
}

/// Set up IDT
pub fn setup_idt() {
    println!("Setting up IDT...");
    
    unsafe {
        // Set up exception handlers (0-31)
        IDT[0] = IDTEntry::interrupt_gate(divide_error_handler as u64, 0x08, 0);
        IDT[1] = IDTEntry::interrupt_gate(debug_handler as u64, 0x08, 0);
        IDT[2] = IDTEntry::interrupt_gate(nmi_handler as u64, 0x08, 0);
        IDT[3] = IDTEntry::trap_gate(breakpoint_handler as u64, 0x08, 0);
        IDT[4] = IDTEntry::interrupt_gate(overflow_handler as u64, 0x08, 0);
        IDT[5] = IDTEntry::interrupt_gate(bound_range_handler as u64, 0x08, 0);
        IDT[6] = IDTEntry::interrupt_gate(invalid_opcode_handler as u64, 0x08, 0);
        IDT[7] = IDTEntry::interrupt_gate(device_not_available_handler as u64, 0x08, 0);
        IDT[8] = IDTEntry::interrupt_gate(double_fault_handler as u64, 0x08, 0);
        IDT[9] = IDTEntry::interrupt_gate(coprocessor_segment_overrun_handler as u64, 0x08, 0);
        IDT[10] = IDTEntry::interrupt_gate(invalid_tss_handler as u64, 0x08, 0);
        IDT[11] = IDTEntry::interrupt_gate(segment_not_present_handler as u64, 0x08, 0);
        IDT[12] = IDTEntry::interrupt_gate(stack_segment_fault_handler as u64, 0x08, 0);
        IDT[13] = IDTEntry::interrupt_gate(general_protection_fault_handler as u64, 0x08, 0);
        IDT[14] = IDTEntry::interrupt_gate(page_fault_handler as u64, 0x08, 0);
        IDT[15] = IDTEntry::interrupt_gate(reserved_handler as u64, 0x08, 0);
        IDT[16] = IDTEntry::interrupt_gate(x87_floating_point_handler as u64, 0x08, 0);
        IDT[17] = IDTEntry::interrupt_gate(alignment_check_handler as u64, 0x08, 0);
        IDT[18] = IDTEntry::interrupt_gate(machine_check_handler as u64, 0x08, 0);
        IDT[19] = IDTEntry::interrupt_gate(simd_floating_point_handler as u64, 0x08, 0);
        IDT[20] = IDTEntry::interrupt_gate(virtualization_handler as u64, 0x08, 0);
        IDT[21] = IDTEntry::interrupt_gate(security_handler as u64, 0x08, 0);
        
        // Set up interrupt handlers (32-255)
        for i in 32..256 {
            IDT[i] = IDTEntry::interrupt_gate(interrupt_handler as u64, 0x08, 0);
        }
        
        // Load IDT
        let idt_ptr = IDTPointer {
            limit: (256 * 16 - 1) as u16,
            base: IDT.as_ptr() as u64,
        };
        
        core::arch::asm!("lidt [{}]", in(reg) &idt_ptr);
    }
    
    println!("IDT set up");
}

/// Initialize PIC
pub fn init_pic() {
    println!("Initializing PIC...");
    
    unsafe {
        PIC_INSTANCE = Some(PIC::new(0x20, 0xA0)); // Master at 0x20, Slave at 0xA0
        
        if let Some(pic) = &mut PIC_INSTANCE {
            pic.init();
        }
    }
    
    println!("PIC initialized");
}

/// Enable interrupts
pub fn enable() {
    println!("Enabling interrupts...");
    
    // Initialize PIC
    init_pic();
    
    // Enable interrupts
    unsafe {
        core::arch::asm!("sti");
    }
    
    println!("Interrupts enabled");
}

/// Disable interrupts
pub fn disable() {
    println!("Disabling interrupts...");
    
    unsafe {
        core::arch::asm!("cli");
    }
    
    println!("Interrupts disabled");
}

/// Enable specific IRQ
pub fn enable_irq(irq: u8) {
    unsafe {
        if let Some(pic) = &mut PIC_INSTANCE {
            pic.enable_irq(irq);
        }
    }
}

/// Disable specific IRQ
pub fn disable_irq(irq: u8) {
    unsafe {
        if let Some(pic) = &mut PIC_INSTANCE {
            pic.disable_irq(irq);
        }
    }
}

/// Exception handlers
#[no_mangle]
extern "C" fn divide_error_handler() {
    exception_handler(ExceptionType::DivideError);
}

#[no_mangle]
extern "C" fn debug_handler() {
    exception_handler(ExceptionType::Debug);
}

#[no_mangle]
extern "C" fn nmi_handler() {
    exception_handler(ExceptionType::NMI);
}

#[no_mangle]
extern "C" fn breakpoint_handler() {
    exception_handler(ExceptionType::Breakpoint);
}

#[no_mangle]
extern "C" fn overflow_handler() {
    exception_handler(ExceptionType::Overflow);
}

#[no_mangle]
extern "C" fn bound_range_handler() {
    exception_handler(ExceptionType::BoundRange);
}

#[no_mangle]
extern "C" fn invalid_opcode_handler() {
    exception_handler(ExceptionType::InvalidOpcode);
}

#[no_mangle]
extern "C" fn device_not_available_handler() {
    exception_handler(ExceptionType::DeviceNotAvailable);
}

#[no_mangle]
extern "C" fn double_fault_handler() {
    exception_handler(ExceptionType::DoubleFault);
}

#[no_mangle]
extern "C" fn coprocessor_segment_overrun_handler() {
    exception_handler(ExceptionType::Reserved15);
}

#[no_mangle]
extern "C" fn invalid_tss_handler() {
    exception_handler(ExceptionType::InvalidTSS);
}

#[no_mangle]
extern "C" fn segment_not_present_handler() {
    exception_handler(ExceptionType::SegmentNotPresent);
}

#[no_mangle]
extern "C" fn stack_segment_fault_handler() {
    exception_handler(ExceptionType::StackSegmentFault);
}

#[no_mangle]
extern "C" fn general_protection_fault_handler() {
    exception_handler(ExceptionType::GeneralProtectionFault);
}

#[no_mangle]
extern "C" fn page_fault_handler() {
    exception_handler(ExceptionType::PageFault);
}

#[no_mangle]
extern "C" fn reserved_handler() {
    exception_handler(ExceptionType::Reserved15);
}

#[no_mangle]
extern "C" fn x87_floating_point_handler() {
    exception_handler(ExceptionType::X87FloatingPoint);
}

#[no_mangle]
extern "C" fn alignment_check_handler() {
    exception_handler(ExceptionType::AlignmentCheck);
}

#[no_mangle]
extern "C" fn machine_check_handler() {
    exception_handler(ExceptionType::MachineCheck);
}

#[no_mangle]
extern "C" fn simd_floating_point_handler() {
    exception_handler(ExceptionType::SimdFloatingPoint);
}

#[no_mangle]
extern "C" fn virtualization_handler() {
    exception_handler(ExceptionType::Virtualization);
}

#[no_mangle]
extern "C" fn security_handler() {
    exception_handler(ExceptionType::Security);
}

/// Generic exception handler
fn exception_handler(exception_type: ExceptionType) {
    let count = EXCEPTION_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("Exception #{}: {:?}", count, exception_type);
    
    // Phase 1: Basic exception handling
    // Phase 2: Full exception processing
    
    match exception_type {
        ExceptionType::GeneralProtectionFault => {
            println!("General Protection Fault");
            // Phase 2: Handle GPF properly
        }
        ExceptionType::PageFault => {
            println!("Page Fault");
            // Phase 2: Handle page fault properly
        }
        ExceptionType::DoubleFault => {
            println!("Double Fault - System Halted");
            crate::arch::x86_64::cpu::halt();
        }
        _ => {
            println!("Unhandled exception: {:?}", exception_type);
        }
    }
}

/// Generic interrupt handler
#[no_mangle]
extern "C" fn interrupt_handler() {
    let count = INTERRUPT_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("Interrupt #{}", count);
    
    // Phase 1: Basic interrupt handling
    // Phase 2: Full interrupt processing
    
    // Get interrupt vector
    let vector = 32; // Phase 2: Read actual vector from stack
    
    // Handle specific interrupts
    match vector {
        32 => {
            // Timer interrupt
            handle_timer_interrupt();
        }
        33 => {
            // Keyboard interrupt
            handle_keyboard_interrupt();
        }
        36 => {
            // COM1 interrupt
            handle_com1_interrupt();
        }
        _ => {
            println!("Unknown interrupt: {}", vector);
        }
    }
    
    // Send EOI to PIC
    unsafe {
        if let Some(pic) = &mut PIC_INSTANCE {
            pic.send_eoi(vector as u8 - 32);
        }
    }
}

/// Handle timer interrupt
fn handle_timer_interrupt() {
    println!("Timer interrupt");
    
    // Phase 1: Basic timer handling
    // Phase 2: Full timer management
    
    // Read timer value
    let tsc = super::boot::regs::rdtsc();
    println!("Timer TSC: {}", tsc);
}

/// Handle keyboard interrupt
fn handle_keyboard_interrupt() {
    println!("Keyboard interrupt");
    
    // Phase 1: Basic keyboard handling
    // Phase 2: Full keyboard handling
    
    // Read keyboard scancode
    let scancode = unsafe {
        let port = 0x60 as *mut u8;
        port.read_volatile()
    };
    
    println!("Keyboard scancode: 0x{:x}", scancode);
}

/// Handle COM1 interrupt
fn handle_com1_interrupt() {
    println!("COM1 interrupt");
    
    // Phase 1: Basic COM1 handling
    // Phase 2: Full COM1 interrupt handling
    
    // Check COM1 status
    let com1_status = crate::arch::x86_64::serial::get_status();
    println!("COM1 status: 0x{:x}", com1_status);
    
    // Read pending characters
    while let Some(c) = crate::arch::x86_64::serial::read_char() {
        println!("COM1 received: 0x{:x}", c);
    }
}

/// Interrupt statistics
pub mod stats {
    use super::*;
    
    /// Get interrupt count
    pub fn get_interrupt_count() -> u64 {
        INTERRUPT_COUNT.load(Ordering::SeqCst)
    }
    
    /// Get exception count
    pub fn get_exception_count() -> u64 {
        EXCEPTION_COUNT.load(Ordering::SeqCst)
    }
    
    /// Reset statistics
    pub fn reset_stats() {
        INTERRUPT_COUNT.store(0, Ordering::SeqCst);
        EXCEPTION_COUNT.store(0, Ordering::SeqCst);
    }
}

/// System call interface
pub mod syscall {
    /// System call numbers
    #[derive(Debug, Clone, Copy)]
    pub enum SyscallNumber {
        Exit = 0,
        Write = 1,
        Read = 2,
        Open = 3,
        Close = 4,
        Mmap = 5,
        Munmap = 6,
        Fork = 7,
        Exec = 8,
        Wait = 9,
        Yield = 10,
    }
    
    /// System call handler
    #[no_mangle]
    pub extern "C" fn syscall_handler() {
        println!("System call received");
        
        // Phase 1: Basic syscall handling
        // Phase 2: Full syscall implementation
        
        // Get syscall number from registers
        let syscall_number = 0; // Phase 2: Read from RAX
        
        println!("System call: {}", syscall_number);
        
        match syscall_number {
            0 => {
                // Exit syscall
                println!("Exit syscall");
            }
            1 => {
                // Write syscall
                println!("Write syscall");
            }
            _ => {
                println!("Unknown syscall: {}", syscall_number);
            }
        }
    }
    
    /// Make system call
    pub fn syscall(number: SyscallNumber, arg1: u64, arg2: u64, arg3: u64) -> u64 {
        let result: u64;
        
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") number as u64,
                in("rdi") arg1,
                in("rsi") arg2,
                in("rdx") arg3,
                out("rax") result,
            );
        }
        
        result
    }
}
