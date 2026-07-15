// XPARQ OS - x86-64 HAL
// Syscall MSR initialization

use core::arch::asm;
use spin::Mutex;

const MSR_STAR: u32 = 0xC0000081;
const MSR_LSTAR: u32 = 0xC0000082;
const MSR_SFMASK: u32 = 0xC0000084;
const MSR_EFER: u32 = 0xC0000080;

pub struct SyscallManager;
pub static SYSCALL_MANAGER: Mutex<SyscallManager> = Mutex::new(SyscallManager);

impl SyscallManager {
    pub fn init(&mut self, handler_addr: u64) {
        unsafe {
            // Enable SCE (System Call Enable) bit in EFER
            let mut efer_low: u32;
            let mut efer_high: u32;
            asm!("rdmsr", out("eax") efer_low, out("edx") efer_high, in("ecx") MSR_EFER);
            
            let efer = ((efer_high as u64) << 32) | (efer_low as u64);
            let new_efer = efer | 1 | (1 << 11); // SCE = bit 0, NXE = bit 11
            asm!("wrmsr", in("eax") (new_efer & 0xFFFFFFFF) as u32, in("edx") (new_efer >> 32) as u32, in("ecx") MSR_EFER);

            // Configure STAR (Segment Selector base for syscall/sysret)
            // SYSRET CS/SS are loaded from bits 63:48, SYSCALL CS/SS are loaded from bits 47:32
            // Assuming Kernel CS = 0x08, SS = 0x10, User CS32 = 0x18, User SS = 0x20, User CS64 = 0x28
            let star: u64 = (0x18 << 48) | (0x08 << 32);
            asm!("wrmsr", in("eax") (star & 0xFFFFFFFF) as u32, in("edx") (star >> 32) as u32, in("ecx") MSR_STAR);

            // Configure LSTAR (Target RIP for syscall)
            asm!("wrmsr", in("eax") (handler_addr & 0xFFFFFFFF) as u32, in("edx") (handler_addr >> 32) as u32, in("ecx") MSR_LSTAR);

            // Configure SFMASK (RFLAGS mask cleared on syscall)
            // Clear IF (Interrupts), TF (Trap), DF (Direction), etc.
            let sfmask: u64 = 0x200; // Clear IF (disable interrupts upon syscall)
            asm!("wrmsr", in("eax") (sfmask & 0xFFFFFFFF) as u32, in("edx") (sfmask >> 32) as u32, in("ecx") MSR_SFMASK);
        }
    }
}
