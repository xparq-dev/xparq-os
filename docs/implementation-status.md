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

## Weekly Update (2026-06-10)

- What was stabilized this week:
  - Created stable HAL architecture with x86_64/arm64 specific modules
  - Implemented VGA Text Mode display driver with full functionality (write, clear, scroll, cursor)
  - Added mouse cursor support to VGA driver
  - Integrated PS/2 mouse driver in kernel
  - Implemented PS/2 keyboard and mouse drivers
  - Integrated HAL with kernel boot path, including keyboard and mouse input handling
  - Created dummy power driver for x86_64
  - Created dummy storage driver for x86_64
  - ✅ Improved PS/2 keyboard driver to properly handle shift modifiers (Left/Right Shift, Caps Lock)

- What moved from `stub` to `in-progress`:
  - HAL display subsystem
  - HAL input subsystem
  - HAL power subsystem
  - HAL storage subsystem

- What remains blocked:
  - No USB HID drivers yet
  - No PCIe bus enumeration
  - No real GPU drivers
  - No actual bootable build yet (Rust not installed)

- Next smallest executable step:
  - Install Rust toolchain
  - Build XPARQ OS using build-and-test.ps1
  - Test in QEMU using test-boot.ps1
  - ✅ Added mouse cursor support to VGA display driver
  - ✅ Added dummy power and storage drivers for x86_64
