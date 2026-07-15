// XPARQ OS - x86-64 GDT and TSS
// Sets up Ring 0 and Ring 3 segments

use core::arch::asm;
use bitflags::bitflags;

// TSS requires a 128-bit descriptor, other entries are 64-bit
#[repr(C, packed)]
pub struct GdtDescriptor {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_middle: u8,
    pub access: u8,
    pub granularity: u8,
    pub base_high: u8,
}

impl GdtDescriptor {
    pub const fn new(base: u32, limit: u32, access: u8, granularity: u8) -> Self {
        Self {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: ((limit >> 16) & 0x0F) as u8 | (granularity & 0xF0),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

#[repr(C, packed)]
pub struct TssDescriptor {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_middle: u8,
    pub access: u8,
    pub granularity: u8,
    pub base_high: u8,
    pub base_upper: u32,
    pub reserved: u32,
}

impl TssDescriptor {
    pub const fn new(base: u64, limit: u32) -> Self {
        Self {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access: 0x89, // Present, Ring 0, TSS
            granularity: ((limit >> 16) & 0x0F) as u8,
            base_high: ((base >> 24) & 0xFF) as u8,
            base_upper: (base >> 32) as u32,
            reserved: 0,
        }
    }
}

#[repr(C, packed)]
pub struct TaskStateSegment {
    pub reserved_1: u32,
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    pub reserved_2: u64,
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    pub reserved_3: u64,
    pub reserved_4: u16,
    pub iopb_offset: u16,
}

impl TaskStateSegment {
    pub const fn new() -> Self {
        Self {
            reserved_1: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            reserved_2: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            reserved_3: 0,
            reserved_4: 0,
            iopb_offset: 104,
        }
    }
}

#[repr(C, packed)]
pub struct Gdt {
    pub null: GdtDescriptor,
    pub kernel_code: GdtDescriptor,
    pub kernel_data: GdtDescriptor,
    pub user_code32: GdtDescriptor,
    pub user_data: GdtDescriptor,
    pub user_code64: GdtDescriptor,
    pub tss: TssDescriptor,
}

#[repr(C, packed)]
pub struct GdtPointer {
    pub limit: u16,
    pub base: u64,
}

pub static mut TSS: TaskStateSegment = TaskStateSegment::new();

pub static mut GDT: Gdt = Gdt {
    null: GdtDescriptor::new(0, 0, 0, 0),
    kernel_code: GdtDescriptor::new(0, 0xFFFFF, 0x9A, 0xAF), // 0x08
    kernel_data: GdtDescriptor::new(0, 0xFFFFF, 0x92, 0xAF), // 0x10
    user_code32: GdtDescriptor::new(0, 0xFFFFF, 0xFA, 0xCF), // 0x18
    user_data: GdtDescriptor::new(0, 0xFFFFF, 0xF2, 0xAF),   // 0x20
    user_code64: GdtDescriptor::new(0, 0xFFFFF, 0xFA, 0xAF), // 0x28
    tss: TssDescriptor::new(0, 103),                         // 0x30
};

pub static mut GDT_PTR: GdtPointer = GdtPointer { limit: 0, base: 0 };

pub fn init() {
    unsafe {
        let tss_base = &raw const TSS as u64;
        GDT.tss = TssDescriptor::new(tss_base, core::mem::size_of::<TaskStateSegment>() as u32 - 1);

        GDT_PTR.limit = (core::mem::size_of::<Gdt>() - 1) as u16;
        GDT_PTR.base = &raw const GDT as u64;

        let ptr = &raw const GDT_PTR as *const GdtPointer;
        asm!("lgdt [{}]", in(reg) ptr);

        // Load segments
        asm!(
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            in("ax") 0x10,
        );

        // Load CS using far return trick
        asm!(
            "push 0x08",
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            "2:",
        );

        // Load TSS
        asm!("ltr ax", in("ax") 0x30);
    }
}

pub fn set_kernel_stack(rsp: u64) {
    unsafe {
        TSS.rsp0 = rsp;
    }
}
