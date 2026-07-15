# XPARQ OS Implementation Status (Reality Check)

This document tracks what is currently implemented vs what is still roadmap-level.
It is the source of truth for execution status and should be updated continuously.

## Current Stage

- Program stage: Phase 2 complete for boot verification, **Phase 3 in progress**
- Repository type: Architecture-first with multiple active prototypes
- Risk level: Medium (prototype code still exists, but boot path is now stable)
- Phase 3 focus: HAL implementation starting with display subsystem

## Authoritative Paths (Current)

- Workspace root: `Cargo.toml`
- Kernel crate: `kernel/`
- HAL crate: `hal/`
- Interface crate: `interfaces/fidl/`
- Bootloader crates: `bootloader/arm64/`, `bootloader/x86_64/`
- Build artifact contract: `docs/build-contract.md`

## Status Ledger

| Area | Authoritative Path | State | Notes |
|---|---|---|---|
| Kernel entrypoint | `kernel/src/main_simple.rs` | in-progress | Active binary entry per `kernel/Cargo.toml` |
| Kernel richer flow | `kernel/src/main.rs`, `kernel/src/lib.rs` | in-progress | Valuable but not current binary source of truth |
| Capability model | `kernel/src/capability.rs` | in-progress | Core model exists, many placeholders remain |
| Memory model | `kernel/src/memory/` | in-progress | `VMO` is richer; `VMAR` still inconsistent in places |
| Scheduler | `kernel/src/scheduler/mod.rs` | in-progress | Core structure exists; old duplicate file still present |
| IPC/Channels | `kernel/src/ipc/` | in-progress | Framework exists, incomplete behavior |
| Syscalls | `kernel/src/syscalls.rs` | stub | Many handlers are placeholders |
| ARM bootloader | `bootloader/arm64/src/main_simple.rs` | in-progress | Cargo configured as staticlib (needs alignment later) |
| x86 bootloader | `bootloader/x86_64/src/mbr.rs` | in-progress | Multiple x86 boot variants still coexist |
| HAL | `hal/src/` | in-progress | Strong interface design, architecture-specific modules (x86_64/arm64) created, basic display drivers implemented, integrated with kernel boot path! |
| FIDL interfaces | `interfaces/fidl/src/` | in-progress | Protocol modeling exists, transport/serialization partial |
| Experimental kernel | `kernel-simple/` | prototype | Excluded from workspace; still useful for quick tests |

## Known Structural Risks

1. Multiple implementations for the same responsibility (kernel main paths, bootloader variants).
2. Duplicate architecture ownership (`kernel/arch/*` and top-level `arch/*`).
3. Legacy/prototype files mixed with active files (`*_old.rs`, `*_simple.rs` variants).
4. Documentation can overstate runtime completeness if not cross-checked with code.

## Enforcement Rules (Starting Now)

1. Every subsystem must have exactly one "authoritative path".
2. Experimental code must be clearly labeled `prototype` or moved under an experiments area.
3. Any new roadmap claim must map to a real path and status in this file.
4. If ownership changes, update this file in the same change set.

## AI Handoff Note

If you are an AI agent continuing work in this repository:

1. Read `docs/ai-handoff.md` first, then this file, then `docs/build-contract.md`.
2. The authoritative boot path is currently `kernel/src/main_simple.rs`.
3. ARM64 and x86-64 boot verification both pass in QEMU.
4. Windows PowerShell scripts in `tools/` are the canonical build entrypoints on this machine.
5. `x86_64` runtime uses `build/x86-64/disk.img`, not `kernel.bin` directly.
6. Phase 3 should start from `hal/` scaffolding, not from rewriting the boot path again.
7. If you change ownership or authoritative paths, update this file in the same commit.

## Daily Update (2026-06-11)

