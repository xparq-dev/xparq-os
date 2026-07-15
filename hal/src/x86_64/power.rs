// XPARQ OS - x86_64 Power Driver
// Power management driver for x86_64 with basic shutdown/reboot support

use crate::power::{PowerDriver, PowerError, PowerSource, PowerSourceType, PowerSourceStatus, 
                   BatteryInfo, BatteryTechnology, BatteryHealth, PowerState, PowerPolicy, 
                   PowerStatistics, ThermalInfo, ThermalPolicy};
use arrayvec::ArrayVec;
use core::ptr::write_volatile;

// x86 I/O port functions
#[cfg(target_arch = "x86_64")]
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
unsafe fn outw(port: u16, value: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
}

/// x86_64 power driver
pub struct X86PowerDriver {
    initialized: bool,
}

impl X86PowerDriver {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    /// Shutdown the system using ACPI (fallback to QEMU debug exit)
    pub fn shutdown(&mut self) -> ! {
        unsafe {
            // Try true ACPI shutdown first
            if crate::x86_64::acpi::ACPI_STATE.initialized {
                if let Some(pm1a_port) = crate::x86_64::acpi::ACPI_STATE.pm1a_control_block {
                    // SLP_TYPa = 5 for QEMU/Bochs, SLP_EN = 1<<13
                    // Value = (5 << 10) | (1 << 13) = 0x1400 | 0x2000 = 0x3400
                    let slp_typa = 5;
                    let slp_en = 1 << 13;
                    let value = (slp_typa << 10) | slp_en;
                    outw(pm1a_port as u16, value);
                }
            }
            
            // QEMU debug exit fallback (port 0x501)
            outb(0x501, 0x01);
        }
        // Fallback to infinite loop
        loop {
            unsafe { core::arch::asm!("hlt", options(nomem, nostack)); }
        }
    }

    /// Reboot the system using the 8042 keyboard controller
    pub fn reboot(&mut self) -> ! {
        unsafe {
            // Wait until the keyboard controller is ready
            loop {
                let status = inb(0x64);
                if (status & 0x02) == 0 {
                    break;
                }
            }
            // Send reset command
            outb(0x64, 0xFE);
        }
        // Fallback to infinite loop
        loop {
            unsafe { core::arch::asm!("hlt", options(nomem, nostack)); }
        }
    }
}

// Need to define inb too for the reboot function
#[cfg(target_arch = "x86_64")]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

impl Default for X86PowerDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerDriver for X86PowerDriver {
    fn name(&self) -> &'static str {
        "x86_64 Power Driver"
    }

    fn init(&mut self) -> Result<(), PowerError> {
        self.initialized = true;
        Ok(())
    }

    fn get_power_sources(&self) -> ArrayVec<PowerSource, 8> {
        let mut sources = ArrayVec::new();
        sources.push(PowerSource {
            id: 0,
            source_type: PowerSourceType::AC,
            status: PowerSourceStatus::Full,
            capacity: None,
            voltage: Some(19000), // 19V
            current: None,
        });
        sources
    }

    fn get_battery_info(&self) -> Option<BatteryInfo> {
        None
    }

    fn set_power_state(&mut self, state: PowerState) -> Result<(), PowerError> {
        match state {
            PowerState::Off => {
                self.shutdown();
            }
            PowerState::Reset => {
                self.reboot();
            }
            _ => {}
        }
        Ok(())
    }

    fn get_power_state(&self) -> PowerState {
        PowerState::On
    }

    fn set_power_policy(&mut self, _policy: PowerPolicy) -> Result<(), PowerError> {
        Ok(())
    }

    fn get_power_policy(&self) -> PowerPolicy {
        PowerPolicy::default()
    }

    fn get_power_statistics(&self) -> PowerStatistics {
        PowerStatistics {
            uptime: 0,
            sleep_time: 0,
            deep_sleep_time: 0,
            hibernate_time: 0,
            power_cycles: 0,
            battery_cycles: None,
            energy_consumed: 0,
            last_charge_time: None,
        }
    }

    fn set_power_saving(&mut self, _enabled: bool) -> Result<(), PowerError> {
        Ok(())
    }

    fn is_power_saving_enabled(&self) -> bool {
        false
    }

    fn get_thermal_info(&self) -> Option<ThermalInfo> {
        None
    }

    fn set_thermal_policy(&mut self, _policy: ThermalPolicy) -> Result<(), PowerError> {
        Ok(())
    }

    fn get_thermal_policy(&self) -> ThermalPolicy {
        ThermalPolicy::default()
    }
}
