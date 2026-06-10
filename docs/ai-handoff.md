# XPARQ OS AI Handoff

This document is a short, machine-readable handoff for the next AI agent or teammate taking over the repository.

## Read First

1. `docs/implementation-status.md`
2. `docs/build-contract.md`
3. `README.md`

## Current Truth

- Phase 2 boot verification is complete.
- ARM64 and x86-64 both boot in QEMU.
- The authoritative kernel entrypoint is `kernel/src/main_simple.rs`.
- Windows PowerShell scripts in `tools/` are the canonical build entrypoints on this machine.
- `x86_64` runtime uses `build/x86-64/disk.img`.
- Phase 3 has not started yet.

## Authoritative Paths

- Workspace root: `Cargo.toml`
- Kernel: `kernel/`
- HAL: `hal/`
- FIDL interfaces: `interfaces/fidl/`
- ARM64 bootloader: `bootloader/arm64/`
- x86-64 bootloader: `bootloader/x86_64/`

## Safe Next Step

Start Phase 3 from `hal/` scaffolding.

Recommended order:

1. Define HAL core traits and shared device abstractions.
2. Add architecture-specific backends for ARM64 and x86-64.
3. Keep the boot path stable unless a real bug is found.

## Do Not Assume

- Do not assume `kernel.bin` is the x86-64 runtime image.
- Do not assume Phase 3 work should modify the boot flow first.
- Do not assume prototype files are authoritative unless this document or `docs/implementation-status.md` says so.

## If You Change Ownership

If you move an authoritative path, change the status of a subsystem, or replace the boot contract, update:

- `docs/implementation-status.md`
- `docs/build-contract.md`
- any README section that points to the old path

