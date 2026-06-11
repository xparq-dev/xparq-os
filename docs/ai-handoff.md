# XPARQ OS AI Handoff (2026-06-11)

This document is a short, machine-readable handoff for the next AI agent or teammate taking over the repository.

## Read First

1. `docs/implementation-status.md` (most important!)
2. `docs/build-contract.md`
3. `README.md`

## Current Truth

- **Phase 2 complete** for boot verification.
- **Phase 3 in progress** (HAL implementation)!
- The authoritative kernel entrypoint is `kernel/src/main_simple.rs`.
- Windows PowerShell scripts in `tools/` are the canonical build entrypoints on this machine.
- `x86_64` runtime uses `build/x86-64/disk.img` (created by `tools/windows/build-and-test.ps1`).
- **Fully functional bootable build exists**!

## Authoritative Paths

- Workspace root: `Cargo.toml`
- Kernel: `kernel/`
- HAL: `hal/`
  - Display subsystem: `hal/src/display/`
  - Input subsystem: `hal/src/input/`
  - x86_64 arch-specific HAL: `hal/src/x86_64/`
- FIDL interfaces: `interfaces/fidl/`
- ARM64 bootloader: `bootloader/arm64/`
- x86-64 bootloader: `bootloader/x86_64/` (built with NASM)

## Current Phase 3 Status

- ✅ Stable HAL architecture with `x86_64/arm64` modules
- ✅ Static driver instances in `x86_64/mod.rs` for VGA, PS2 Keyboard, PS2 Mouse
- ✅ DisplayManager + InputManager updated to use static drivers
- ✅ Kernel uses HAL subsystems for input/display

## Safe Next Steps (Phase 3)

1. **Test boot in QEMU** if QEMU is installed (`tools/windows/test-boot.ps1` or `test-quick.ps1`)
2. **Continue Phase 3**:
   - Implement PCIe bus enumeration for `x86_64`
   - Work on VBE framebuffer mode (replace text mode)
   - Improve power/storage HAL drivers beyond dummy
3. **Keep the boot path stable** unless real bug is found

## Do Not Assume

- Do not assume `kernel.bin` is the x86-64 runtime image (use `disk.img`)
- Do not assume prototype files are authoritative unless `docs/implementation-status.md` says so

## If You Change Ownership

If you move an authoritative path, change the status of a subsystem, or replace the boot contract, update:

- `docs/implementation-status.md`
- `docs/build-contract.md`
- any README section that points to the old path