- What was stabilized this week (including today):
  - Created stable HAL architecture with x86_64/arm64 specific modules
  - Implemented VGA Text Mode display driver with full functionality (write, clear, scroll, cursor)
  - Added mouse cursor support to VGA driver
  - Integrated PS/2 mouse driver in kernel
  - Implemented PS/2 keyboard and mouse drivers
  - Integrated HAL with kernel boot path, including keyboard and mouse input handling
  - Created dummy power driver for x86_64
  - Created dummy storage driver for x86_64
  - ✅ Improved PS/2 keyboard driver to properly handle shift modifiers (Left/Right Shift, Caps Lock)
  - ✅ Fixed duplicate `HalCapabilities` struct in `hal/src/lib.rs`
  - ✅ Added linker script config to kernel's `.cargo/config.toml`
  - ✅ Added static driver instances to x86_64 HAL (VGA, PS/2 Keyboard, PS/2 Mouse)
  - ✅ Updated InputManager to collect events from static PS/2 drivers
  - ✅ Updated DisplayManager to use static VGA driver
  - ✅ Updated `kernel/src/main_simple.rs` to use HAL subsystems instead of direct driver instantiation
  - ✅ Updated `tools/windows/build-and-test.ps1` to build bootloader + kernel + disk image
  - ✅ **Fully bootable build working!** (produces `build/x86-64/disk.img`)

- What moved from `stub` to `in-progress`:
  - HAL display subsystem
  - HAL input subsystem
  - HAL power subsystem
  - HAL storage subsystem

- What remains blocked:
  - No USB HID drivers yet
  - No real GPU drivers

- What was completed today (2026-06-11):
  - ✅ PCIe bus enumeration using ECAM (MMIO) is now implemented!
  - ✅ Integrated PCIe manager into HAL initialization
  - ✅ Kernel now enumerates and displays all PCI devices on VGA console and serial output
  - ✅ Tested in QEMU!
  - ✅ Added VBE framebuffer mode support (1024x768, 32bpp)!
  - ✅ Created X86Display driver that automatically uses VBE if available, falls back to VGA text mode!
  - ✅ Updated bootloader to set VBE mode and store mode info at 0x7E00
  - ✅ Updated hal/src/x86_64/display.rs, hal/src/x86_64/mod.rs, and kernel/src/main_simple.rs to use new X86Display!
  - ✅ Fully buildable and testable in QEMU!
  - ✅ Added VGA font support (created vga-font.bin with generate_font.rs)!
  - ✅ Updated display driver to use the VGA font for text rendering in VBE mode!
  - ✅ Improved storage driver! Replaced dummy implementation with RAM disk support (64 MB, 512-byte sectors)!
  - ✅ Added static storage driver instance to hal/src/x86_64/mod.rs!
  - ✅ Improved power driver! Added shutdown and reboot support!
  - ✅ Added static power driver instance to hal/src/x86_64/mod.rs!
  - ✅ Added ATA/IDE support to storage driver!
  - ✅ Updated .gitignore to not ignore vga-font.bin!
  - ✅ Added Connectivity and USB subsystem modules to HAL!
  - ✅ Updated HAL lib.rs to include new modules!
  - ✅ Updated DeviceManager to handle new connectivity and USB drivers!
  - ✅ Added Audio and Sensors subsystem modules to HAL!
  - ✅ Updated DeviceManager to handle new audio and sensor drivers!
  - ✅ Updated HAL lib.rs init() to initialize audio and sensors too!
  - ✅ Implemented foundational USB Device Enumeration (xHCI event ring polling, port reset, and enable slot commands).
  - ✅ **Phase 3.5 Started!** Built the first Desktop GUI Window Manager using the VBE Framebuffer. Mouse events now move a graphical cursor and can activate windows!
  - ✅ **Phase 3.5 GUI Polish!** Implemented Z-Order stacking, window dragging, floating dock taskbar, minimize/maximize/close buttons, and in-window text rendering!
  - ✅ Updated docs/implementation-status.md!

- Next smallest executable step:
  - Advance the USB HID stack: Issue Address Device command and parse HID Report Descriptors.
